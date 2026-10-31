//! The controls inside a modal: the engine's own widgets (a button, an
//! icon button, a switch, tabs, a select, a slider, a value edited in
//! place) and the pressables a modal draws itself (a variation's tile, a
//! side menu's item, a remove), each under its id. A modal lays them out
//! (one rect per control, in order) and draws around them; this routes
//! the pointer, projects them, runs what an actor asks of them and holds
//! the keyboard focus. Nothing here is visual: every widget draws from
//! its own recipe.

use accord::{Role, SemId, SnapshotBuilder, Value};

use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::Theme;
use crate::widgets::buttons::{Button, IconButton, Press};
use crate::widgets::form::new_field::NewField;
use crate::widgets::form::{Select, Slider, Switch};
use crate::widgets::nav::Tabs;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// How a pressable the modal draws is projected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemRole {
    /// A plain press (`activate`).
    Button,
    /// One choice of a group (`select`; the group is the id's parent).
    Radio,
    /// One tab of a tab list (`select`; the list is the id's parent).
    Tab,
}

/// A pressable a modal draws itself: its name, its pointer state, its
/// role, whether it is the group's choice, and the keyboard focus.
#[derive(Clone, Debug)]
pub struct Item {
    pub label: String,
    pub press: Press,
    pub role: ItemRole,
    pub checked: bool,
}

impl Item {
    pub fn new(label: impl Into<String>, role: ItemRole) -> Self {
        Self {
            label: label.into(),
            press: Press::default(),
            role,
            checked: false,
        }
    }

    /// Hovered or chosen: what its modal lights.
    pub fn lit(&self) -> bool {
        self.press.hovered || self.checked
    }
}

/// One control: the widget and the name a projection gives it.
#[derive(Clone, Debug)]
pub enum Kind {
    Button(Button),
    Icon(IconButton, String),
    Switch(Switch, String),
    Tabs(Tabs, String),
    Select(Select, String),
    Slider(Slider, String),
    Field(NewField, String),
    Item(Item),
}

/// A control under its id.
#[derive(Clone, Debug)]
pub struct Control {
    pub kind: Kind,
    id: Option<SemId>,
    focused: bool,
}

impl Control {
    pub fn new(kind: Kind) -> Self {
        Self {
            kind,
            id: None,
            focused: false,
        }
    }

    /// Under `id`: the widget binds what it draws to it.
    pub fn sem(mut self, id: Option<SemId>) -> Self {
        let Some(id) = id else {
            return self;
        };
        self.kind = match self.kind {
            Kind::Button(w) => Kind::Button(w.sem(id.clone())),
            Kind::Icon(w, l) => Kind::Icon(w.sem(id.clone()), l),
            Kind::Switch(w, l) => Kind::Switch(w.sem(id.clone()), l),
            Kind::Tabs(w, l) => Kind::Tabs(w.sem(id.clone()), l),
            Kind::Select(w, l) => Kind::Select(w.sem(id.clone()), l),
            Kind::Slider(w, l) => Kind::Slider(w.sem(id.clone()), l),
            Kind::Field(w, l) => Kind::Field(w.sem(id.clone()), l),
            item @ Kind::Item(_) => item,
        };
        self.id = Some(id);
        self
    }

    pub fn id(&self) -> Option<&SemId> {
        self.id.as_ref()
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Whether `id` is this control or a node it projects under it (a
    /// tab, an option, the pencil).
    fn owns(&self, id: &SemId) -> bool {
        self.id.as_ref().is_some_and(|own| {
            id == own
                || id
                    .as_str()
                    .strip_prefix(own.as_str())
                    .is_some_and(|rest| rest.starts_with(['[', '.']))
        })
    }

    /// The index of `id` under this control (`<own>[<i>]`).
    fn index_of(&self, id: &SemId) -> Option<usize> {
        let own = self.id.as_ref()?;
        id.as_str()
            .strip_prefix(own.as_str())?
            .strip_prefix('[')?
            .strip_suffix(']')?
            .parse()
            .ok()
    }

    fn set_focus(&mut self, id: Option<&SemId>) {
        let here = id.filter(|id| self.owns(id));
        let tab = here.and_then(|id| self.index_of(id));
        self.focused = here.is_some();
        match &mut self.kind {
            Kind::Button(w) => w.set_focused(self.focused),
            Kind::Icon(w, _) => w.set_focused(self.focused),
            Kind::Switch(w, _) => w.set_focused(self.focused),
            Kind::Select(w, _) => w.set_focused(self.focused),
            Kind::Slider(w, _) => w.set_focused(self.focused),
            Kind::Field(w, _) => w.set_icon_focus(self.focused),
            Kind::Tabs(w, _) => w.set_focused_tab(tab),
            Kind::Item(_) => {}
        }
    }

    /// An open select: it has every event until it closes.
    pub fn is_open(&self) -> bool {
        matches!(&self.kind, Kind::Select(s, _) if s.is_open())
    }

    fn handle_event(&mut self, event: &WidgetEvent, r: Rect, theme: &Theme) -> EventResult {
        match &mut self.kind {
            Kind::Button(w) => w.handle_event(event, r),
            Kind::Icon(w, _) => w.handle_event(event, r),
            Kind::Switch(w, _) => w.handle_event(event, r),
            Kind::Tabs(w, _) => w.handle_event(event, r, theme),
            Kind::Select(w, _) => w.handle_event(event, r, theme),
            Kind::Slider(w, _) => w.handle_event(event, r, theme),
            Kind::Field(w, _) => w.handle_event(event, r, theme),
            Kind::Item(i) => i.press.handle(event, r, false),
        }
    }

    fn render(&self, c: &mut Compositor, r: Rect, theme: &Theme) {
        match &self.kind {
            Kind::Button(w) => w.render(c, r, theme),
            Kind::Icon(w, _) => w.render(c, r, theme),
            Kind::Switch(w, _) => w.render(c, r, theme),
            Kind::Tabs(w, _) => w.render(c, r, theme),
            Kind::Select(w, _) => w.render(c, r, theme),
            Kind::Slider(w, _) => w.render(c, r, theme),
            Kind::Field(w, _) => w.render(c, r, theme),
            Kind::Item(_) => {
                if let Some(id) = &self.id {
                    c.bind(id, [r.x, r.y, r.w, r.h]);
                }
            }
        }
    }

    fn project(&self, b: &mut SnapshotBuilder, parent: &SemId) {
        let Some(id) = &self.id else {
            return;
        };
        let p = Some(parent);
        match &self.kind {
            Kind::Button(w) => {
                w.project(b, id, p);
            }
            Kind::Icon(w, l) => {
                w.project(b, id, l.as_str(), p);
            }
            Kind::Switch(w, l) => {
                w.project(b, id, l.as_str(), p);
            }
            Kind::Tabs(w, l) => {
                w.project(b, id, l.as_str(), p);
            }
            Kind::Select(w, l) => {
                w.project(b, id, l.as_str(), p);
            }
            Kind::Slider(w, l) => {
                w.project(b, id, l.as_str(), p);
            }
            Kind::Field(w, l) => w.project(b, id, l, p),
            Kind::Item(i) => project_item(b, id, i, parent),
        }
        if self.focused {
            b.focus(id);
        }
    }

    /// `action` on `target`, one of this control's nodes: `Ok(true)` when
    /// it changed something, `Err` when the control has no such action.
    fn run(&mut self, target: &SemId, action: &str, param: Option<&Value>) -> Result<bool, String> {
        let at = self.index_of(target);
        let own = self.id.as_ref() == Some(target);
        let refused = || format!("{target} has no {action}");
        match (&mut self.kind, action, param) {
            (Kind::Button(_) | Kind::Icon(..), "activate", None) if own => Ok(false),
            (Kind::Item(i), "activate", None) if own && i.role == ItemRole::Button => Ok(false),
            (Kind::Item(i), "select", None) if own && i.role != ItemRole::Button => {
                Ok(!std::mem::replace(&mut i.checked, true))
            }
            (Kind::Switch(w, _), "toggle", None) if own => {
                w.toggle();
                Ok(true)
            }
            (Kind::Tabs(w, _), "select", None) => match at {
                Some(i) if i < w.labels.len() => Ok(std::mem::replace(&mut w.active, i) != i),
                _ => Err(refused()),
            },
            (Kind::Select(w, _), "open", None) if own => {
                w.open();
                Ok(true)
            }
            (Kind::Select(w, _), "close", None) if own => {
                w.close();
                Ok(true)
            }
            (Kind::Select(w, _), "select", Some(Value::Text(label))) if own => {
                let found = w.options.iter().any(|o| o == label);
                found.then(|| w.select_label(label)).ok_or_else(refused)
            }
            (Kind::Select(w, _), "select", None) => match at {
                Some(i) if i < w.options.len() => Ok(w.select_index(i)),
                _ => Err(refused()),
            },
            (Kind::Slider(w, _), "set_value", Some(v)) if own => {
                let v = match v {
                    Value::Num(n) => *n as f32,
                    Value::Int(n) => *n as f32,
                    _ => return Err(refused()),
                };
                let before = w.value();
                w.set_value(v);
                Ok(w.value() != before)
            }
            (Kind::Slider(w, _), "increment", None) if own => Ok(w.increment()),
            (Kind::Slider(w, _), "decrement", None) if own => Ok(w.decrement()),
            (Kind::Field(w, _), "set_value", Some(Value::Text(t))) if own => Ok(w.set_value(t)),
            (Kind::Field(w, _), "activate", None) if w.icon_id().as_ref() == Some(target) => {
                Ok(if w.is_editing() { w.save() } else { w.edit() })
            }
            _ => Err(refused()),
        }
    }
}

/// A pressable the modal draws, as its role projects it.
fn project_item(b: &mut SnapshotBuilder, id: &SemId, item: &Item, parent: &SemId) {
    let role = match item.role {
        ItemRole::Button => Role::Button,
        ItemRole::Radio => Role::Radio,
        ItemRole::Tab => Role::Tab,
    };
    let group = id.parent_path().unwrap_or_else(|| parent.clone());
    let mut node = b.node(id, role, item.label.as_str());
    match item.role {
        ItemRole::Button => {
            node.parent(parent);
            node.action("activate");
        }
        ItemRole::Radio | ItemRole::Tab => {
            node.parent(&group);
            if item.role == ItemRole::Radio {
                node.state("checked", item.checked);
            }
            node.action("select").reads(format!("{group}:selected"));
        }
    }
}

/// A modal's controls in the order it lays them out.
#[derive(Clone, Debug, Default)]
pub struct Controls(pub Vec<Control>);

impl Controls {
    pub fn push(&mut self, kind: Kind, id: Option<SemId>) {
        self.0.push(Control::new(kind).sem(id));
    }

    /// The pointer over the controls at `rects`: an open select first and
    /// alone; then each control. The index of the one that was clicked.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        rects: &[Rect],
        theme: &Theme,
    ) -> (Option<usize>, EventResult) {
        if let Some((i, open)) = self.0.iter_mut().enumerate().find(|(_, c)| c.is_open()) {
            let Some(r) = rects.get(i) else {
                return (None, EventResult::IGNORED);
            };
            let mut res = open.handle_event(event, *r, theme);
            res.handled = true;
            return (res.clicked.then_some(i), res);
        }
        let mut out = EventResult::IGNORED;
        let mut clicked = None;
        for (i, (control, r)) in self.0.iter_mut().zip(rects).enumerate() {
            let res = control.handle_event(event, *r, theme);
            if res.clicked && clicked.is_none() {
                clicked = Some(i);
            }
            out = out.merge(res);
        }
        (clicked, out)
    }

    pub fn render(&self, c: &mut Compositor, rects: &[Rect], theme: &Theme) {
        for (control, r) in self.0.iter().zip(rects) {
            control.render(c, *r, theme);
        }
    }

    /// The open select's list, on `layer` above the page.
    pub fn render_overlay(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        rects: &[Rect],
        theme: &Theme,
    ) {
        for (control, r) in self.0.iter().zip(rects) {
            if let Kind::Select(s, _) = &control.kind
                && s.is_open()
            {
                s.render_dropdown(c, layer, *r, theme);
            }
        }
    }

    pub fn project(&self, b: &mut SnapshotBuilder, parent: &SemId) {
        for control in &self.0 {
            control.project(b, parent);
        }
    }

    /// `action` on `target`: `None` when no control owns it, else which
    /// control and whether it changed or was refused.
    pub fn run(
        &mut self,
        target: &SemId,
        action: &str,
        param: Option<&Value>,
    ) -> Option<(usize, Result<bool, String>)> {
        let (i, control) = self
            .0
            .iter_mut()
            .enumerate()
            .find(|(_, c)| c.owns(target))?;
        Some((i, control.run(target, action, param)))
    }

    /// The keyboard focus on `id` (or off every control): whether one of
    /// them took it.
    pub fn focus(&mut self, id: Option<&SemId>) -> bool {
        let mut took = false;
        for control in &mut self.0 {
            control.set_focus(id);
            took |= control.focused;
        }
        took
    }

    /// Closes an open select: `true` when one was.
    pub fn close(&mut self) -> bool {
        let mut closed = false;
        for control in &mut self.0 {
            if let Kind::Select(s, _) = &mut control.kind
                && s.is_open()
            {
                s.close();
                closed = true;
            }
        }
        closed
    }

    pub fn any_open(&self) -> bool {
        self.0.iter().any(Control::is_open)
    }

    /// The switches' knobs travel: `true` while one moves.
    pub fn tick(&mut self, dt: f32) -> bool {
        let mut moving = false;
        for control in &mut self.0 {
            if let Kind::Switch(s, _) = &mut control.kind {
                moving |= s.tick(dt);
            }
        }
        moving
    }

    /// The pressable at `i`, when it is one the modal draws.
    pub fn item(&self, i: usize) -> Option<&Item> {
        match self.0.get(i).map(|c| &c.kind) {
            Some(Kind::Item(item)) => Some(item),
            _ => None,
        }
    }

    /// Marks the pressables `range` as a group whose choice is `chosen`.
    pub fn choose(&mut self, range: std::ops::Range<usize>, chosen: Option<usize>) {
        for (k, control) in self.0[range].iter_mut().enumerate() {
            if let Kind::Item(item) = &mut control.kind {
                item.checked = Some(k) == chosen;
            }
        }
    }
}
