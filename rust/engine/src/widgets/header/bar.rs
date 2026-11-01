//! A top bar of the kit: a row of parts across `theme.header`'s bar,
//! with the free width after the part that pushes the rest away (the
//! kit's `mr-auto`), and the parts that leave first when the row does not
//! fit (back and forward, the kit's `max-lg:hidden`; the asset header's
//! download and share; Create, once the search is its button). The asset header's thumbnails float at the bar's
//! middle (`absolute left-1/2`) while there is room between its two
//! sides, and leave when there is not. The search gives way last, down
//! to the narrowest useful field (`theme.size.field_min_w`); below that
//! the row changes its composition: the search is its icon button (the
//! minimal header's), and while its box has the caret it takes the row
//! alone.
//!
//! The kit's three headers are constructors: [`Header::simple`] (with
//! the filter, [`Header::simple_filters`], or the logo and the search at
//! the end, [`Header::simple_search`]), [`Header::minimal`] and
//! [`Header::asset`]. The search is the form's input group, the labeled
//! buttons the buttons' `Button`; the rest is [`super::parts`].
//!
//! Projected ([`Header::project`]) as a toolbar: each part under it as
//! `<id>.<key>` (`back`, `search`, `alerts`, `profile`...), the
//! thumbnails a group of radios `<id>.assets[<i>]`, the one current
//! `checked`. [`Header::run`] does what a click does.

use accord::{Request, Role, SemId, SnapshotBuilder, Value};

use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::Theme;
use crate::widgets::buttons::Button;
use crate::widgets::form::input_group::{Addon, InputGroup};
use crate::widgets::form::tokens::Look;
use crate::widgets::hit::hit_rect;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

use super::logo::{Logo, LogoKind};
use super::parts::{Action, ActionStyle, Avatar, Thumbs};

mod layout;

/// One part of the row.
#[derive(Clone, Debug)]
pub enum HeaderItem {
    Logo(Logo),
    Action(Action),
    /// Boxed: the input group is many times the other parts.
    Search(Box<InputGroup>),
    Button(Button),
    Avatar(Avatar),
}

/// A part, its id's key, and how the row treats it.
#[derive(Clone, Debug)]
pub struct HeaderSlot {
    pub key: &'static str,
    pub item: HeaderItem,
    /// The free width goes after it.
    pub grow: bool,
    /// The gap before it is the pair's (`gap-2`), not the row's.
    pub joined: bool,
    /// When it leaves the row that does not fit: `0` never, `1` first
    /// (back and forward, the asset's download and share), `2` once the
    /// search is already its button (Create).
    pub leaves: u8,
}

fn slot(key: &'static str, item: HeaderItem) -> HeaderSlot {
    HeaderSlot {
        key,
        item,
        grow: false,
        joined: false,
        leaves: 0,
    }
}

impl HeaderSlot {
    fn grow(mut self) -> Self {
        self.grow = true;
        self
    }
}

/// What the keyboard is on: a part of the row, or a thumbnail.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeaderPart {
    Slot(usize),
    Thumb(usize),
    /// The search's icon button.
    SearchAction,
}

/// Where a header's parts land.
#[derive(Clone, Debug, PartialEq)]
pub struct HeaderLayout {
    /// Per slot, `None` for one that left.
    pub slots: Vec<Option<Rect>>,
    pub thumbs: Option<Rect>,
    /// The search is its icon button: the row has no room for its box.
    pub compact: bool,
    /// The logo shows its symbol alone: the row has no room for its name.
    pub mark: bool,
}

/// The row's gap (`gap-4`, or the asset header's `gap-3`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Spacing {
    Wide,
    Tight,
}

#[derive(Clone, Debug)]
pub struct Header {
    pub slots: Vec<HeaderSlot>,
    pub thumbs: Option<Thumbs>,
    spacing: Spacing,
    focus: Option<HeaderPart>,
    /// The search's icon button, where the row has no room for its box.
    find: Action,
    sem: Option<SemId>,
}

fn search() -> InputGroup {
    InputGroup::new("Search...")
        .look(Look::Header)
        .start(Addon::Icon("search"))
        .end(Addon::Hint("⌘ K".into()))
}

fn alerts() -> HeaderSlot {
    slot(
        "alerts",
        HeaderItem::Action(Action::new("zap", "Notifications", ActionStyle::Plain).dot(true)),
    )
}

fn profile() -> HeaderSlot {
    slot("profile", HeaderItem::Avatar(Avatar::default()))
}

fn create() -> HeaderSlot {
    HeaderSlot {
        leaves: 2,
        ..slot("create", HeaderItem::Button(Button::new("Create")))
    }
}

/// Back (disabled: nowhere to go back to) and forward, the pair that
/// leaves first.
fn arrows() -> [HeaderSlot; 2] {
    let pair = |key, icon, label, disabled, joined| HeaderSlot {
        key,
        item: HeaderItem::Action(Action::new(icon, label, ActionStyle::Nav).disabled(disabled)),
        grow: false,
        joined,
        leaves: 1,
    };
    [
        pair("back", "arrow-left", "Back", true, false),
        pair("forward", "arrow-right", "Forward", false, true),
    ]
}

impl Header {
    fn of(slots: Vec<HeaderSlot>, spacing: Spacing) -> Self {
        Self {
            slots,
            thumbs: None,
            spacing,
            focus: None,
            find: Action::new("search", "Search", ActionStyle::Plain),
            sem: None,
        }
    }

    /// `HeaderSimple`: back and forward, the search, the alerts, Create
    /// and the profile.
    pub fn simple() -> Self {
        let [back, forward] = arrows();
        let slots = vec![
            back,
            forward,
            slot("search", HeaderItem::Search(Box::new(search()))).grow(),
            alerts(),
            create(),
            profile(),
        ];
        Self::of(slots, Spacing::Wide)
    }

    /// `HeaderSimple isFilters`: the filter in Create's place.
    pub fn simple_filters() -> Self {
        let [back, forward] = arrows();
        let filters = Action::new("filter", "Filters", ActionStyle::Plain);
        let slots = vec![
            back,
            forward,
            slot("search", HeaderItem::Search(Box::new(search()))).grow(),
            slot("filters", HeaderItem::Action(filters)),
            alerts(),
            profile(),
        ];
        Self::of(slots, Spacing::Wide)
    }

    /// `HeaderSimple isShowSearch`: the logo, the search at the end, the
    /// alerts and the profile.
    pub fn simple_search(product: impl Into<String>) -> Self {
        let slots = vec![
            slot("logo", HeaderItem::Logo(Logo::new(product))).grow(),
            slot("search", HeaderItem::Search(Box::new(search()))),
            alerts(),
            profile(),
        ];
        Self::of(slots, Spacing::Wide)
    }

    /// `HeaderMinimal`: the symbol, the search as a button, the alerts and
    /// the profile.
    pub fn minimal(product: impl Into<String>) -> Self {
        let logo = Logo::new(product).kind(LogoKind::Symbol);
        let find = Action::new("search", "Search", ActionStyle::Plain);
        let slots = vec![
            slot("logo", HeaderItem::Logo(logo)).grow(),
            slot("find", HeaderItem::Action(find)),
            alerts(),
            profile(),
        ];
        Self::of(slots, Spacing::Wide)
    }

    /// `HeaderAsset`: close, the thumbnails of `count` assets with
    /// `current` the one shown, download, share and Add to Asset.
    pub fn asset(count: usize, current: usize) -> Self {
        let framed =
            |icon, label| HeaderItem::Action(Action::new(icon, label, ActionStyle::Framed));
        let leaves = |s: HeaderSlot| HeaderSlot { leaves: 1, ..s };
        let slots = vec![
            slot("close", framed("x", "Close")).grow(),
            leaves(slot("download", framed("download", "Download"))),
            leaves(slot("share", framed("forward", "Share"))),
            slot("add", HeaderItem::Button(Button::new("Add to Asset"))),
        ];
        let mut header = Self::of(slots, Spacing::Tight);
        header.thumbs = Some(Thumbs::new(count, current));
        header
    }

    /// `logo` in the place of the header's own, in its kind (the
    /// symbol alone stays the symbol): another product's name and mark.
    pub fn with_logo(mut self, logo: Logo) -> Self {
        for s in &mut self.slots {
            if let HeaderItem::Logo(old) = &s.item {
                let mut new = logo.clone().kind(old.kind);
                if let Some(id) = old.sem_id() {
                    new = new.sem(id.clone());
                }
                s.item = HeaderItem::Logo(new);
            }
        }
        self
    }

    /// The id the header projects as; each part is `<id>.<key>`.
    pub fn sem(mut self, id: SemId) -> Self {
        for s in &mut self.slots {
            let Ok(child) = id.child(s.key) else {
                continue;
            };
            s.item = match std::mem::replace(&mut s.item, HeaderItem::Avatar(Avatar::default())) {
                HeaderItem::Logo(l) => HeaderItem::Logo(l.sem(child)),
                HeaderItem::Search(g) => HeaderItem::Search(Box::new(g.sem(child))),
                HeaderItem::Button(b) => HeaderItem::Button(b.sem(child)),
                other => other,
            };
        }
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    fn slot_id(&self, i: usize) -> Option<SemId> {
        self.sem.as_ref()?.child(self.slots.get(i)?.key).ok()
    }

    fn assets_id(&self) -> Option<SemId> {
        self.sem.as_ref()?.child("assets").ok()
    }

    /// The part `id` names.
    pub fn part(&self, id: &SemId) -> Option<HeaderPart> {
        if self.search_action(id).is_some() {
            return Some(HeaderPart::SearchAction);
        }
        if let Some(i) = (0..self.slots.len()).find(|i| self.slot_id(*i).as_ref() == Some(id)) {
            return Some(HeaderPart::Slot(i));
        }
        let assets = self.assets_id()?;
        let n = self.thumbs.as_ref()?.count;
        (0..n)
            .find(|i| &assets.key(i.to_string()) == id)
            .map(HeaderPart::Thumb)
    }

    /// The search's action, `<id>.search.action`, as the search's own.
    fn search_action(&self, id: &SemId) -> Option<usize> {
        self.slots.iter().position(|s| match &s.item {
            HeaderItem::Search(g) => g.action_id().as_ref() == Some(id),
            _ => false,
        })
    }

    /// The pointer, to every part.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let l = self.layout(bounds, theme);
        let mut result = EventResult::IGNORED;
        for (s, r) in self.slots.iter_mut().zip(&l.slots) {
            let Some(r) = *r else {
                continue;
            };
            let got = match &mut s.item {
                HeaderItem::Logo(w) => w.handle_event(event, r),
                HeaderItem::Action(w) => w.handle_event(event, r),
                // its icon button: a click gives the box the caret, and
                // the box takes the row
                HeaderItem::Search(w) if l.compact => {
                    // the field folded to its icon presses in the least
                    // target around it (44 px with a finger)
                    let got = self.find.handle_event(event, hit_rect(r, theme));
                    if got.clicked {
                        w.input.set_focused(true);
                    }
                    got
                }
                HeaderItem::Search(w) => w.handle_event(event, r, theme),
                HeaderItem::Button(w) => w.handle_event(event, r),
                HeaderItem::Avatar(w) => w.handle_event(event, r),
            };
            result = result.merge(got);
        }
        if let (Some(t), Some(r)) = (&mut self.thumbs, l.thumbs) {
            result = result.merge(t.handle_event(event, r, theme));
        }
        result
    }

    /// The keyboard's focus on `part`, or off every part. The search's
    /// focus is its caret.
    pub fn set_focus(&mut self, part: Option<HeaderPart>) {
        self.focus = part;
        for (i, s) in self.slots.iter_mut().enumerate() {
            let on = part == Some(HeaderPart::Slot(i));
            match &mut s.item {
                HeaderItem::Logo(w) => w.set_focused(on),
                HeaderItem::Action(w) => w.press.focused = on && !w.disabled,
                HeaderItem::Search(w) => w.input.set_focused(on),
                HeaderItem::Button(w) => w.set_focused(on),
                HeaderItem::Avatar(w) => w.press.focused = on,
            }
        }
        if let Some(g) = self.search_mut() {
            g.set_action_focus(part == Some(HeaderPart::SearchAction));
        }
        if let Some(t) = &mut self.thumbs {
            t.focus = match part {
                Some(HeaderPart::Thumb(i)) => Some(i),
                _ => None,
            };
        }
    }

    pub fn focus(&self) -> Option<HeaderPart> {
        self.focus
    }

    /// The id of `part`.
    pub fn part_id(&self, part: HeaderPart) -> Option<SemId> {
        match part {
            HeaderPart::Slot(i) => self.slot_id(i),
            HeaderPart::Thumb(i) => Some(self.assets_id()?.key(i.to_string())),
            HeaderPart::SearchAction => self.search()?.action_id(),
        }
    }

    /// What has the keyboard: the part a reader focused, or the search's
    /// box when a click put the caret there.
    pub fn focused_id(&self) -> Option<SemId> {
        if let Some(part) = self.focus {
            return self.part_id(part);
        }
        let search = self.search()?;
        search
            .input
            .is_focused()
            .then(|| search.sem_id().cloned())?
    }

    /// The search's text field, when the header has one.
    pub fn search_mut(&mut self) -> Option<&mut InputGroup> {
        self.slots.iter_mut().find_map(|s| match &mut s.item {
            HeaderItem::Search(g) => Some(&mut **g),
            _ => None,
        })
    }

    pub fn search(&self) -> Option<&InputGroup> {
        self.slots.iter().find_map(|s| match &s.item {
            HeaderItem::Search(g) => Some(&**g),
            _ => None,
        })
    }

    /// The bar, then its parts over it.
    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let layer = LayerId::DEFAULT;
        let h = &theme.header;
        c.push_to_layer(
            layer,
            crate::widgets::recipe::rounded_rect(
                bounds.x, bounds.y, bounds.w, bounds.h, 0.0, h.fill.0,
            ),
        );
        let l = self.layout(bounds, theme);
        for (i, (s, r)) in self.slots.iter().zip(&l.slots).enumerate() {
            let Some(r) = *r else {
                continue;
            };
            if let (HeaderItem::Action(_) | HeaderItem::Avatar(_), Some(id)) =
                (&s.item, self.slot_id(i))
            {
                c.bind_to_layer(layer, &id, [r.x, r.y, r.w, r.h]);
            }
            match &s.item {
                HeaderItem::Logo(w) if l.mark => {
                    w.clone()
                        .kind(LogoKind::Symbol)
                        .render_to_layer(c, layer, r, theme);
                }
                HeaderItem::Logo(w) => w.render_to_layer(c, layer, r, theme),
                HeaderItem::Action(w) => w.render(c, layer, r, theme),
                HeaderItem::Search(_) if l.compact => {
                    if let Some(id) = self.slot_id(i) {
                        let h = hit_rect(r, theme);
                        c.bind_to_layer(layer, &id, [h.x, h.y, h.w, h.h]);
                    }
                    self.find.render(c, layer, r, theme);
                }
                HeaderItem::Search(w) => w.render_to_layer(c, layer, r, theme),
                HeaderItem::Button(w) => w.render_to_layer(c, layer, r, theme),
                HeaderItem::Avatar(w) => w.render(c, layer, r, theme),
            }
        }
        if let (Some(t), Some(r)) = (&self.thumbs, l.thumbs) {
            if let Some(assets) = self.assets_id() {
                c.bind_to_layer(layer, &assets, [r.x, r.y, r.w, r.h]);
                for (i, tr) in t.rects(r, theme).into_iter().enumerate() {
                    let id = assets.key(i.to_string());
                    c.bind_to_layer(layer, &id, [tr.x, tr.y, tr.w, tr.h]);
                }
            }
            t.render(c, layer, r, theme);
        }
    }

    /// Declares the header as a toolbar named `label` under `parent`, its
    /// parts under it, and the focus it holds.
    pub fn project(&self, b: &mut SnapshotBuilder, label: &str, parent: &SemId) {
        let Some(id) = &self.sem else {
            return;
        };
        b.node(id, Role::Toolbar, label).parent(parent);
        for (i, s) in self.slots.iter().enumerate() {
            let Some(at) = self.slot_id(i) else {
                continue;
            };
            match &s.item {
                HeaderItem::Logo(w) => {
                    w.project(b, &at, Some(id));
                }
                HeaderItem::Action(w) => {
                    let mut node = b.node(&at, Role::Button, w.label);
                    node.parent(id).enabled(!w.disabled);
                    if w.dot {
                        node.description("New notifications");
                    }
                    node.action("activate");
                }
                HeaderItem::Search(w) => w.project(b, &at, "Search", Some(id)),
                HeaderItem::Button(w) => {
                    w.project(b, &at, Some(id));
                }
                HeaderItem::Avatar(_) => {
                    let mut node = b.node(&at, Role::Button, "Profile");
                    node.parent(id).action("activate");
                }
            }
        }
        if let (Some(t), Some(assets)) = (&self.thumbs, self.assets_id()) {
            b.node(&assets, Role::Group, "Assets")
                .parent(id)
                .state("selected", Value::Text(format!("Asset {}", t.current + 1)));
            let reads = format!("{assets}:selected");
            for i in 0..t.count {
                let mut node = b.node(
                    assets.key(i.to_string()),
                    Role::Radio,
                    format!("Asset {}", i + 1),
                );
                node.parent(&assets).state("checked", i == t.current);
                node.action("select").reads(reads.as_str());
            }
        }
    }

    /// What a click does, asked by a reader or an agent: a part's
    /// `activate` presses it (nothing else happens), a thumbnail's
    /// `select` makes it current, the search's `set_value` sets its text.
    pub fn run(&mut self, req: &Request) -> Result<(), String> {
        let action = req.action.as_str();
        if self.search_action(&req.target).is_some() {
            return (action == "activate")
                .then_some(())
                .ok_or_else(|| format!("the search has no {action}"));
        }
        let part = self
            .part(&req.target)
            .ok_or_else(|| format!("no part {}", req.target))?;
        match (part, action) {
            (HeaderPart::Thumb(i), "select") => {
                if let Some(t) = &mut self.thumbs {
                    t.current = i;
                }
                Ok(())
            }
            (HeaderPart::Slot(i), "set_value") => match (&mut self.slots[i].item, &req.param) {
                (HeaderItem::Search(g), Some(Value::Text(text))) => g
                    .input
                    .set_value(text)
                    .then_some(())
                    .ok_or_else(|| "the search refused the text".to_string()),
                _ => Err(format!("{} takes no text", req.target)),
            },
            (HeaderPart::Slot(i), "activate") => match &self.slots[i].item {
                HeaderItem::Action(a) if a.disabled => Err(format!("{} is disabled", req.target)),
                HeaderItem::Search(_) => Err("the search box has no activate".into()),
                _ => Ok(()),
            },
            _ => Err(format!("{} has no {action}", req.target)),
        }
    }
}
