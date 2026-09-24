//! the retained comps widget behind each node, indexed by its `SemId`:
//! built once, kept across frames (hover, focus, caret, a knob in flight),
//! and fed the state the app holds.

use comps::action::{Button, ButtonVariant, Chip, IconButton, SubmitButton};
use comps::content::{Column, Table};
use comps::form::{Checkbox, Select, Slider, Switch, TextField};
use comps::nav::Tabs;
use comps::{Rect, Wake};
use engine::theme::Theme;
use sem::{Snapshot, Value};
use uir::{CapabilityRef, Component};

use crate::expand::Node;
use crate::token;

/// the icon an `icon_button` draws: the IR v1 has no icon field (the
/// icon table is the comps's), so every icon button shows this one and is
/// named by its label.
pub(crate) const ICON: &str = "circle";

/// a dialog's two buttons, which send its `cancel` and `confirm`.
#[derive(Clone, Debug)]
pub(crate) struct DialogButtons {
    pub cancel: Button,
    pub confirm: Button,
}

/// the widget of one node. the static components (a container, a card, a
/// text, a stat, a badge...) keep nothing between frames: they are drawn
/// from the node and the snapshot each time.
#[derive(Debug)]
pub(crate) enum Widget {
    Static,
    Button(Button),
    Submit(SubmitButton),
    IconButton(IconButton),
    TextField(Box<TextField>),
    Checkbox(Checkbox),
    Switch(Switch),
    Slider(Slider),
    Select(Box<Select>),
    Tabs(Tabs),
    Chip(Chip),
    /// built at the first frame: its columns read the theme.
    Table(Option<Box<Table>>),
    Dialog(Box<DialogButtons>),
}

/// the declaration of `cap` in `shown`, with the node that holds it.
pub(crate) fn declared<'a>(
    shown: &'a Snapshot,
    cap: &CapabilityRef,
) -> Option<(&'a sem::Node, &'a sem::ActionDecl)> {
    let node = shown.get(&cap.target)?;
    let decl = node.actions.iter().find(|a| a.name == cap.action)?;
    Some((node, decl))
}

/// whether `event` of `node` can run now: its capability is declared
/// and enabled in `shown`. an event the node does not bind counts as
/// enabled (it is never sent).
pub(crate) fn enabled(node: &Node, event: &str, shown: &Snapshot) -> bool {
    match node.bind.actions.get(event) {
        None => true,
        Some(cap) => declared(shown, cap).is_some_and(|(_, d)| d.enabled),
    }
}

/// the text of a value the way a widget shows it.
pub(crate) fn text_of(value: &Value) -> String {
    match value {
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Num(n) => n.to_string(),
        Value::Text(t) => t.clone(),
        Value::List(items) => items.join(", "),
        Value::Set(items) => items.iter().cloned().collect::<Vec<_>>().join(", "),
    }
}

/// the items of a `List` or a `Set`, in order.
pub(crate) fn list_of(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::List(items)) => items.clone(),
        Some(Value::Set(items)) => items.iter().cloned().collect(),
        _ => Vec::new(),
    }
}

fn number(value: Option<&Value>) -> Option<f32> {
    match value? {
        Value::Num(n) => Some(*n as f32),
        Value::Int(i) => Some(*i as f32),
        _ => None,
    }
}

fn flag(value: Option<&Value>) -> bool {
    matches!(value, Some(Value::Bool(true)))
}

/// the label of the capability behind `event`, or `fallback`.
fn cap_label(node: &Node, event: &str, shown: &Snapshot, fallback: &str) -> String {
    node.bind
        .actions
        .get(event)
        .and_then(|cap| declared(shown, cap))
        .and_then(|(_, d)| d.label.clone())
        .unwrap_or_else(|| fallback.to_owned())
}

impl Widget {
    /// the widget of `node`; the ones that bind themselves get its id.
    pub fn new(node: &Node) -> Widget {
        let id = node.id.clone();
        let label = node.label.clone().unwrap_or_default();
        match node.component {
            Some(Component::Button) if node.bind.state.contains_key("busy") => {
                Widget::Submit(SubmitButton::new().sem(id))
            }
            Some(Component::Button) => Widget::Button(Button::new(label).sem(id)),
            Some(Component::IconButton) => Widget::IconButton(IconButton::new(ICON).sem(id)),
            Some(Component::TextField) => {
                Widget::TextField(Box::new(TextField::new(label).sem(id)))
            }
            Some(Component::Checkbox) => {
                Widget::Checkbox(Checkbox::new(false).label(label).sem(id))
            }
            Some(Component::Switch) => Widget::Switch(Switch::new(false).sem(id)),
            Some(Component::Slider) => Widget::Slider(Slider::new(0.0, 1.0, 0.0).sem(id)),
            Some(Component::Select) => {
                Widget::Select(Box::new(Select::new(Vec::<String>::new(), 0)))
            }
            Some(Component::Tabs) => Widget::Tabs(Tabs::new(Vec::<String>::new())),
            Some(Component::Chip) => Widget::Chip(Chip::new(label).sem(id)),
            Some(Component::Table) => Widget::Table(None),
            Some(Component::Dialog) => Widget::Dialog(Box::new(DialogButtons {
                cancel: Button::new("cancel"),
                confirm: Button::new("confirm").variant(ButtonVariant::Secondary),
            })),
            _ => Widget::Static,
        }
    }

    /// feeds the widget what the app holds in `shown`: the bound state,
    /// and whether its capabilities can run (a capability disabled or
    /// gone leaves the control disabled: a stale surface fails safe).
    pub fn sync(&mut self, node: &Node, shown: &Snapshot) {
        let value = |prop: &str| node.value(prop, shown);
        match self {
            Widget::Static => {}
            Widget::Button(b) => {
                b.label = node.label.clone().unwrap_or_default();
                b.disabled = !enabled(node, "activate", shown);
            }
            Widget::Submit(b) => {
                b.active = flag(value("busy"));
                b.disabled = !enabled(node, "activate", shown);
            }
            Widget::IconButton(b) => {
                b.active = flag(value("active"));
                b.disabled = !enabled(node, "activate", shown);
            }
            Widget::TextField(f) => {
                let text = value("text").map(text_of).unwrap_or_default();
                if f.text() != text {
                    f.set_text(&text);
                }
                f.disabled = !enabled(node, "set_value", shown);
            }
            Widget::Checkbox(c) => {
                c.checked = flag(value("checked"));
                c.disabled = !enabled(node, "toggle", shown);
            }
            Widget::Switch(s) => {
                if s.on != flag(value("on")) {
                    s.toggle();
                }
                s.disabled = !enabled(node, "toggle", shown);
            }
            Widget::Slider(s) => sync_slider(s, node, shown),
            Widget::Select(s) => {
                let options = list_of(value("options"));
                if s.options != options {
                    s.options = options;
                }
                let chosen = value("selected").map(text_of);
                if let Some(i) = chosen.and_then(|c| s.options.iter().position(|o| *o == c)) {
                    s.selected = i;
                }
                s.disabled = !enabled(node, "select", shown);
            }
            Widget::Tabs(t) => {
                let labels = list_of(value("options"));
                if t.labels != labels {
                    t.labels = labels;
                }
                let chosen = value("selected").map(text_of);
                if let Some(i) = chosen.and_then(|c| t.labels.iter().position(|o| *o == c)) {
                    t.active = i;
                }
            }
            Widget::Chip(c) => {
                c.label = node.label.clone().unwrap_or_default();
                c.selected = flag(value("selected"));
                c.interactive = node.bind.actions.contains_key("activate");
                c.selectable = node.bind.actions.contains_key("toggle")
                    || node.bind.state.contains_key("selected");
            }
            Widget::Table(table) => {
                if let Some(t) = table {
                    t.set_row_count(list_of(value("rows")).len());
                }
            }
            Widget::Dialog(d) => {
                d.cancel.label = cap_label(node, "cancel", shown, "cancel");
                d.confirm.label = cap_label(node, "confirm", shown, "confirm");
                let destructive = node
                    .bind
                    .actions
                    .get("confirm")
                    .and_then(|cap| declared(shown, cap))
                    .is_some_and(|(_, d)| d.policy.destructive);
                d.confirm.variant = if destructive {
                    ButtonVariant::Warning
                } else {
                    ButtonVariant::Secondary
                };
                d.cancel.disabled = !enabled(node, "cancel", shown);
                d.confirm.disabled = !enabled(node, "confirm", shown);
            }
        }
    }

    /// what depends on the theme: the button's step of its ladder, the
    /// table's columns. called before each layout.
    pub fn prepare(&mut self, node: &Node, shown: &Snapshot, theme: &Theme) {
        match self {
            Widget::Button(b) => {
                if let Some(size) = node.props.control_size {
                    b.size = token::button_size(size, theme);
                }
            }
            Widget::Table(table @ None) => {
                let title = node.label.clone().unwrap_or_default();
                let mut t = Table::new(vec![Column::new(title, theme)], theme);
                t.set_row_count(list_of(node.value("rows", shown)).len());
                *table = Some(Box::new(t));
            }
            _ => {}
        }
    }

    /// an open dropdown takes every pointer event until it closes.
    pub fn is_open(&self) -> bool {
        matches!(self, Widget::Select(s) if s.is_open())
    }

    /// motion the widget runs on its own: the caret, a knob, a scrollbar
    /// fading.
    pub fn wake(&mut self, dt: f32) -> Wake {
        match self {
            Widget::TextField(f) => f.wake(dt),
            Widget::Switch(s) => Wake::moving(s.tick(dt)),
            Widget::Table(Some(t)) => Wake::moving(t.rows.tick(dt)),
            _ => Wake::idle(),
        }
    }
}

/// the slider's range and step come from the node its `set_value` acts
/// on (`Constraints::range`, `step`), its value from the binding.
fn sync_slider(s: &mut Slider, node: &Node, shown: &Snapshot) {
    let target = node
        .bind
        .actions
        .get("set_value")
        .and_then(|cap| declared(shown, cap))
        .map(|(n, _)| &n.constraints);
    if let Some((min, max)) = target.and_then(|c| c.range) {
        s.min = min as f32;
        s.max = max as f32;
    }
    s.step = target.and_then(|c| c.step).map(|v| v as f32);
    if let Some(v) = number(node.value("value", shown)) {
        s.set_value(v);
    }
    s.disabled = !enabled(node, "set_value", shown);
}

/// where a dialog draws its buttons: the bottom row inside its padding,
/// right aligned, cancel then confirm.
pub(crate) fn dialog_buttons(d: &DialogButtons, rect: Rect, theme: &Theme) -> (Rect, Rect) {
    let pad = theme.spacing.lg;
    let gap = theme.spacing.sm;
    let (cw, ch) = d.cancel.preferred_size(theme);
    let (fw, fh) = d.confirm.preferred_size(theme);
    let y = rect.y + rect.h - pad - ch.max(fh);
    let confirm = Rect::new(rect.x + rect.w - pad - fw, y, fw, fh);
    let cancel = Rect::new(confirm.x - gap - cw, y, cw, ch);
    (cancel, confirm)
}
