//! what the app offers a surface, found by structure: the registry's
//! kinds and parameters, and the snapshot's roles and parents. nothing
//! here knows an app's names; each thing found keeps the app's own words
//! (labels, keys, id names) for the intent's words to choose among.

use std::collections::BTreeSet;

use sem::{ActionDecl, Actor, Actors, Node, Role, SemId, Snapshot, StateRef, Value, ValueKind};
use uir::{Capabilities, CapabilityRef};

use crate::list::{List, lists};
use crate::words::{fold, folded, split};

/// true when `value` is a frame ref: a text naming a media or image node
/// of `app` and an instant, `<id>@<seconds>`, the form an app projects a
/// frame it can decode in (the silences: `silences.video@4.017052`). an
/// `image` bound to it shows that frame. the node named is looked up by
/// its role, never by its name.
pub(crate) fn is_frame_ref(app: &Snapshot, value: &Value) -> bool {
    let Value::Text(text) = value else {
        return false;
    };
    let Some((node, at)) = text.rsplit_once('@') else {
        return false;
    };
    let Ok(id) = SemId::parse(node) else {
        return false;
    };
    at.parse::<f64>().is_ok_and(f64::is_finite)
        && app
            .get(&id)
            .is_some_and(|n| matches!(n.role, Role::Media | Role::Image))
}

/// a text input of the app.
#[derive(Clone, Debug)]
pub(crate) struct Entry {
    /// the app's node.
    pub node: SemId,
    pub label: String,
    /// the text it holds.
    pub text: StateRef,
    /// the action that writes it: a text parameter.
    pub write: CapabilityRef,
    /// what commits it: an action with no parameter on the node itself, or
    /// on a button beside it (then with that button's label).
    pub commit: Option<(CapabilityRef, Option<String>)>,
    pub words: BTreeSet<String>,
}

/// one option of a choice.
#[derive(Clone, Debug)]
pub(crate) struct Opt {
    pub cap: CapabilityRef,
    pub label: String,
    pub words: BTreeSet<String>,
}

/// a choice of the app: a node whose children are options.
#[derive(Clone, Debug)]
pub(crate) struct Choice {
    pub node: SemId,
    pub label: String,
    pub words: BTreeSet<String>,
    pub options: Vec<Opt>,
}

/// a number the app shows (an item of a list shows its own in the list).
#[derive(Clone, Debug)]
pub(crate) struct Count {
    pub at: StateRef,
    /// its key: the node's label may carry the value itself.
    pub label: String,
    pub words: BTreeSet<String>,
}

/// a media or image node of the app with the frames it offers.
#[derive(Clone, Debug)]
pub(crate) struct Media {
    pub node: SemId,
    pub label: String,
    /// the node's words and the keys of its views.
    pub words: BTreeSet<String>,
    /// each frame ref state, with its key: the views of the media
    /// (`original`, `result`), in projection order.
    pub views: Vec<(StateRef, String)>,
}

/// everything the app offers a surface, in projection order.
#[derive(Clone, Debug, Default)]
pub(crate) struct Parts {
    pub entries: Vec<Entry>,
    pub choices: Vec<Choice>,
    pub counts: Vec<Count>,
    pub lists: Vec<List>,
    pub media: Vec<Media>,
}

/// true when `actors` lets `actor` ask, as the dispatcher reads it.
pub(crate) fn allows(actors: &Actors, actor: &Actor) -> bool {
    match actor {
        Actor::Human => actors.human,
        Actor::Agent { .. } => actors.agent,
        Actor::Assistive => actors.assistive,
        Actor::Test { .. } => actors.test,
    }
}

/// the keys of an id, `[...]` segments, as text.
fn keys(id: &SemId) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = id.as_str();
    while let Some(open) = rest.find('[') {
        let after = &rest[open + 1..];
        let Some(close) = after.find(']') else {
            break;
        };
        out.push(&after[..close]);
        rest = &after[close + 1..];
    }
    out
}

/// the last name segment of an id, without its keys.
fn last_name(id: &SemId) -> &str {
    let text = id.as_str();
    let bare = text.split('[').next().unwrap_or(text);
    bare.rsplit('.').next().unwrap_or(bare)
}

/// the words a node goes by: its label, its last name, its keys.
fn node_words(node: &Node) -> BTreeSet<String> {
    let mut words = folded(&node.label);
    words.extend(split(last_name(&node.id)).map(fold));
    for key in keys(&node.id) {
        words.extend(folded(key));
    }
    words.retain(|w| !w.chars().all(|c| c.is_ascii_digit()));
    words
}

impl Parts {
    /// what `app` offers a surface an IR by `author` may bind: every
    /// action here is in `caps`, allowed to `author` and not destructive
    /// (a surface binds a destructive action only to a dialog's confirm).
    pub fn of(caps: &Capabilities, app: &Snapshot, author: &Actor) -> Parts {
        let usable = |node: &Node, decl: &ActionDecl| {
            caps.action(&node.id, &decl.name).is_some()
                && allows(&decl.policy.actors, author)
                && !decl.policy.destructive
        };
        let cap = |node: &Node, decl: &ActionDecl| CapabilityRef {
            target: node.id.clone(),
            action: decl.name.clone(),
        };
        let frame = |v: &Value| is_frame_ref(app, v);
        let mut parts = Parts {
            lists: lists(caps, app, &usable, &frame),
            ..Parts::default()
        };
        // an item of a list shows its numbers in the list, not as counts
        let listed: BTreeSet<SemId> = parts.lists.iter().flat_map(|l| l.members(app)).collect();
        let mut groups: Vec<SemId> = Vec::new();
        for node in app.nodes().values() {
            let bare: Vec<&ActionDecl> = node
                .actions
                .iter()
                .filter(|a| a.param.is_none() && usable(node, a))
                .collect();
            if node.role == Role::TextInput {
                let text = node.state.iter().find(|(_, v)| v.kind() == ValueKind::Text);
                let write = node.actions.iter().find(|a| {
                    a.param.is_some_and(|p| p.kind == ValueKind::Text) && usable(node, a)
                });
                if let (Some((key, _)), Some(write)) = (text, write) {
                    let text = StateRef::new(node.id.clone(), key.clone());
                    if caps.datum(&text).is_some() {
                        let commit = match bare.first() {
                            Some(a) => Some((cap(node, a), None)),
                            None => beside(app, node, &usable),
                        };
                        parts.entries.push(Entry {
                            node: node.id.clone(),
                            label: node.label.clone(),
                            text,
                            write: cap(node, write),
                            commit,
                            words: node_words(node),
                        });
                    }
                }
            }
            let option = matches!(node.role, Role::Tab | Role::Option | Role::Radio);
            if let (true, [only], Some(parent)) = (option, bare.as_slice(), &node.parent) {
                if node.actions.len() == 1 {
                    let at = match groups.iter().position(|g| g == parent) {
                        Some(at) => at,
                        None => {
                            let label = app.get(parent).map_or("", |p| p.label.as_str());
                            parts.choices.push(Choice {
                                node: parent.clone(),
                                label: label.to_owned(),
                                words: folded(label),
                                options: Vec::new(),
                            });
                            groups.push(parent.clone());
                            groups.len() - 1
                        }
                    };
                    parts.choices[at].options.push(Opt {
                        cap: cap(node, only),
                        label: node.label.clone(),
                        words: node_words(node),
                    });
                }
            }
            if node.constraints.secret {
                continue;
            }
            for (key, value) in &node.state {
                let at = StateRef::new(node.id.clone(), key.clone());
                let number = matches!(value.kind(), ValueKind::Int | ValueKind::Num);
                if number && !listed.contains(&node.id) && caps.datum(&at).is_some() {
                    parts.counts.push(Count {
                        label: key.as_str().to_owned(),
                        words: split(key.as_str()).map(fold).collect(),
                        at,
                    });
                }
            }
            if matches!(node.role, Role::Media | Role::Image) {
                let views: Vec<(StateRef, String)> = node
                    .state
                    .iter()
                    .filter(|(_, v)| is_frame_ref(app, v))
                    .map(|(key, _)| (StateRef::new(node.id.clone(), key.clone()), key.to_string()))
                    .filter(|(at, _)| caps.datum(at).is_some())
                    .collect();
                if !views.is_empty() {
                    let mut words = node_words(node);
                    for (_, key) in &views {
                        words.extend(split(key).map(fold));
                    }
                    parts.media.push(Media {
                        node: node.id.clone(),
                        label: node.label.clone(),
                        words,
                        views,
                    });
                }
            }
        }
        parts
    }

    /// every word the app uses for what a surface can show, sorted.
    pub fn words(&self) -> Vec<String> {
        let mut all = BTreeSet::new();
        for e in &self.entries {
            all.extend(e.words.iter().cloned());
        }
        for c in &self.choices {
            all.extend(c.words.iter().cloned());
            for o in &c.options {
                all.extend(o.words.iter().cloned());
            }
        }
        for c in &self.counts {
            all.extend(c.words.iter().cloned());
        }
        for l in &self.lists {
            all.extend(l.words.iter().cloned());
        }
        for m in &self.media {
            all.extend(m.words.iter().cloned());
        }
        all.into_iter().collect()
    }

    /// true when the app has something of `part`.
    pub fn has(&self, part: crate::words::Part) -> bool {
        use crate::words::Part;
        match part {
            Part::Entry => !self.entries.is_empty(),
            Part::Choice => !self.choices.is_empty(),
            Part::Count => !self.counts.is_empty(),
            Part::List => !self.lists.is_empty(),
            Part::Media => !self.media.is_empty(),
        }
    }
}

/// the commit of a text input that has none of its own: a button beside
/// it (the same parent) with one action and no parameter.
fn beside(
    app: &Snapshot,
    input: &Node,
    usable: &impl Fn(&Node, &ActionDecl) -> bool,
) -> Option<(CapabilityRef, Option<String>)> {
    let parent = input.parent.as_ref()?;
    app.children(parent).find_map(|n| {
        let [only] = n.actions.as_slice() else {
            return None;
        };
        let fits = n.role == Role::Button && only.param.is_none() && usable(n, only);
        fits.then(|| {
            let cap = CapabilityRef {
                target: n.id.clone(),
                action: only.name.clone(),
            };
            (cap, Some(n.label.clone()))
        })
    })
}
