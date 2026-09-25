//! the deterministic proposer: an intent read by a small grammar, turned
//! into bindings by rules over the registry's structure.
//!
//! the rules, and what each binds:
//!
//! - **entry** ("add", "adicionar"): for each text input of the app (role
//!   `text_input`, a `text` state, an action with a text parameter), a
//!   `text_field` bound to the state, the writing action (`set_value`) and
//!   the action with no parameter on the same node (`submit`, the Enter
//!   key), and a `button` bound to that action; with none on the node, a
//!   button beside it (the same parent) with one action and no parameter.
//! - **choice** ("filter", "filtro"): for each choice of the app (a node
//!   whose children are tabs, options or radios with one action and no
//!   parameter), a container of `chip`s, each bound to its option's
//!   action.
//! - **count** ("counts", "stats", "contagem"): one `stat` per `int` or
//!   `num` state, named by its key.
//! - **list** ("items", "list", "itens", "lista"): for each list of the
//!   app (a `List` or `Set` state whose values are the keys of its node's
//!   children of one kind, all of them), a `list` that repeats over it;
//!   each item a row of what every item declares alike, by sense: what
//!   the item is (an `image` per frame ref, a `text` per other text, a
//!   `badge` per decision), its numbers (a small `stat` each, named by
//!   its key), its actions (a `checkbox` per bool with an action of no
//!   parameter that reads only it, a `button` per other action of no
//!   parameter); each sense of two nodes or more is a group of its own
//!   (`list.rs`, `row.rs`).
//! - **media** ("video", "image", "frame", "vídeo", "imagem", "quadro"):
//!   for each media or image node of the app with a frame ref state, an
//!   `image` bound to it: the first view (the first such state), or the
//!   views a word of the intent names by their key; with **compare**
//!   ("compare", "comparar"), every view, side by side. the media goes
//!   on top.
//! - **form** ("compact", "two columns", "compacto", "duas colunas"): the
//!   layout, the lists under the entry. without one, the current
//!   surface's form (two columns when its root is a grid), else compact.
//! - **everything** ("everything", "tudo"): every part.
//!
//! every other word of the intent is a word of the app (a label, an id's
//! name or key, a state's key) and chooses: a part the intent names keeps
//! only its members the word names (all of them when it names none), a
//! part it does not name shows only what the word names. a word that is
//! neither the grammar's nor the app's is an error that lists the app's
//! words. an intent with only a form reshapes what the current surface
//! shows. nothing here names an app: the same rules read the todo of the
//! showcase and the ide's workbench.

use std::collections::BTreeSet;

use sem::{Node, Role};
use uir::{
    Align, Axis, Component, Elevation, Ir, IrNode, Layout, Props, Size, SizeToken, SpacingToken,
    Surface,
};

use crate::error::ProposeError;
use crate::ids::{Anchor, Namer, anchor, anchors};
use crate::list::List;
use crate::parts::{Choice, Count, Entry, Media, Parts};
use crate::words::{Form, GRAMMAR, Part, Reading, fold, read};
use crate::{Ask, Proposer};

/// the deterministic proposer. it has no state: the same ask gives the
/// same IR.
#[derive(Clone, Copy, Debug, Default)]
pub struct Rules;

/// the members of each part a proposal shows.
#[derive(Default)]
struct Chosen {
    entries: Vec<Entry>,
    choices: Vec<Choice>,
    counts: Vec<Count>,
    lists: Vec<List>,
    media: Vec<Media>,
}

impl Chosen {
    fn is_empty(&self) -> bool {
        let none = self.entries.is_empty() && self.choices.is_empty();
        none && self.counts.is_empty() && self.lists.is_empty() && self.media.is_empty()
    }
}

/// the views of `m` the intent shows: every one with "compare", the ones
/// a word names by their key, else the first.
fn views(m: &Media, r: &Reading) -> Vec<(sem::StateRef, String)> {
    if r.compare {
        return m.views.clone();
    }
    let named: Vec<_> = m
        .views
        .iter()
        .filter(|(_, key)| r.words.iter().any(|(_, w)| fold(key) == *w))
        .cloned()
        .collect();
    if !named.is_empty() {
        return named;
    }
    m.views.iter().take(1).cloned().collect()
}

/// true when one of `words` names something with `own` words.
fn names(words: &[(String, String)], own: &BTreeSet<String>) -> bool {
    words.iter().any(|(_, w)| own.contains(w))
}

/// the members to show: the named part keeps what the words name (all
/// when they name none of it), an unnamed part only what they name.
fn keep<T: Clone>(
    all: &[T],
    named: bool,
    words: &[(String, String)],
    own: impl Fn(&T) -> bool,
) -> Vec<T> {
    let hit: Vec<T> = all.iter().filter(|m| own(m)).cloned().collect();
    if named && hit.is_empty() {
        all.to_vec()
    } else if named || !words.is_empty() {
        hit
    } else {
        Vec::new()
    }
}

fn choose(r: &Reading, parts: &Parts) -> Chosen {
    let named = |p: Part| r.everything || r.parts.contains(&p);
    let w = &r.words;
    let choices: Vec<Choice> = parts
        .choices
        .iter()
        .filter_map(|c| {
            let options = if names(w, &c.words) {
                c.options.clone()
            } else {
                keep(&c.options, named(Part::Choice), w, |o| names(w, &o.words))
            };
            let mut c = c.clone();
            c.options = options;
            (!c.options.is_empty()).then_some(c)
        })
        .collect();
    let media = keep(&parts.media, named(Part::Media), w, |m| names(w, &m.words))
        .into_iter()
        .map(|mut m| {
            m.views = views(&m, r);
            m
        })
        .collect();
    Chosen {
        entries: keep(&parts.entries, named(Part::Entry), w, |e| {
            names(w, &e.words)
        }),
        choices,
        counts: keep(&parts.counts, named(Part::Count), w, |c| names(w, &c.words)),
        lists: keep(&parts.lists, named(Part::List), w, |l| names(w, &l.words)),
        media,
    }
}

/// what the current surface shows, as parts: an intent with only a form
/// reshapes it.
fn shown(parts: &Parts, current: &IrNode) -> Chosen {
    let on = anchors(current);
    let cap = |c: &uir::CapabilityRef| on.contains(&Anchor::Action(c.clone()));
    let choices = parts
        .choices
        .iter()
        .filter_map(|c| {
            let mut c = c.clone();
            c.options.retain(|o| cap(&o.cap));
            (!c.options.is_empty()).then_some(c)
        })
        .collect();
    Chosen {
        entries: parts
            .entries
            .iter()
            .filter(|e| cap(&e.write))
            .cloned()
            .collect(),
        choices,
        counts: parts
            .counts
            .iter()
            .filter(|c| on.contains(&Anchor::State(c.at.clone())))
            .cloned()
            .collect(),
        lists: parts
            .lists
            .iter()
            .filter(|l| on.contains(&Anchor::Each(l.each.clone())))
            .cloned()
            .collect(),
        media: parts
            .media
            .iter()
            .filter_map(|m| {
                let mut m = m.clone();
                m.views
                    .retain(|(at, _)| on.contains(&Anchor::State(at.clone())));
                (!m.views.is_empty()).then_some(m)
            })
            .collect(),
    }
}

pub(crate) fn stack(axis: Axis, gap: SpacingToken, align: Option<Align>) -> Option<Layout> {
    Some(Layout::Stack {
        axis,
        gap: Some(gap),
        align,
    })
}

/// the first `max` characters of `text`.
fn clip(text: &str, max: usize) -> String {
    text.chars().take(max).collect()
}

/// a label that names something: `text`, or `or` when it is blank.
fn named(text: &str, or: &str, max: usize) -> String {
    let t = if text.trim().is_empty() { or } else { text };
    clip(t, max)
}

pub(crate) struct Build<'a> {
    pub namer: Namer,
    pub max: usize,
    add: Option<&'a str>,
}

impl Build<'_> {
    pub(crate) fn bound(&mut self, mut node: IrNode, name: &str, label: String) -> IrNode {
        let (id, label) = match anchor(&node) {
            Some(a) => self.namer.bound(&a, name, label),
            None => (node.sem.clone(), label),
        };
        node.sem = id;
        node.label = Some(clip(&label, self.max));
        node
    }

    fn container(
        &mut self,
        name: &str,
        label: Option<String>,
        layout: Option<Layout>,
        children: Vec<IrNode>,
    ) -> IrNode {
        let mut held = BTreeSet::new();
        for c in &children {
            held.extend(anchors(c));
        }
        let (id, label) = self.namer.container(&held, name, label);
        let mut node = IrNode::new(id);
        node.label = label.map(|l| clip(&l, self.max));
        node.layout = layout;
        node.children = children;
        node
    }

    fn entry(&mut self, e: &Entry, form: Form) -> IrNode {
        let mut field = IrNode::new(e.node.clone());
        field.component = Some(Component::TextField);
        field.bind.state.insert("text".to_owned(), e.text.clone());
        field
            .bind
            .actions
            .insert("set_value".to_owned(), e.write.clone());
        if let Some((commit, _)) = &e.commit {
            field
                .bind
                .actions
                .insert("submit".to_owned(), commit.clone());
        }
        if form == Form::Compact {
            field.size = Size {
                flex: Some(1),
                ..Size::default()
            };
        }
        let label = named(&e.label, e.text.key.as_str(), self.max);
        let mut children = vec![self.bound(field, e.node.as_str(), label)];
        if let Some((commit, beside)) = &e.commit {
            let mut button = IrNode::new(commit.target.clone());
            button.component = Some(Component::Button);
            button
                .bind
                .actions
                .insert("activate".to_owned(), commit.clone());
            let label = beside
                .clone()
                .or_else(|| self.add.map(str::to_owned))
                .unwrap_or_else(|| commit.action.as_str().to_owned());
            let name = format!("{}_{}", e.node, commit.action);
            children.push(self.bound(button, &name, label));
        }
        let layout = match form {
            Form::Compact => stack(Axis::Horizontal, SpacingToken::Sm, Some(Align::Center)),
            Form::Columns => stack(Axis::Vertical, SpacingToken::Sm, Some(Align::Stretch)),
        };
        self.container("entry", None, layout, children)
    }

    fn choice(&mut self, c: &Choice, form: Form) -> IrNode {
        let chips = c
            .options
            .iter()
            .map(|o| {
                let mut chip = IrNode::new(o.cap.target.clone());
                chip.component = Some(Component::Chip);
                chip.bind
                    .actions
                    .insert("activate".to_owned(), o.cap.clone());
                let label = named(&o.label, o.cap.target.as_str(), self.max);
                self.bound(chip, o.cap.target.as_str(), label)
            })
            .collect();
        let layout = match form {
            Form::Compact => stack(Axis::Horizontal, SpacingToken::Xs, None),
            Form::Columns => stack(Axis::Vertical, SpacingToken::Xs, Some(Align::Start)),
        };
        let label = (!c.label.trim().is_empty()).then(|| c.label.clone());
        self.container(c.node.as_str(), label, layout, chips)
    }

    fn counts(&mut self, counts: &[Count], form: Form) -> IrNode {
        let stats = counts
            .iter()
            .map(|c| {
                let mut stat = IrNode::new(c.at.node.clone());
                stat.component = Some(Component::Stat);
                stat.bind.state.insert("value".to_owned(), c.at.clone());
                let name = format!("{}.{}", c.at.node, c.at.key);
                self.bound(stat, &name, c.label.clone())
            })
            .collect();
        let layout = match form {
            Form::Compact => Some(Layout::Grid {
                min_col: SizeToken::CardMinW,
                gap: Some(SpacingToken::Sm),
            }),
            Form::Columns => stack(Axis::Vertical, SpacingToken::Sm, None),
        };
        self.container("counts", None, layout, stats)
    }

    /// the views of a media node: one `image` per view, bound to its
    /// frame ref, named by the node when there is one view and by the
    /// view's key when there are more, which then sit side by side.
    fn media(&mut self, m: &Media) -> IrNode {
        let one = m.views.len() == 1;
        let mut images: Vec<IrNode> = m
            .views
            .iter()
            .map(|(at, key)| {
                let mut image = IrNode::new(m.node.clone());
                image.component = Some(Component::Image);
                image.bind.state.insert("source".to_owned(), at.clone());
                let label = if one {
                    named(&m.label, key, self.max)
                } else {
                    key.clone()
                };
                let name = format!("{}.{}", m.node, key);
                self.bound(image, &name, label)
            })
            .collect();
        if one && let Some(image) = images.pop() {
            return image;
        }
        let grid = Layout::Grid {
            min_col: SizeToken::FieldMinW,
            gap: Some(SpacingToken::Lg),
        };
        let name = format!("{}_views", m.node);
        self.container(&name, None, Some(grid), images)
    }

    /// `nodes` as one node: itself when it is one, a column otherwise.
    fn column(&mut self, name: &str, mut nodes: Vec<IrNode>) -> Option<IrNode> {
        match nodes.len() {
            0 => None,
            1 => nodes.pop(),
            _ => {
                let layout = stack(Axis::Vertical, SpacingToken::Md, None);
                Some(self.container(name, None, layout, nodes))
            }
        }
    }

    fn surface(&mut self, chosen: &Chosen, form: Form, card: Option<String>) -> IrNode {
        // the media on top, then the entries, and the lists under them
        let mut main: Vec<IrNode> = chosen.media.iter().map(|m| self.media(m)).collect();
        main.extend(chosen.entries.iter().map(|e| self.entry(e, form)));
        for l in &chosen.lists {
            let list = self.list(l);
            main.push(list);
        }
        let mut side: Vec<IrNode> = chosen
            .choices
            .iter()
            .map(|c| self.choice(c, form))
            .collect();
        if !chosen.counts.is_empty() {
            side.push(self.counts(&chosen.counts, form));
        }
        let (children, layout) = match form {
            Form::Compact => {
                let mut all = main;
                all.extend(side);
                (all, stack(Axis::Vertical, SpacingToken::Md, None))
            }
            Form::Columns => {
                let side = self.column("side", side);
                let main = self.column("main", main);
                let grid = Layout::Grid {
                    min_col: SizeToken::FieldMinW,
                    gap: Some(SpacingToken::Lg),
                };
                (side.into_iter().chain(main).collect(), Some(grid))
            }
        };
        let (id, label) = self.namer.root(card);
        let mut root = IrNode::new(id);
        root.component = Some(Component::Card);
        root.label = label.map(|l| clip(&l, self.max));
        root.props = Props {
            padding: Some(SpacingToken::Lg),
            elevation: Some(Elevation::Raised),
            ..Props::default()
        };
        root.layout = layout;
        root.children = children;
        root
    }
}

/// the label of the app's first region, for a new surface's card.
fn region(app: &sem::Snapshot) -> Option<String> {
    app.nodes()
        .values()
        .find(|n: &&Node| n.role == Role::Region && !n.label.trim().is_empty())
        .map(|n| n.label.clone())
}

impl Proposer for Rules {
    fn propose(&mut self, ask: &Ask<'_>) -> Result<Ir, ProposeError> {
        let reading = read(ask.intent);
        if reading.is_empty() {
            return Err(ProposeError::NotUnderstood {
                intent: ask.intent.trim().to_owned(),
                grammar: GRAMMAR.to_owned(),
            });
        }
        let parts = Parts::of(ask.caps, ask.app, &ask.policy.author);
        let app_words = parts.words();
        let unknown: Vec<String> = reading
            .words
            .iter()
            .filter(|(_, w)| !app_words.contains(w))
            .map(|(written, _)| written.clone())
            .collect();
        if !unknown.is_empty() {
            return Err(ProposeError::NoMatch {
                words: unknown,
                app: app_words,
            });
        }
        for part in Part::ALL {
            if reading.parts.contains(&part) && !parts.has(part) {
                return Err(ProposeError::Nothing {
                    part,
                    missing: missing(part),
                });
            }
        }
        let current = ask.current.map(|v| v.ir());
        let everything = || {
            let mut all = reading.clone();
            all.everything = true;
            choose(&all, &parts)
        };
        let only_form = reading.parts.is_empty() && !reading.everything && reading.words.is_empty();
        let chosen = match (only_form, current) {
            (true, Some(ir)) => {
                let now = shown(&parts, &ir.surface.root);
                if now.is_empty() { everything() } else { now }
            }
            (true, None) => everything(),
            (false, _) => choose(&reading, &parts),
        };
        if chosen.is_empty() {
            return Err(ProposeError::Empty);
        }
        let form = reading.form.unwrap_or(match current {
            Some(ir) if matches!(ir.surface.root.layout, Some(Layout::Grid { .. })) => {
                Form::Columns
            }
            _ => Form::Compact,
        });
        let max = ask.policy.max_text;
        let mut build = Build {
            namer: Namer::new(current, ask.app),
            max,
            add: reading.add.as_deref(),
        };
        let root = build.surface(&chosen, form, region(ask.app));
        let title = match current {
            Some(ir) => ir.surface.title.clone(),
            None => clip(ask.intent.trim(), max),
        };
        Ok(Ir::new(Surface {
            id: build.namer.surface().clone(),
            title,
            root,
        }))
    }
}

fn missing(part: Part) -> String {
    match part {
        Part::Entry => "no text input with an action that takes a text".to_owned(),
        Part::Choice => {
            "no choice (tabs, options or radios whose action takes no parameter)".to_owned()
        }
        Part::Count => "no number (an int or a num state)".to_owned(),
        Part::List => {
            "no list of items (a list or a set state whose values key its node's children)"
                .to_owned()
        }
        Part::Media => {
            "no media (a media or image node with a frame ref state, a text naming a media \
             node and an instant)"
                .to_owned()
        }
    }
}
