//! The component pages, projected (task-75 phase 2b): a page whose
//! widgets project themselves in comps (task-59) declares them under a
//! region of its own, so a screen reader hears it and an agent reads it
//! through the port in the one projection of the view; what either asks
//! runs through the gallery's one dispatcher.
//!
//! A page is a [`Component`]: its widgets carry the ids the page gave them
//! (`.sem(id)`, so each draw binds its rect beside the scene), it projects
//! them under its root, it runs a request against its own state and says
//! what the view does after (a toast, a modal: the view owns them), and it
//! takes the focus a reader asks for. The table below names the panes that
//! are one, one line each; `page_semantics`, `act`, `assist_focus` and the
//! Escape reach a component page through it and nowhere else.
//!
//! A person's click stays in the widget, as before: a comps widget changes
//! its own state in its event (a checkbox checks, a tab selects, a drag
//! moves the knob), and the projection reads that state, so it follows.
//! Every action a widget declares reads what it changes, so a request
//! decided before a person's click comes back stale. What a page does on
//! top of a click (the modal of the Dialog page, the toast of the Toast
//! page, the echo of a crumb) is one method, called by the click and by
//! [`Component::run`] alike: the person, a reader and an agent end in the
//! same state.

use accord::{
    Execution, Operable, Outcome, Receipt, Request, RequestId, Role, SemErrors, SemId, Semantics,
    Snapshot, SnapshotBuilder,
};
use engine::styling::theme::Intent;
use engine::widgets::overlay::sheet::Edge;

use super::super::apps::todo::TodoPage;
use super::super::components::toast::ToastPage;
use super::super::pages::panes::Panes;
use super::super::{Pane, ShowcaseView};

/// What the view does once a component page ran a request: the page
/// asks, and the view owns the toasts and the overlays.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::views) enum After {
    /// Nothing beyond the page's own state.
    #[default]
    Nothing,
    /// A toast of this intent, with the Toast page's message for it.
    Toast(Intent),
    /// The Dialog page's modal: the destructive one or the plain one.
    Modal { destructive: bool },
    // task-104 front B, overlays (#155)
    /// The Sheet page's sheet, held to this edge of the window.
    Sheet(Edge),
    /// The Drawer page's drawer.
    Drawer,
    /// The Context Menu page's menu, at the middle of its area.
    ContextMenu,
}

/// A component page as the view's projection and the gallery's
/// dispatcher see it.
pub(in crate::views) trait Component {
    /// The root of the page's ids; `None` when they did not parse (the
    /// page's test proves they do), and the page projects nothing.
    fn root(&self) -> Option<&SemId>;

    /// The page's widgets into `b`, each with `root` as its parent or an
    /// ancestor, and the focus the page holds.
    fn project(&self, b: &mut SnapshotBuilder, root: &SemId);

    /// Runs `req`, which the dispatcher accepted against the projection:
    /// what the view does after, or why it could not.
    fn run(&mut self, req: &Request) -> Result<After, String>;

    /// Keyboard focus from a screen reader to `id`: the page takes it (a
    /// widget of its own) or takes it from its widgets (another node of
    /// the page). `false` for an id that is not the page's.
    fn focus(&mut self, id: &SemId) -> bool;

    /// Escape, on a page that draws no focus of its own for what a reader
    /// focused: the focus goes. `false` when there was none.
    fn blur(&mut self) -> bool {
        false
    }
}

/// Whether `id` is `root` or under it (`root.x`, `root[k]`).
pub(in crate::views) fn under(root: &SemId, id: &SemId) -> bool {
    id.as_str()
        .strip_prefix(root.as_str())
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(['.', '[']))
}

/// The table: one line per pane whose page is a [`Component`]. The two
/// lookups it writes are the only place a component page is named.
macro_rules! components {
    ($($pane:ident => $field:ident),* $(,)?) => {
        impl Panes {
            /// The page of `pane`, when it is a component page.
            pub(in crate::views) fn component(&self, pane: Pane) -> Option<&dyn Component> {
                match pane {
                    $(Pane::$pane => Some(&self.$field),)*
                    _ => None,
                }
            }

            /// The page of `pane`, when it is a component page, with the
            /// todo page, whose dispatcher is the gallery's one.
            fn component_mut(&mut self, pane: Pane) -> Option<(&mut dyn Component, &mut TodoPage)> {
                match pane {
                    $(Pane::$pane => Some((&mut self.$field, &mut self.todo)),)*
                    _ => None,
                }
            }
        }
    };
}

components! {
    Breadcrumb => breadcrumb,
    Button => button,
    Checkbox => checkbox,
    Chip => chip,
    Dialog => dialog,
    IconButton => icon_button,
    Input => input,
    NavigationMenu => navigation_menu,
    Resizable => resizable,
    ScrollArea => scroll_area,
    Select => select,
    Slider => slider,
    Spinner => spinner,
    Switch => switch,
    Table => table,
    Tabs => tabs,
    Toast => toast,
    Tooltip => tooltip,
    Tree => tree,
    // task-104: each front's component pages under its anchor
    // task-104 front A, fields (#154)
    Field => field,
    Label => label,
    InputGroup => input_group,
    // task-104 front B, overlays (#155)
    AlertDialog => alert_dialog,
    ContextMenu => context_menu,
    Drawer => drawer,
    DropdownMenu => dropdown_menu,
    HoverCard => hover_card,
    Menubar => menubar,
    Popover => popover,
    Sheet => sheet,
    // task-104 front C, conversation and feedback (#156)
    Alert => alert,
    Attachment => attachment,
    Bubble => bubble,
    Marker => marker,
    Message => message,
    MessageScroller => message_scroller,
    // task-104 front D, disclosure and display (#157)
    Accordion => accordion,
    AspectRatio => aspect_ratio,
    Carousel => carousel,
    Collapsible => collapsible,
    Item => item,
    Kbd => kbd,
    Pagination => pagination,
    StaticTable => table_static,
    Panel => panel,
    PanelHeader => panel_header,
    // task-104 front E, selection and dates (#158)
    ButtonGroup => button_group,
    RadioGroup => radio_group,
    Toggle => toggle,
    ToggleGroup => toggle_group,
    // task-104 front G, navigation and data
    Chart => chart,
    CodeBlock => code_block,
    Graph => graph,
    Stat => stat,
}

/// The page's region, named by the page's title, inert while an overlay
/// holds the pointer, and the page under it.
fn region(page: &dyn Component, root: &SemId, title: &str, inert: bool, b: &mut SnapshotBuilder) {
    b.node(root, Role::Region, title).inert(inert);
    page.project(b, root);
}

/// A component page as the dispatcher sees it: it projects the page
/// under its region and runs what is asked of it, keeping what the view
/// does after.
struct Page<'a> {
    page: &'a mut dyn Component,
    root: SemId,
    title: &'static str,
    inert: bool,
    after: After,
}

impl Semantics for Page<'_> {
    fn project(&self, b: &mut SnapshotBuilder) {
        region(&*self.page, &self.root, self.title, self.inert, b);
    }
}

impl Operable for Page<'_> {
    /// Everything a component page does ends now.
    fn execute(&mut self, _id: RequestId, req: &Request) -> Execution {
        match self.page.run(req) {
            Ok(after) => {
                self.after = after;
                Execution::Done(Outcome::default())
            }
            Err(why) => Execution::Failed(why),
        }
    }
}

impl ShowcaseView {
    /// The component page of `pane` projected now, under its region;
    /// `None` for a pane whose page is not one.
    pub(in crate::views) fn component_semantics(
        &self,
        pane: Pane,
    ) -> Option<Result<Snapshot, SemErrors>> {
        let page = self.panes.component(pane)?;
        let root = page.root()?;
        let mut b = SnapshotBuilder::new();
        let inert = self.overlay_holds_pointer();
        region(page, root, self.current().title, inert, &mut b);
        Some(b.finish())
    }

    /// Whether `id` is the active component page's.
    pub fn is_component(&self, id: &SemId) -> bool {
        self.pane()
            .and_then(|pane| self.panes.component(pane))
            .and_then(Component::root)
            .is_some_and(|root| under(root, id))
    }

    /// A request aimed at the active component page, from any actor but
    /// the person's pointer, through the gallery's one dispatcher: the
    /// receipt as it stands, after what the page asked of the view is
    /// done. `None` when the active page is not a component page.
    pub(in crate::views) fn act_component(&mut self, req: Request) -> Option<Receipt> {
        let pane = self.pane()?;
        let title = self.current().title;
        let inert = self.overlay_holds_pointer();
        let (page, todo) = self.panes.component_mut(pane)?;
        let root = page.root()?.clone();
        let mut op = Page {
            page,
            root,
            title,
            inert,
            after: After::Nothing,
        };
        let id = todo.with_dispatcher(|d, _, now| d.dispatch(&mut op, req, now));
        let after = op.after;
        let receipt = todo.receipt(id).cloned();
        self.after_component(after);
        receipt
    }

    /// What a component page asked of the view, done: the click's path
    /// and the dispatcher's end here alike.
    pub(in crate::views) fn after_component(&mut self, after: After) {
        match after {
            After::Nothing => {}
            After::Toast(intent) => {
                self.toasts
                    .push(ToastPage::message(intent), intent, &self.theme);
            }
            After::Modal { destructive } => self.open_modal(destructive),
            // task-104 front B, overlays (#155)
            After::Sheet(edge) => self.open_sheet(edge),
            After::Drawer => self.open_drawer(),
            After::ContextMenu => self.open_page_menu(),
        }
    }

    /// Keyboard focus from a screen reader to a node of the active
    /// component page; refused while an overlay holds the pointer (the
    /// page is inert then).
    pub(in crate::views) fn focus_component(&mut self, id: &SemId) -> bool {
        if self.overlay_holds_pointer() {
            return false;
        }
        let Some(pane) = self.pane() else {
            return false;
        };
        self.panes
            .component_mut(pane)
            .is_some_and(|(page, _)| page.focus(id))
    }

    /// Escape on a component page: the focus a reader left goes.
    pub(in crate::views) fn blur_component(&mut self) -> bool {
        let Some(pane) = self.pane() else {
            return false;
        };
        self.panes
            .component_mut(pane)
            .is_some_and(|(page, _)| page.blur())
    }
}

#[cfg(test)]
mod tests;
