//! One invalid IR per rule of the verifier, each with the rule, the JSON
//! pointer and the reason it is refused with. every IR here is `base()`
//! broken in one place, so each test names exactly what it breaks.

mod support;

use sem::Actor;
use serde_json::json;
use support::{
    COMMIT, FILES, ITEM, MESSAGE, STAGE, STATUS, base, caps, errors, errors_under, refused,
    refused_under, with, without,
};
use uir::{GenerationPolicy, Ir, Rule, verify};

#[test]
fn the_base_surface_verifies() {
    assert_eq!(errors(&base()), []);
}

// rule 1: schema and version

#[test]
fn another_version_is_refused_at_the_version() {
    refused(
        &with(base(), "/plev_ir", json!(2)),
        Rule::Schema,
        "/plev_ir",
        "version 2",
    );
    let mut ir: Ir = serde_json::from_value(base()).unwrap();
    ir.plev_ir = 7;
    let errs = verify(&ir, &caps(), &GenerationPolicy::default()).unwrap_err();
    assert_eq!(errs.len(), 1);
    assert_eq!(
        (errs[0].rule, errs[0].pointer.as_str()),
        (Rule::Schema, "/plev_ir")
    );
}

#[test]
fn an_unknown_field_is_refused_where_it_is() {
    let doc = with(base(), &format!("{COMMIT}/onclick"), json!("run()"));
    refused(
        &doc,
        Rule::Schema,
        &format!("{COMMIT}/onclick"),
        "unknown field `onclick`",
    );
    let doc = without(base(), "/surface/title");
    refused(&doc, Rule::Schema, "/surface", "missing field `title`");
    let errs = uir::verify_json("{ not json", &caps(), &GenerationPolicy::default()).unwrap_err();
    assert_eq!((errs[0].rule, errs[0].pointer.as_str()), (Rule::Schema, ""));
}

// rule 2: ids valid and unique

#[test]
fn an_id_is_valid_and_used_once() {
    let doc = with(base(), &format!("{COMMIT}/sem"), json!("Review.Commit"));
    refused(&doc, Rule::Identity, &format!("{COMMIT}/sem"), "invalid");
    let doc = with(base(), &format!("{COMMIT}/sem"), json!("review.message"));
    refused(
        &doc,
        Rule::Identity,
        &format!("{COMMIT}/sem"),
        "already the id at /surface/root/children/1/sem",
    );
    let doc = with(base(), "/surface/root/sem", json!("review"));
    refused(
        &doc,
        Rule::Identity,
        "/surface/root/sem",
        "already the id at /surface/id",
    );
}

#[test]
fn the_item_lives_only_inside_a_repeat_and_once_per_id() {
    let doc = with(
        base(),
        &format!("{COMMIT}/sem"),
        json!("review.commit[{item}]"),
    );
    refused(
        &doc,
        Rule::Identity,
        &format!("{COMMIT}/sem"),
        "only means something inside a repeat",
    );
    let doc = with(base(), &format!("{STATUS}/sem"), json!("review.status"));
    refused(
        &doc,
        Rule::Identity,
        &format!("{STATUS}/sem"),
        "carries the item in its id",
    );
    let doc = with(
        base(),
        &format!("{STATUS}/sem"),
        json!("review.file[{item}].s[{item}]"),
    );
    refused(
        &doc,
        Rule::Identity,
        &format!("{STATUS}/sem"),
        "one `[{item}]` per id",
    );
}

#[test]
fn a_fixed_id_never_collides_with_an_instance() {
    let doc = with(base(), &format!("{COMMIT}/sem"), json!("review.file[a.rs]"));
    refused(
        &doc,
        Rule::Identity,
        &format!("{COMMIT}/sem"),
        "also what the template at",
    );
}

#[test]
fn a_node_with_an_app_id_sends_only_that_nodes_actions() {
    let doc = with(base(), &format!("{COMMIT}/sem"), json!("commit.message"));
    refused(
        &doc,
        Rule::Identity,
        &format!("{COMMIT}/bind/actions/activate"),
        "is the app's own node",
    );
    let doc = with(base(), &format!("{COMMIT}/sem"), json!("commit.submit"));
    assert_eq!(
        errors(&doc),
        [],
        "the app's node sending its own action is fine"
    );
}

// rule 3: vocabulary, tokens, layout primitives

#[test]
fn a_component_the_vocabulary_lacks_is_refused() {
    let doc = with(base(), &format!("{COMMIT}/component"), json!("video"));
    refused(
        &doc,
        Rule::Vocabulary,
        &format!("{COMMIT}/component"),
        "unknown variant `video`",
    );
    let doc = with(base(), &format!("{COMMIT}/component"), json!("toolbar"));
    refused(
        &doc,
        Rule::Vocabulary,
        &format!("{COMMIT}/component"),
        "unknown variant",
    );
}

#[test]
fn a_prop_is_a_token_never_a_literal() {
    let doc = with(
        base(),
        &format!("{COMMIT}/props"),
        json!({ "color": "#ff0000" }),
    );
    refused(
        &doc,
        Rule::Vocabulary,
        &format!("{COMMIT}/props/color"),
        "unknown variant `#ff0000`",
    );
    let doc = with(base(), &format!("{COMMIT}/props"), json!({ "padding": 12 }));
    refused(
        &doc,
        Rule::Vocabulary,
        &format!("{COMMIT}/props/padding"),
        "invalid type",
    );
    let doc = with(
        base(),
        &format!("{COMMIT}/props"),
        json!({ "width": "card_w" }),
    );
    refused(
        &doc,
        Rule::Vocabulary,
        &format!("{COMMIT}/props/width"),
        "unknown field `width`",
    );
    let doc = with(base(), &format!("{COMMIT}/size"), json!({ "min": 280 }));
    refused(
        &doc,
        Rule::Vocabulary,
        &format!("{COMMIT}/size/min"),
        "invalid type",
    );
}

#[test]
fn a_leaf_holds_no_children_and_arranges_nothing() {
    let doc = with(
        base(),
        &format!("{COMMIT}/children"),
        json!([{ "sem": "x" }]),
    );
    refused(
        &doc,
        Rule::Vocabulary,
        &format!("{COMMIT}/children"),
        "a button holds no children",
    );
    let doc = with(base(), &format!("{COMMIT}/layout"), json!("overlay"));
    refused(
        &doc,
        Rule::Vocabulary,
        &format!("{COMMIT}/layout"),
        "a button arranges no children",
    );
}

#[test]
fn a_layout_keeps_its_primitives_within_bounds() {
    let split = json!({ "split": { "axis": "horizontal", "ratio": 50 } });
    let doc = with(base(), "/surface/root/layout", split);
    refused(
        &doc,
        Rule::Vocabulary,
        "/surface/root/layout",
        "exactly two children, not 3",
    );
    let doc = with(
        with(
            base(),
            &format!("{ITEM}/layout"),
            json!({ "split": { "axis": "vertical", "ratio": 5 } }),
        ),
        &format!("{ITEM}/size"),
        json!({ "flex": 0 }),
    );
    refused(
        &doc,
        Rule::Vocabulary,
        &format!("{ITEM}/layout/split/ratio"),
        "ratio 5",
    );
    refused(
        &doc,
        Rule::Vocabulary,
        &format!("{ITEM}/size/flex"),
        "flex 0",
    );
    let doc = with(
        base(),
        "/surface/root/layout",
        json!({ "flow": { "wrap": true } }),
    );
    refused(
        &doc,
        Rule::Vocabulary,
        "/surface/root/layout",
        "unknown variant `flow`",
    );
}

#[test]
fn only_a_list_or_a_container_repeats_and_only_once() {
    let doc = with(base(), &format!("{FILES}/component"), json!("card"));
    refused(
        &doc,
        Rule::Vocabulary,
        &format!("{FILES}/repeat"),
        "a card repeats nothing",
    );
    let doc = with(
        base(),
        &format!("{FILES}/children"),
        json!([{ "sem": "review.extra" }]),
    );
    refused(
        &doc,
        Rule::Vocabulary,
        &format!("{FILES}/children"),
        "its instances are",
    );
    let inner = json!({ "each": "changes:files", "item": { "sem": "review.file[{item}].x" } });
    let doc = with(base(), &format!("{ITEM}/repeat"), inner);
    refused(
        &doc,
        Rule::Vocabulary,
        &format!("{ITEM}/repeat"),
        "a repeat inside a repeat",
    );
}

// rule 4: bound state exists and fits

#[test]
fn a_bound_state_exists_and_its_kind_fits() {
    let at = format!("{MESSAGE}/bind/state/text");
    refused(
        &with(base(), &at, json!("commit.message:body")),
        Rule::State,
        &at,
        "no state `commit.message:body`",
    );
    refused(
        &with(base(), &at, json!("changes:staged")),
        Rule::State,
        &at,
        "`changes:staged` is set; a text_field's `text` takes text",
    );
    let doc = with(
        base(),
        &format!("{MESSAGE}/bind/state/value"),
        json!("commit.message:text"),
    );
    refused(
        &doc,
        Rule::State,
        &format!("{MESSAGE}/bind/state/value"),
        "a text_field shows no `value`; it shows `text`",
    );
    let doc = with(
        base(),
        "/surface/root/bind",
        json!({ "state": { "text": "counter:count" } }),
    );
    refused(
        &doc,
        Rule::State,
        "/surface/root/bind/state/text",
        "a container shows no `text`; it shows nothing",
    );
}

#[test]
fn a_repeat_runs_over_a_list_or_a_set_the_app_declares() {
    let at = format!("{FILES}/repeat/each");
    refused(
        &with(base(), &at, json!("counter:count")),
        Rule::State,
        &at,
        "a repeat runs over a list or a set",
    );
    refused(
        &with(base(), &at, json!("changes:gone")),
        Rule::State,
        &at,
        "no state `changes:gone`",
    );
    assert_eq!(
        errors(&with(base(), &at, json!("changes:staged"))),
        [],
        "a set repeats too"
    );
}

#[test]
fn a_template_binding_names_every_node_that_fits_it_alike() {
    let at = format!("{STATUS}/bind/state/text");
    refused(
        &with(base(), &at, json!("changes.file[{item}]:gone")),
        Rule::State,
        &at,
        "no state of the app fits",
    );
    refused(
        &with(base(), &at, json!("odd[{item}]:v")),
        Rule::State,
        &at,
        "not all of one kind",
    );
    let doc = with(
        base(),
        &format!("{MESSAGE}/bind/state/text"),
        json!("changes.file[{item}]:status"),
    );
    refused(
        &doc,
        Rule::State,
        &format!("{MESSAGE}/bind/state/text"),
        "only means something inside a repeat",
    );
}

// rule 5: bound action exists and its parameter fits

#[test]
fn a_bound_action_exists_and_its_parameter_fits() {
    let at = format!("{COMMIT}/bind/actions/activate");
    let doc = with(
        base(),
        &at,
        json!({ "target": "commit.submit", "action": "push" }),
    );
    refused(
        &doc,
        Rule::Action,
        &at,
        "no action `push` on `commit.submit`",
    );
    let doc = with(
        base(),
        &at,
        json!({ "target": "commit.message", "action": "set_value" }),
    );
    refused(
        &doc,
        Rule::Action,
        &at,
        "`set_value` takes a text; a button's `activate` sends none",
    );
    let at = format!("{MESSAGE}/bind/actions/set_value");
    let doc = with(
        base(),
        &at,
        json!({ "target": "commit.submit", "action": "submit" }),
    );
    refused(
        &doc,
        Rule::Action,
        &at,
        "`submit` takes nothing; a text_field's `set_value` sends a text",
    );
    let at = format!("{COMMIT}/bind/actions/toggle");
    let doc = with(
        base(),
        &at,
        json!({ "target": "commit.submit", "action": "submit" }),
    );
    refused(
        &doc,
        Rule::Action,
        &at,
        "a button sends no `toggle`; it sends `activate`",
    );
}

#[test]
fn a_template_action_is_on_every_node_that_fits() {
    let at = format!("{STAGE}/bind/actions/activate");
    let doc = with(
        base(),
        &at,
        json!({ "target": "half[{item}]", "action": "go" }),
    );
    refused(&doc, Rule::Action, &at, "`half[y]` lacks it");
    let doc = with(
        base(),
        &at,
        json!({ "target": "nothing[{item}]", "action": "go" }),
    );
    refused(&doc, Rule::Action, &at, "no node of the app fits");
}

#[test]
fn a_chip_is_a_toggle_or_a_button_not_both() {
    let chip = json!({
        "sem": "review.chip",
        "component": "chip",
        "label": "staged",
        "bind": { "actions": {
            "toggle": { "target": "commit.form", "action": "cancel" },
            "activate": { "target": "commit.submit", "action": "submit" }
        } }
    });
    let doc = with(base(), "/surface/root/children/3", chip);
    refused(
        &doc,
        Rule::Action,
        "/surface/root/children/3/bind/actions",
        "not both",
    );
}

// rule 6: generation policy

#[test]
fn an_agent_binds_only_what_agents_may_ask_for() {
    let admin = json!({
        "sem": "review.reset",
        "component": "button",
        "label": "reset everything",
        "bind": { "actions": { "activate": { "target": "admin", "action": "reset" } } }
    });
    let doc = with(base(), "/surface/root/children/3", admin);
    let at = "/surface/root/children/3/bind/actions/activate";
    refused(
        &doc,
        Rule::Policy,
        at,
        "agent:unknown may not bind `reset` on `admin`: its policy leaves agents out",
    );
    let person = GenerationPolicy::by(Actor::Human);
    assert_eq!(errors_under(&doc, &person), [], "a person may bind it");
}

#[test]
fn a_destructive_action_sits_behind_a_confirmation() {
    let at = format!("{STAGE}/bind/actions/activate");
    let doc = with(
        base(),
        &at,
        json!({ "target": "changes.file[{item}]", "action": "discard" }),
    );
    refused(
        &doc,
        Rule::Policy,
        &at,
        "is destructive and the app asks no one to confirm it",
    );
    let dialog = json!({
        "sem": "review.discard",
        "component": "dialog",
        "label": "discard the changes to a.rs?",
        "description": "they are lost",
        "bind": { "actions": {
            "confirm": { "target": "changes.file[a.rs]", "action": "discard" },
            "cancel": { "target": "commit.form", "action": "cancel" }
        } }
    });
    let doc = with(base(), "/surface/root/children/3", dialog);
    assert_eq!(errors(&doc), [], "a dialog's confirm is the confirmation");
    let trash = json!({
        "sem": "review.trash",
        "component": "button",
        "label": "empty the trash",
        "bind": { "actions": { "activate": { "target": "trash", "action": "empty" } } }
    });
    let doc = with(base(), "/surface/root/children/3", trash);
    assert_eq!(
        errors(&doc),
        [],
        "the app asks everyone to confirm this one"
    );
}

// rule 7: accessibility

#[test]
fn every_control_is_named() {
    let doc = without(base(), &format!("{COMMIT}/label"));
    refused(
        &doc,
        Rule::Access,
        &format!("{COMMIT}/label"),
        "a button needs a label",
    );
    let doc = with(base(), &format!("{COMMIT}/label"), json!("   "));
    refused(
        &doc,
        Rule::Access,
        &format!("{COMMIT}/label"),
        "a button needs a label",
    );
    let doc = without(base(), &format!("{STATUS}/bind"));
    refused(
        &doc,
        Rule::Access,
        &format!("{STATUS}/label"),
        "a badge needs a label or its `text` bound",
    );
    let doc = with(base(), "/surface/title", json!(""));
    refused(
        &doc,
        Rule::Access,
        "/surface/title",
        "a surface needs a title",
    );
}

#[test]
fn a_repeated_item_is_named_by_its_item() {
    let doc = with(base(), &format!("{ITEM}/label"), json!("file"));
    refused(
        &doc,
        Rule::Access,
        &format!("{ITEM}/label"),
        "named by its item",
    );
}

#[test]
fn a_node_without_a_role_is_never_named() {
    let line = json!({ "sem": "review.line", "component": "separator", "label": "a line" });
    let doc = with(base(), "/surface/root/children/3", line);
    refused(
        &doc,
        Rule::Access,
        "/surface/root/children/3/label",
        "has no role",
    );
}

#[test]
fn a_control_is_never_only_visual() {
    let doc = without(base(), &format!("{MESSAGE}/bind/actions"));
    refused(
        &doc,
        Rule::Access,
        &format!("{MESSAGE}/bind"),
        "only visual",
    );
}

#[test]
fn the_focus_order_follows_the_tree_and_an_overlay_keeps_controls_in_one_layer() {
    let ir: Ir = serde_json::from_value(base()).unwrap();
    let verified = verify(&ir, &caps(), &GenerationPolicy::default()).unwrap();
    let order: Vec<&str> = verified
        .focus_order()
        .iter()
        .map(|id| id.as_str())
        .collect();
    assert_eq!(
        order,
        [
            "review.file[{item}].stage",
            "review.message",
            "review.commit"
        ]
    );
    let doc = with(base(), "/surface/root/layout", json!("overlay"));
    refused(
        &doc,
        Rule::Access,
        "/surface/root/layout",
        "controls in 3 layers of an overlay",
    );
}

// rule 8: limits

#[test]
fn a_surface_stays_within_the_limits_of_its_policy() {
    let few = GenerationPolicy {
        max_nodes: 3,
        ..GenerationPolicy::default()
    };
    refused_under(
        &base(),
        &few,
        Rule::Limits,
        "/surface/root",
        "7 nodes; the policy allows 3",
    );
    let shallow = GenerationPolicy {
        max_depth: 2,
        ..GenerationPolicy::default()
    };
    refused_under(
        &base(),
        &shallow,
        Rule::Limits,
        ITEM,
        "depth 3; the policy allows 2",
    );
    let terse = GenerationPolicy {
        max_text: 10,
        ..GenerationPolicy::default()
    };
    refused_under(
        &base(),
        &terse,
        Rule::Limits,
        &format!("{MESSAGE}/label"),
        "14 characters",
    );
    refused_under(
        &base(),
        &terse,
        Rule::Limits,
        "/surface/title",
        "17 characters",
    );
}

#[test]
fn every_error_comes_at_once() {
    let doc = with(
        base(),
        &format!("{COMMIT}/bind/actions/activate/action"),
        json!("push"),
    );
    let doc = with(
        doc,
        &format!("{MESSAGE}/bind/state/text"),
        json!("changes:staged"),
    );
    let doc = with(doc, &format!("{STATUS}/sem"), json!("review.status"));
    let doc = with(doc, &format!("{ITEM}/label"), json!("file"));
    let terse = GenerationPolicy {
        max_text: 10,
        ..GenerationPolicy::default()
    };
    let rules: Vec<Rule> = errors_under(&doc, &terse).iter().map(|e| e.rule).collect();
    for rule in [
        Rule::Identity,
        Rule::State,
        Rule::Action,
        Rule::Access,
        Rule::Limits,
    ] {
        assert!(rules.contains(&rule), "{rule:?} missing from {rules:?}");
    }
}
