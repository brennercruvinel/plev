//! the closed component vocabulary, declared as data.
//!
//! every `Component` is a widget that exists in `rust/engine/src/widgets` or in the
//! engine builder, and its `Spec` says what the IR may do with it: the
//! role it takes in the semantic contract, whether it holds children,
//! how it is named, the state it can show and the events it can send.
//! the role and the names of state and events follow the `project` of the
//! comps widget where it has one (`Button` sends `activate`, `TextField`
//! shows `text` and sends `set_value`), so a materialized node declares
//! what the widget already declares. a new component needs the widget in
//! comps first and an ADR.

use accord::{Role, ValueKind};
use serde::{Deserialize, Serialize};

use crate::ir::Bind;

/// the kind of a chart, one per chart of `engine::widgets::charts`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChartKind {
    Line,
    Bar,
    Area,
    Donut,
}

impl ChartKind {
    pub const ALL: [ChartKind; 4] = [
        ChartKind::Line,
        ChartKind::Bar,
        ChartKind::Area,
        ChartKind::Donut,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            ChartKind::Line => "line",
            ChartKind::Bar => "bar",
            ChartKind::Area => "area",
            ChartKind::Donut => "donut",
        }
    }
}

/// what a node is drawn as. `Chart` writes as `{"chart": "line"}`, every
/// other component as its name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Component {
    /// `engine::widgets::buttons::Button` (and `SubmitButton`, whose `busy` it shows).
    Button,
    /// `engine::widgets::buttons::IconButton`.
    IconButton,
    /// `engine::widgets::form::TextField`.
    TextField,
    /// `engine::widgets::form::Checkbox`.
    Checkbox,
    /// `engine::widgets::form::Switch`.
    Switch,
    /// `engine::widgets::form::Slider`.
    Slider,
    /// `engine::widgets::form::Select`.
    Select,
    /// `engine::widgets::nav::Tabs`.
    Tabs,
    /// `engine::widgets::content::VirtualList`.
    List,
    /// `engine::widgets::content::Table`.
    Table,
    /// `engine::widgets::content::Card`.
    Card,
    /// `engine::widgets::content::Stat`.
    Stat,
    /// `engine::widgets::charts`: line, bar, stacked area, donut.
    Chart(ChartKind),
    /// `engine::widgets::content::Separator`.
    Separator,
    /// `engine::widgets::feedback::Modal`.
    Dialog,
    /// `engine::widgets::content::EmptyState`.
    EmptyState,
    /// `engine::widgets::feedback::ProgressBar`.
    Progress,
    /// `engine::widgets::buttons::Badge`.
    Badge,
    /// `engine::widgets::buttons::Chip`.
    Chip,
    /// a text run of the engine builder (`engine::compose::builder::text`).
    Text,
    /// an image of the engine builder (`engine::compose::builder::image`).
    Image,
    /// `engine::widgets::content::CodeBlock`: text in the mono step, a row per line,
    /// each line with its kind when the app says it (task-97).
    Code,
}

/// the kind of one line a `code` shows, by the name an app's `kinds`
/// state holds for it: a diff's context, a line added, a line removed, a
/// hunk's header. the IR never holds one (the verifier checks that
/// `kinds` is a list, not its values); the app projects the names, the
/// materializer draws each by the theme's tokens, and a name it does not
/// know is a context line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LineKind {
    #[default]
    Context,
    Added,
    Removed,
    Header,
}

impl LineKind {
    pub const ALL: [LineKind; 4] = [
        LineKind::Context,
        LineKind::Added,
        LineKind::Removed,
        LineKind::Header,
    ];

    /// the name an app writes in a `kinds` list.
    pub fn as_str(self) -> &'static str {
        match self {
            LineKind::Context => "context",
            LineKind::Added => "added",
            LineKind::Removed => "removed",
            LineKind::Header => "header",
        }
    }

    /// the kind `name` names, if any.
    pub fn parse(name: &str) -> Option<LineKind> {
        LineKind::ALL.into_iter().find(|k| k.as_str() == name)
    }
}

/// how a node gets its accessible name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Named {
    /// it needs a label that is not blank.
    Always,
    /// a label, or the named state bound (a text shows its `text`).
    OrBound(&'static str),
    /// a label is welcome, not needed (a container).
    Optional,
    /// it has no role, so nothing can name it (a separator).
    Never,
}

/// a piece of state a component can show, and the kinds it accepts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateProp {
    pub name: &'static str,
    pub kinds: &'static [ValueKind],
}

/// an event a component can send, and the kind of the value it sends with
/// it (`None` for a plain activation).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub name: &'static str,
    pub param: Option<ValueKind>,
}

/// what the IR may do with a component.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    /// the role it takes in the semantic contract; `None` for a
    /// decorative component (a separator), which is never a control.
    pub role: Option<Role>,
    /// it holds children (and may repeat a template).
    pub container: bool,
    pub named: Named,
    pub state: &'static [StateProp],
    pub events: &'static [Event],
}

impl Spec {
    pub fn state_prop(&self, name: &str) -> Option<&'static StateProp> {
        self.state.iter().find(|p| p.name == name)
    }

    pub fn event(&self, name: &str) -> Option<&'static Event> {
        self.events.iter().find(|e| e.name == name)
    }
}

const fn prop(name: &'static str, kinds: &'static [ValueKind]) -> StateProp {
    StateProp { name, kinds }
}

const fn event(name: &'static str, param: Option<ValueKind>) -> Event {
    Event { name, param }
}

const fn spec(
    role: Option<Role>,
    container: bool,
    named: Named,
    state: &'static [StateProp],
    events: &'static [Event],
) -> Spec {
    Spec {
        role,
        container,
        named,
        state,
        events,
    }
}

use Named::{Always, Never, Optional, OrBound};
use ValueKind::{Bool, Int, List, Num, Set, Text};

const ACTIVATE: Event = event("activate", None);
const TOGGLE: Event = event("toggle", None);

/// a node without a component: a layout container (the engine builder's
/// `div`), a `Group` in the contract. a toolbar is one of these holding
/// buttons.
pub const CONTAINER: Spec = spec(Some(Role::Group), true, Optional, &[], &[]);

const BUTTON: Spec = spec(
    Some(Role::Button),
    false,
    Always,
    &[prop("busy", &[Bool])],
    &[ACTIVATE],
);
const ICON_BUTTON: Spec = spec(
    Some(Role::Button),
    false,
    Always,
    &[prop("active", &[Bool])],
    &[ACTIVATE],
);
const TEXT_FIELD: Spec = spec(
    Some(Role::TextInput),
    false,
    Always,
    &[prop("text", &[Text])],
    &[event("set_value", Some(Text)), event("submit", None)],
);
const CHECKBOX: Spec = spec(
    Some(Role::Checkbox),
    false,
    Always,
    &[prop("checked", &[Bool])],
    &[TOGGLE],
);
const SWITCH: Spec = spec(
    Some(Role::Toggle),
    false,
    Always,
    &[prop("on", &[Bool])],
    &[TOGGLE],
);
const SLIDER: Spec = spec(
    Some(Role::Slider),
    false,
    Always,
    &[prop("value", &[Num, Int])],
    &[event("set_value", Some(Num))],
);
const SELECT: Spec = spec(
    Some(Role::Select),
    false,
    Always,
    &[prop("selected", &[Text]), prop("options", &[List, Set])],
    &[event("select", Some(Text))],
);
const TABS: Spec = spec(
    Some(Role::TabList),
    false,
    Always,
    &[prop("selected", &[Text]), prop("options", &[List, Set])],
    &[event("select", Some(Text))],
);
const LIST: Spec = spec(Some(Role::List), true, Optional, &[], &[]);
const TABLE: Spec = spec(
    Some(Role::Table),
    false,
    Optional,
    &[prop("rows", &[List, Set])],
    &[],
);
const CARD: Spec = spec(Some(Role::Group), true, Optional, &[], &[]);
const STAT: Spec = spec(
    Some(Role::Label),
    false,
    Always,
    &[prop("value", &[Text, Int, Num])],
    &[],
);
const CHART: Spec = spec(
    Some(Role::Image),
    false,
    Always,
    &[prop("data", &[List])],
    &[],
);
const SEPARATOR: Spec = spec(None, false, Never, &[], &[]);
const DIALOG: Spec = spec(
    Some(Role::Dialog),
    true,
    Always,
    &[prop("open", &[Bool])],
    &[event("confirm", None), event("cancel", None)],
);
const EMPTY_STATE: Spec = spec(Some(Role::Status), false, Always, &[], &[]);
const PROGRESS: Spec = spec(
    Some(Role::ProgressBar),
    false,
    Always,
    &[prop("value", &[Num, Int])],
    &[],
);
const BADGE: Spec = spec(
    Some(Role::Label),
    false,
    OrBound("text"),
    &[prop("text", &[Text, Int])],
    &[],
);
const CHIP: Spec = spec(
    Some(Role::Label),
    false,
    Always,
    &[prop("selected", &[Bool])],
    &[TOGGLE, ACTIVATE],
);
const TEXT: Spec = spec(
    Some(Role::Label),
    false,
    OrBound("text"),
    &[prop("text", &[Text, Int, Num])],
    &[],
);
const IMAGE: Spec = spec(
    Some(Role::Image),
    false,
    Always,
    &[prop("source", &[Text])],
    &[],
);
/// a code block: a region named by its caption, as a scrollable block of
/// text is to a screen reader. `lines` is the code, a list of lines or one
/// text; `kinds` is the kind of each line, `LineKind` by name.
const CODE: Spec = spec(
    Some(Role::Region),
    false,
    Always,
    &[prop("lines", &[List, Text]), prop("kinds", &[List])],
    &[],
);

impl Component {
    /// every component, a chart once per kind.
    pub const ALL: [Component; 25] = [
        Component::Button,
        Component::IconButton,
        Component::TextField,
        Component::Checkbox,
        Component::Switch,
        Component::Slider,
        Component::Select,
        Component::Tabs,
        Component::List,
        Component::Table,
        Component::Card,
        Component::Stat,
        Component::Chart(ChartKind::Line),
        Component::Chart(ChartKind::Bar),
        Component::Chart(ChartKind::Area),
        Component::Chart(ChartKind::Donut),
        Component::Separator,
        Component::Dialog,
        Component::EmptyState,
        Component::Progress,
        Component::Badge,
        Component::Chip,
        Component::Text,
        Component::Image,
        Component::Code,
    ];

    /// the name in the JSON form (`chart` for every chart).
    pub fn name(self) -> &'static str {
        match self {
            Component::Button => "button",
            Component::IconButton => "icon_button",
            Component::TextField => "text_field",
            Component::Checkbox => "checkbox",
            Component::Switch => "switch",
            Component::Slider => "slider",
            Component::Select => "select",
            Component::Tabs => "tabs",
            Component::List => "list",
            Component::Table => "table",
            Component::Card => "card",
            Component::Stat => "stat",
            Component::Chart(_) => "chart",
            Component::Separator => "separator",
            Component::Dialog => "dialog",
            Component::EmptyState => "empty_state",
            Component::Progress => "progress",
            Component::Badge => "badge",
            Component::Chip => "chip",
            Component::Text => "text",
            Component::Image => "image",
            Component::Code => "code",
        }
    }

    pub fn spec(self) -> &'static Spec {
        match self {
            Component::Button => &BUTTON,
            Component::IconButton => &ICON_BUTTON,
            Component::TextField => &TEXT_FIELD,
            Component::Checkbox => &CHECKBOX,
            Component::Switch => &SWITCH,
            Component::Slider => &SLIDER,
            Component::Select => &SELECT,
            Component::Tabs => &TABS,
            Component::List => &LIST,
            Component::Table => &TABLE,
            Component::Card => &CARD,
            Component::Stat => &STAT,
            Component::Chart(_) => &CHART,
            Component::Separator => &SEPARATOR,
            Component::Dialog => &DIALOG,
            Component::EmptyState => &EMPTY_STATE,
            Component::Progress => &PROGRESS,
            Component::Badge => &BADGE,
            Component::Chip => &CHIP,
            Component::Text => &TEXT,
            Component::Image => &IMAGE,
            Component::Code => &CODE,
        }
    }
}

/// the spec of a node: its component's, or a layout container's.
pub fn spec_of(component: Option<Component>) -> &'static Spec {
    component.map_or(&CONTAINER, Component::spec)
}

/// the role a node takes, which for a chip follows what it binds, as the
/// comps chip projects it: a toggle when it shows `selected` or sends
/// `toggle`, a button when it sends `activate`, a label otherwise.
pub fn role_of(component: Option<Component>, bind: &Bind) -> Option<Role> {
    if component != Some(Component::Chip) {
        return spec_of(component).role;
    }
    if bind.state.contains_key("selected") || bind.actions.contains_key("toggle") {
        Some(Role::Toggle)
    } else if bind.actions.contains_key("activate") {
        Some(Role::Button)
    } else {
        Some(Role::Label)
    }
}

/// a control: a node a person or an agent operates directly, a stop of
/// the focus order. every interactive role, and a tab strip.
pub fn is_control(role: Option<Role>) -> bool {
    role.is_some_and(|r| r.is_interactive() || r == Role::TabList)
}

/// every state name and every event name some component accepts, in
/// order and once each: what the schema lets a binding name.
pub fn names() -> (Vec<&'static str>, Vec<&'static str>) {
    let mut state = Vec::new();
    let mut events = Vec::new();
    for c in Component::ALL {
        for p in c.spec().state {
            if !state.contains(&p.name) {
                state.push(p.name);
            }
        }
        for e in c.spec().events {
            if !events.contains(&e.name) {
                events.push(e.name);
            }
        }
    }
    (state, events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_component_writes_as_its_name_and_a_chart_with_its_kind() {
        for c in Component::ALL {
            let json = serde_json::to_value(c).unwrap();
            match c {
                Component::Chart(kind) => {
                    assert_eq!(json, serde_json::json!({ "chart": kind.as_str() }));
                }
                _ => assert_eq!(json, serde_json::json!(c.name())),
            }
            assert_eq!(serde_json::from_value::<Component>(json).unwrap(), c);
        }
        assert!(serde_json::from_str::<Component>(r#""video""#).is_err());
        assert!(serde_json::from_str::<Component>(r#""toolbar""#).is_err());
    }

    #[test]
    fn every_interactive_component_is_named_and_sends_something() {
        for c in Component::ALL {
            let spec = c.spec();
            if spec.role.is_some_and(Role::is_interactive) {
                assert_eq!(spec.named, Named::Always, "{c:?}");
                assert!(!spec.events.is_empty(), "{c:?}");
                assert!(!spec.container, "{c:?}: a control holds no children");
            }
        }
    }

    #[test]
    fn a_code_is_a_named_region_that_shows_lines_and_their_kinds() {
        let spec = Component::Code.spec();
        assert_eq!(spec.role, Some(Role::Region));
        assert_eq!(spec.named, Named::Always);
        assert!(!spec.container && spec.events.is_empty());
        let kinds = |name: &str| spec.state_prop(name).map(|p| p.kinds);
        assert_eq!(kinds("lines"), Some(&[List, Text][..]));
        assert_eq!(kinds("kinds"), Some(&[List][..]));
        assert!(!is_control(role_of(
            Some(Component::Code),
            &Bind::default()
        )));
        for k in LineKind::ALL {
            assert_eq!(LineKind::parse(k.as_str()), Some(k));
        }
        assert_eq!(LineKind::parse("+"), None);
        assert_eq!(LineKind::default(), LineKind::Context);
        // the vocabulary grows at its end: every name of before keeps its
        // place in the schema's lists
        assert_eq!(Component::ALL.last(), Some(&Component::Code));
        let (state, _) = names();
        assert_eq!(state[state.len() - 2..], ["lines", "kinds"]);
    }

    #[test]
    fn a_chip_takes_its_role_from_what_it_binds() {
        let mut bind = Bind::default();
        assert_eq!(role_of(Some(Component::Chip), &bind), Some(Role::Label));
        bind.actions.insert(
            "activate".to_owned(),
            crate::ir::CapabilityRef::parse("todo.filter[all]", "select").unwrap(),
        );
        assert_eq!(role_of(Some(Component::Chip), &bind), Some(Role::Button));
        bind.state
            .insert("selected".to_owned(), "a:on".parse().unwrap());
        assert_eq!(role_of(Some(Component::Chip), &bind), Some(Role::Toggle));
        assert_eq!(role_of(None, &bind), Some(Role::Group));
        assert_eq!(role_of(Some(Component::Separator), &bind), None);
    }
}
