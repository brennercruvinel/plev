//! The person's preferences of accessibility over the surfaces of a host
//! (task-70 phase 4): a surface opens with them before its first frame;
//! turned on with one open, it takes them at once, in the person's name,
//! in the same history; an agent does not set them; turned off, what the
//! preference changed goes back byte for byte and what others asked in
//! between stays; what anyone asks applies under them, "undo" never takes
//! them back, a node shown after takes them, and a new form takes them
//! too. What a model is shown (`current`) never has them.

// shared with the other surface tests: this binary uses the fixtures and
// the page.
#[allow(dead_code)]
mod support;

use comps::{Rect, Wake};
use engine::theme::Theme;
use sem::{Actor, Snapshot};
use support::{COMPARE_SILENCES, REVIEW_SILENCES, SILENCES, id, page, settle, snapshot};
use support::{TURN, verified_local};
use surface::{SurfaceError, SurfaceHost};
use uir::{
    Adapted, Capabilities, ColorRole, Context, GenerationPolicy, Op, Preference, Preferences,
    TypographyRole,
};

const AREA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 900.0,
    h: 1400.0,
};

fn agent() -> Actor {
    Actor::Agent {
        name: "claude".to_owned(),
    }
}

/// Larger text and more contrast.
fn both() -> Preferences {
    Preferences::default()
        .with(Preference::LargerText, true)
        .with(Preference::MoreContrast, true)
}

/// The review of the Silences over its app, and the host with it open.
fn open(prefs: Option<Preferences>) -> (Snapshot, SurfaceHost) {
    let app = snapshot(SILENCES);
    let mut host = SurfaceHost::new();
    if let Some(prefs) = prefs {
        host.prefer(prefs, Actor::Human, &app)
            .expect("the person's");
    }
    host.open(verified_local(REVIEW_SILENCES, &app), &app)
        .expect("opens");
    (app, host)
}

/// The surface the host draws, written.
fn drawn(host: &SurfaceHost) -> String {
    let surface = host.get(&id("review")).expect("open");
    serde_json::to_string(surface.verified().ir()).expect("an IR writes")
}

fn written(a: &Adapted) -> String {
    serde_json::to_string(a.ir()).expect("an IR writes")
}

/// `prefs` over the review as it was asked, as `uir` makes them.
fn made(app: &Snapshot, asked: &Adapted, prefs: Preferences) -> Adapted {
    let caps = Capabilities::from_snapshot(app);
    let policy = GenerationPolicy::default();
    let cx = Context {
        caps: &caps,
        policy: &policy,
    };
    prefs.over(asked, &Actor::Human, cx).adapted
}

fn typography(host: &SurfaceHost, at: &str) -> Option<TypographyRole> {
    let surface = host.get(&id("review")).expect("open");
    let root = &surface.verified().ir().surface.root;
    fn find<'a>(n: &'a uir::IrNode, at: &str) -> Option<&'a uir::IrNode> {
        if n.sem.as_str() == at {
            return Some(n);
        }
        n.children
            .iter()
            .find_map(|c| find(c, at))
            .or_else(|| n.repeat.as_ref().and_then(|r| find(&r.item, at)))
    }
    find(root, at).and_then(|n| n.props.typography)
}

#[test]
fn a_surface_opens_with_the_preferences_before_its_first_frame() {
    let (app, mut host) = open(Some(both()));
    let review = id("review");
    let asked = Adapted::new(verified_local(REVIEW_SILENCES, &app));
    assert_eq!(drawn(&host), written(&made(&app, &asked, both())));
    assert_eq!(
        typography(&host, "review.cut[{item}].range"),
        Some(TypographyRole::Title)
    );
    // the history has them, the person's, and nothing asked
    let changes = host.changes(&review);
    assert!(!changes.is_empty());
    assert!(
        changes
            .iter()
            .all(|c| c.preference.is_some() && c.by == Actor::Human)
    );
    assert!(host.last_ask(&review).is_empty());
    // what a model is shown is the review as it was asked
    let current = host.current(&review).expect("open");
    assert_eq!(written(&current), written(&asked));
    // born with them: the first frame is at rest, nothing to transition
    let theme = Theme::hoff_core();
    let (_, after) = settle(host.get_mut(&review).expect("open"), AREA, &theme);
    assert_eq!(after, 0, "no frame after the first");
}

#[test]
fn turned_on_with_a_surface_open_it_takes_them_at_once_in_the_person_s_name() {
    let (app, mut host) = open(None);
    let review = id("review");
    let before = drawn(&host);
    let seen = page(&app, host.get(&review).expect("open"));
    host.prefer(both(), Actor::Assistive, &app)
        .expect("assistive technology is the person's");
    assert_ne!(drawn(&host), before);
    assert_eq!(host.preferences(), both());
    let changes = host.changes(&review);
    assert!(changes.iter().all(|c| c.by == Actor::Assistive));
    let named: Vec<Preference> = changes.iter().filter_map(|c| c.preference).collect();
    assert!(named.contains(&Preference::LargerText));
    assert!(named.contains(&Preference::MoreContrast));
    // the projection does not say it: a surface projects no look
    let now = page(&app, host.get(&review).expect("open"));
    assert_eq!(now.revision(), seen.revision());
    // "undo" takes back what was asked, and nothing was
    let undone = host.undo_asked(&review, &app);
    assert_eq!(undone, Err(SurfaceError::NothingToUndo(review.clone())));
}

#[test]
fn an_agent_does_not_set_the_preferences() {
    let (app, mut host) = open(None);
    let before = drawn(&host);
    let refused = host.prefer(both(), agent(), &app);
    assert_eq!(refused, Err(SurfaceError::NotThePerson(agent())));
    assert_eq!(host.preferences(), Preferences::default());
    assert_eq!(drawn(&host), before);
    assert!(host.changes(&id("review")).is_empty());
}

/// The acceptance: the preference on, an agent hides a node, the person
/// turns the preference off: the node stays hidden, the agent's change
/// stays in the history as the agent's, and the rest is back byte for
/// byte, the review with the hide alone.
#[test]
fn turned_off_what_it_changed_goes_back_and_what_an_agent_asked_stays() {
    let (app, mut host) = open(Some(both()));
    let review = id("review");
    let hide = Op::Hide {
        id: id("review.detection"),
    };
    host.apply(&review, hide.clone(), agent(), &app)
        .expect("the agent hides");
    assert_eq!(
        typography(&host, "review.cut[{item}].range"),
        Some(TypographyRole::Title),
        "the preference over the agent's change"
    );
    // one off: the other stays
    let contrast = both().with(Preference::LargerText, false);
    host.prefer(contrast, Actor::Human, &app).expect("off");
    assert_eq!(typography(&host, "review.cut[{item}].range"), None);
    // both off: the review with the agent's hide, byte for byte
    host.prefer(Preferences::default(), Actor::Human, &app)
        .expect("off");
    let caps = Capabilities::from_snapshot(&app);
    let policy = GenerationPolicy::default();
    let cx = Context {
        caps: &caps,
        policy: &policy,
    };
    let asked = Adapted::new(verified_local(REVIEW_SILENCES, &app));
    let (expected, _) = asked.apply(&hide, &agent(), cx).expect("the hide");
    assert_eq!(drawn(&host), written(&expected));
    assert_eq!(host.adapted(&review), Some(&expected), "aside the same");
    let changes = host.changes(&review);
    assert_eq!(changes.len(), 1);
    assert_eq!((&changes[0].op, &changes[0].by), (&hide, &agent()));
    assert_eq!(changes[0].preference, None);
}

/// What anyone asks applies under the preferences, which are made again
/// over it; "undo" takes back only the ask, and a node shown after the
/// preference takes it.
#[test]
fn an_ask_applies_under_them_and_a_node_shown_after_takes_them() {
    let (app, mut host) = open(None);
    let review = id("review");
    host.apply(
        &review,
        Op::Hide {
            id: id("review.cuts"),
        },
        Actor::Human,
        &app,
    )
    .expect("hidden");
    host.prefer(both(), Actor::Human, &app).expect("on");
    let preferred = drawn(&host);
    host.apply(
        &review,
        Op::Show {
            id: id("review.cuts"),
        },
        agent(),
        &app,
    )
    .expect("shown");
    assert_eq!(
        typography(&host, "review.cut[{item}].range"),
        Some(TypographyRole::Title),
        "shown after, with the preference"
    );
    let surface = host.get(&review).expect("open");
    let root = &surface.verified().ir().surface.root;
    let cuts = root
        .children
        .iter()
        .find(|n| n.sem == id("review.cuts"))
        .expect("the cuts");
    let item = &cuts.repeat.as_ref().expect("a repeat").item;
    let decision = item
        .children
        .iter()
        .flat_map(|n| n.children.iter())
        .find(|n| n.sem == id("review.cut[{item}].decision"))
        .expect("the badge");
    assert_eq!(decision.props.background, Some(ColorRole::Accent));
    // "undo" takes the show back, and the preferences stay
    let undone = host.undo_asked(&review, &app).expect("undone");
    assert_eq!(undone.len(), 1);
    assert_eq!(drawn(&host), preferred);
    assert_eq!(host.preferences(), both());
}

#[test]
fn a_new_form_takes_them_too_and_a_model_is_shown_it_as_it_came() {
    let (app, mut host) = open(Some(both()));
    let compare = verified_local(COMPARE_SILENCES, &app);
    let asked = Adapted::new(compare.clone());
    host.transform(compare, &app).expect("the new form");
    assert_eq!(drawn(&host), written(&made(&app, &asked, both())));
    let current = host.current(&id("review")).expect("open");
    assert_eq!(written(&current), written(&asked));
    // and it settles: the change of form is drawn and is over
    let theme = Theme::hoff_core();
    settle(host.get_mut(&id("review")).expect("open"), AREA, &theme);
    assert_eq!(host.wake(TURN), Wake::idle());
}
