//! The person's preferences of accessibility as operations (task-70
//! phase 4): each preference is a list of phase 1 operations by token,
//! each verified again, identity kept, the inverses giving the surface
//! back byte for byte, and nothing but tokens in what they make; what
//! each reaches and what it leaves, said.

// shared with the other uir tests: this binary uses the registry only.
#[allow(dead_code)]
use crate::support;

use std::collections::BTreeSet;

use accord::{Actor, ParamSpec, SemId, ValueKind};
use plevir::{
    ActionCap, Adapted, Capabilities, ColorRole, Context, DataCap, GenerationPolicy, Ir, IrNode,
    Op, Preference, Preferences, SpacingToken, Step, TypographyRole, verify, verify_json,
};
use serde_json::{Value, json};
use support::caps;

/// A surface with a text of every kind a preference reads: a headline,
/// one at the top of the ramp, one in each secondary ink, one in an
/// intent; a stat, a slider, a table, an image; the files repeated with a
/// badge and a dim name; a button.
fn doc() -> Value {
    json!({
        "plev_ir": 1,
        "surface": {
            "id": "look",
            "title": "a look at the commit",
            "root": {
                "sem": "look.root",
                "props": { "padding": "lg" },
                "layout": { "stack": { "axis": "vertical", "gap": "md" } },
                "children": [
                    { "sem": "look.title", "component": "text", "label": "the commit",
                      "props": { "typography": "headline" } },
                    { "sem": "look.top", "component": "text", "label": "on top",
                      "props": { "typography": "h4" } },
                    { "sem": "look.dim", "component": "text", "label": "dim",
                      "props": { "color": "text_dim" } },
                    { "sem": "look.mid", "component": "text", "label": "mid",
                      "props": { "color": "text_mid" } },
                    { "sem": "look.ok", "component": "text", "label": "ok",
                      "props": { "color": "success" } },
                    { "sem": "look.count", "component": "stat", "label": "files",
                      "bind": { "state": { "value": "counter:count" } } },
                    { "sem": "look.level", "component": "slider", "label": "level",
                      "bind": { "state": { "value": "meter:level" },
                                "actions": { "set_value": {
                                    "target": "meter", "action": "set_value"
                                } } } },
                    { "sem": "look.rows", "component": "table", "label": "the files",
                      "bind": { "state": { "rows": "changes:files" } } },
                    { "sem": "look.frame", "component": "image", "label": "the message",
                      "bind": { "state": { "source": "commit.message:text" } } },
                    {
                        "sem": "look.files",
                        "component": "list",
                        "label": "files",
                        "layout": { "stack": { "axis": "vertical", "gap": "xs" } },
                        "repeat": {
                            "each": "changes:files",
                            "item": {
                                "sem": "look.file[{item}]",
                                "layout": { "stack": { "axis": "horizontal", "gap": "sm" } },
                                "children": [
                                    { "sem": "look.file[{item}].status", "component": "badge",
                                      "bind": { "state": { "text": "changes.file[{item}]:status" } } },
                                    { "sem": "look.file[{item}].name", "component": "text",
                                      "props": { "color": "text_mid" },
                                      "bind": { "state": { "text": "changes.file[{item}]:status" } } }
                                ]
                            }
                        }
                    },
                    { "sem": "look.commit", "component": "button", "label": "commit",
                      "bind": { "actions": { "activate": {
                          "target": "commit.submit", "action": "submit"
                      } } } }
                ]
            }
        }
    })
}

/// `caps()` with a number for the slider, and the action that sets it.
fn registry() -> Capabilities {
    let set = ActionCap {
        param: Some(ParamSpec {
            kind: ValueKind::Num,
            constrained: true,
        }),
        ..ActionCap::new(id("meter"), "set_value".parse().unwrap())
    };
    caps()
        .with_data(DataCap {
            at: "meter:level".parse().unwrap(),
            kind: ValueKind::Num,
            label: "level".to_owned(),
        })
        .with_action(set)
}

fn id(text: &str) -> SemId {
    SemId::parse(text).unwrap()
}

fn start(caps: &Capabilities, policy: &GenerationPolicy) -> Adapted {
    let ir: Ir = serde_json::from_value(doc()).expect("the document reads");
    Adapted::new(verify(&ir, caps, policy).unwrap_or_else(|e| panic!("{e:#?}")))
}

fn node<'a>(a: &'a Adapted, at: &str) -> &'a IrNode {
    fn find<'a>(n: &'a IrNode, at: &str) -> Option<&'a IrNode> {
        if n.sem.as_str() == at {
            return Some(n);
        }
        n.children
            .iter()
            .find_map(|c| find(c, at))
            .or_else(|| n.repeat.as_ref().and_then(|r| find(&r.item, at)))
    }
    find(&a.ir().surface.root, at).unwrap_or_else(|| panic!("no node {at}"))
}

fn written(a: &Adapted) -> String {
    serde_json::to_string(a.ir()).expect("an IR writes")
}

/// Every combination of the preferences: each alone, and all of them.
fn every() -> Vec<Preferences> {
    let mut out: Vec<Preferences> = Preference::ALL
        .into_iter()
        .map(|p| Preferences::default().with(p, true))
        .collect();
    out.push(Preferences {
        larger_text: true,
        more_contrast: true,
        density: Some(Step::Sparser),
    });
    out
}

#[test]
fn every_preference_is_operations_by_token_verified_with_identity_and_their_inverse() {
    let (caps, policy) = (registry(), GenerationPolicy::by(Actor::Human));
    let cx = Context {
        caps: &caps,
        policy: &policy,
    };
    let before = start(&caps, &policy);
    for prefs in every() {
        let made = prefs.over(&before, &Actor::Human, cx);
        assert!(!made.applied.is_empty(), "{prefs:?} changes something");
        for applied in &made.applied {
            assert!(
                matches!(
                    applied.op,
                    Op::Emphasize { .. } | Op::Restyle { .. } | Op::Density { .. }
                ),
                "{prefs:?}: {} is an operation on a look",
                applied.op
            );
            assert!(prefs.has(applied.preference));
        }
        // verified again, and every id of the surface still in its tree
        let after = &made.adapted;
        assert!(verify(after.ir(), &caps, &policy).is_ok(), "{prefs:?}");
        assert_eq!(after.ids(), before.ids(), "{prefs:?}: identity");
        // nothing but tokens: what they make reads back as an IR
        let json = written(after);
        let read = verify_json(&json, &caps, &policy).expect("the result reads and verifies");
        assert_eq!(read.ir(), after.ir());
        // the inverses, the last first, give the surface back
        let mut back = after.clone();
        for applied in made.applied.iter().rev() {
            back = back.undo(&applied.inverse, cx).expect("undone");
        }
        assert_eq!(written(&back), written(&before), "{prefs:?}: byte for byte");
    }
    // none on changes nothing
    let none = Preferences::default().over(&before, &Actor::Human, cx);
    assert!(none.applied.is_empty() && none.refused.is_empty());
    assert_eq!(none.adapted, before);
}

#[test]
fn larger_text_climbs_every_text_and_stat_one_step_and_says_what_is_at_the_top() {
    let (caps, policy) = (registry(), GenerationPolicy::default());
    let cx = Context {
        caps: &caps,
        policy: &policy,
    };
    let before = start(&caps, &policy);
    let prefs = Preferences::default().with(Preference::LargerText, true);
    let made = prefs.over(&before, &Actor::Human, cx);
    let after = &made.adapted;
    let ty = |at: &str| node(after, at).props.typography;
    assert_eq!(ty("look.title"), Some(TypographyRole::H4));
    assert_eq!(ty("look.dim"), Some(TypographyRole::Title), "body, one up");
    assert_eq!(ty("look.count"), Some(TypographyRole::Headline), "a stat");
    assert_eq!(
        ty("look.file[{item}].name"),
        Some(TypographyRole::Title),
        "the template, for every item"
    );
    // the top of the ramp stays, and is said
    assert_eq!(ty("look.top"), Some(TypographyRole::H4));
    let top: Vec<String> = made
        .refused
        .iter()
        .map(|(_, op, _)| op.to_string())
        .collect();
    assert_eq!(top, ["emphasize look.top"]);
    // what draws its text at the component's size is not reached
    for at in [
        "look.level",
        "look.rows",
        "look.frame",
        "look.commit",
        "look.file[{item}].status",
    ] {
        assert_eq!(node(after, at).props, node(&before, at).props, "{at}");
    }
}

#[test]
fn more_contrast_takes_the_strongest_ink_and_fills_a_badge() {
    let (caps, policy) = (registry(), GenerationPolicy::default());
    let cx = Context {
        caps: &caps,
        policy: &policy,
    };
    let before = start(&caps, &policy);
    let prefs = Preferences::default().with(Preference::MoreContrast, true);
    let made = prefs.over(&before, &Actor::Human, cx);
    assert!(made.refused.is_empty(), "{:?}", made.refused);
    let after = &made.adapted;
    for at in [
        "look.dim",
        "look.mid",
        "look.file[{item}].name",
        "look.level",
        "look.rows",
        "look.frame",
    ] {
        assert_eq!(node(after, at).props.color, Some(ColorRole::Text), "{at}");
    }
    assert_eq!(
        node(after, "look.file[{item}].status").props.background,
        Some(ColorRole::Accent),
        "the badge filled"
    );
    // an intent's ink is its meaning; a text with no ink is `text`; a
    // stat and a button are drawn in the theme's own colors
    for at in ["look.ok", "look.title", "look.count", "look.commit"] {
        assert_eq!(node(after, at).props, node(&before, at).props, "{at}");
    }
    let changed: BTreeSet<String> = made.applied.iter().map(|a| a.op.to_string()).collect();
    assert_eq!(changed.len(), 7);
}

#[test]
fn a_density_is_one_step_and_one_takes_the_place_of_the_other() {
    let denser = Preferences::default().with(Preference::Denser, true);
    assert_eq!(denser.density, Some(Step::Denser));
    let sparser = denser.with(Preference::Sparser, true);
    assert_eq!(
        sparser.density,
        Some(Step::Sparser),
        "one takes the other's place"
    );
    assert!(!sparser.has(Preference::Denser));
    assert_eq!(sparser.with(Preference::Denser, false), sparser);
    assert_eq!(sparser.toggled(Preference::Sparser), Preferences::default());
    let all = Preferences {
        larger_text: true,
        more_contrast: true,
        density: Some(Step::Denser),
    };
    assert_eq!(
        all.each(),
        [
            Preference::LargerText,
            Preference::MoreContrast,
            Preference::Denser
        ]
    );
    let (caps, policy) = (registry(), GenerationPolicy::default());
    let cx = Context {
        caps: &caps,
        policy: &policy,
    };
    let before = start(&caps, &policy);
    let after = denser.over(&before, &Actor::Human, cx).adapted;
    assert_eq!(
        node(&after, "look.root").props.padding,
        Some(SpacingToken::Md)
    );
    let gap = |a: &Adapted, at: &str| match node(a, at).layout {
        Some(plevir::Layout::Stack { gap, .. }) => gap,
        _ => None,
    };
    assert_eq!(gap(&after, "look.root"), Some(SpacingToken::Sm));
    assert_eq!(gap(&after, "look.files"), Some(SpacingToken::Xs), "held");
    assert_eq!(gap(&after, "look.file[{item}]"), Some(SpacingToken::Xs));
}

/// The two roles `restyle` gained for "more contrast": a badge's fill
/// (its filled shape, by intent) and the ink of the text the host draws
/// for a slider, a table and an image.
#[test]
fn restyle_reaches_a_badge_s_fill_and_the_ink_of_a_slider_a_table_an_image() {
    let (caps, policy) = (registry(), GenerationPolicy::default());
    let cx = Context {
        caps: &caps,
        policy: &policy,
    };
    let before = start(&caps, &policy);
    let restyle =
        |at: &str, role| before.apply(&Op::Restyle { id: id(at), role }, &Actor::Human, cx);
    let (danger, _) = restyle("look.file[{item}].status", ColorRole::Danger).expect("filled");
    assert_eq!(
        node(&danger, "look.file[{item}].status").props.background,
        Some(ColorRole::Danger)
    );
    let why = restyle("look.file[{item}].status", ColorRole::Bg)
        .err()
        .map(|e| e.to_string());
    assert!(
        why.as_deref()
            .is_some_and(|w| w.contains("a badge's fill is one of accent, success, danger, info")),
        "{why:?}"
    );
    for at in ["look.level", "look.rows", "look.frame"] {
        let (ink, _) = restyle(at, ColorRole::Text).expect("an ink");
        assert_eq!(node(&ink, at).props.color, Some(ColorRole::Text));
        let why = restyle(at, ColorRole::BgPanel).err().map(|e| e.to_string());
        assert!(
            why.as_deref()
                .is_some_and(|w| w.contains("color is one of")),
            "{at}: {why:?}"
        );
    }
    // a stat and a button: the theme's own colors
    for at in ["look.count", "look.commit"] {
        let why = restyle(at, ColorRole::Text).err().map(|e| e.to_string());
        assert!(
            why.as_deref()
                .is_some_and(|w| w.contains("theme's own colors")),
            "{at}: {why:?}"
        );
    }
}
