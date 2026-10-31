//! The pages' state: one struct per pane (`pages::Pane`), built once
//! with the theme and owned by the view for the app's whole life.

use engine::styling::theme::Theme;

use super::super::apps::{builder, capture, dock, editor, files, git, silences, surface, todo};
use super::super::components::{
    avatar, badge, breadcrumb, button, card, chart, checkbox, chip, code_block, context_menu,
    dialog, empty, graph, icon_button, input, navigation_menu, panel, panel_header, progress,
    resizable, scroll_area, select, separator, sidebar, skeleton, slider, spinner, stat, switch,
    table, tabs, toast, tooltip, tree,
};
use super::super::foundations::{effects, icons, theme, typography};

// task-104: each front imports its pages under its anchor, a group of its
// own (the blank lines keep rustfmt from sorting one front into another)
// task-104 front A, fields (#154)
use super::super::components::field;

// task-104 front B, overlays (#155)
use super::super::components::popover;

// task-104 front C, conversation and feedback (#156)
use super::super::components::message;

// task-104 front D, disclosure and display (#157)
use super::super::components::{accordion, aspect_ratio, collapsible, item, kbd};

// task-104 front E, selection and dates (#158)
use super::super::components::toggle;

pub struct Panes {
    // -- foundations
    pub themes: theme::ThemePage,
    pub typography: typography::TypographyPage,
    pub icons: icons::IconsPage,
    pub effects: effects::EffectsPage,
    // -- components
    pub avatar: avatar::AvatarPage,
    pub badge: badge::BadgePage,
    pub breadcrumb: breadcrumb::BreadcrumbPage,
    pub button: button::ButtonPage,
    pub card: card::CardPage,
    pub chart: chart::ChartPage,
    pub checkbox: checkbox::CheckboxPage,
    pub chip: chip::ChipPage,
    pub code_block: code_block::CodeBlockPage,
    pub context_menu: context_menu::ContextMenuPage,
    pub dialog: dialog::DialogPage,
    pub empty: empty::EmptyPage,
    pub graph: graph::GraphPage,
    pub icon_button: icon_button::IconButtonPage,
    pub input: input::InputPage,
    pub navigation_menu: navigation_menu::NavigationMenuPage,
    pub panel: panel::PanelPage,
    pub panel_header: panel_header::PanelHeaderPage,
    pub progress: progress::ProgressPage,
    pub resizable: resizable::ResizablePage,
    pub scroll_area: scroll_area::ScrollAreaPage,
    pub select: select::SelectPage,
    pub separator: separator::SeparatorPage,
    pub sidebar: sidebar::SidebarPage,
    pub skeleton: skeleton::SkeletonPage,
    pub slider: slider::SliderPage,
    pub spinner: spinner::SpinnerPage,
    pub stat: stat::StatPage,
    pub switch: switch::SwitchPage,
    pub table: table::TablePage,
    pub tabs: tabs::TabsPage,
    pub toast: toast::ToastPage,
    pub tooltip: tooltip::TooltipPage,
    pub tree: tree::TreePage,
    // -- task-104: each front's pages under its anchor
    // task-104 front A, fields (#154)
    pub field: field::FieldPage,
    // task-104 front B, overlays (#155)
    pub popover: popover::PopoverPage,
    // task-104 front C, conversation and feedback (#156)
    pub message: message::MessagePage,
    // task-104 front D, disclosure and display (#157)
    pub accordion: accordion::AccordionPage,
    pub aspect_ratio: aspect_ratio::AspectRatioPage,
    pub collapsible: collapsible::CollapsiblePage,
    pub item: item::ItemPage,
    pub kbd: kbd::KbdPage,
    // task-104 front E, selection and dates (#158)
    pub toggle: toggle::TogglePage,
    // -- apps demo
    pub todo: todo::TodoPage,
    /// Materialized over the todo's capabilities: it operates `todo`.
    pub surface: surface::SurfacePage,
    /// The silences app; it dispatches through `todo`'s dispatcher, the
    /// gallery's one.
    pub silences: silences::SilencesPage,
    /// The file dialog and the file drop as capabilities; it dispatches
    /// through `todo`'s dispatcher too.
    pub files: files::FilesPage,
    /// The camera and the microphone as capabilities; it dispatches
    /// through `todo`'s dispatcher too, and holds no device until the
    /// shell gives it the platform's (`ShowcaseView::set_capture`).
    pub capture: capture::CapturePage,
    pub dock: dock::DockPage,
    pub editor: editor::EditorPage,
    pub git: git::GitPage,
    pub builder: builder::BuilderPage,
    /// Where the active page shows when it left the main window: a
    /// window of its own (task-76 item 2, `dispatch::Detached`). `None`,
    /// the usual: in the main window.
    pub detached: Option<super::super::dispatcher::Detached>,
    /// The shell's capabilities (task-76 item 4, `dispatch::Shell`): the
    /// page, the theme, the page in a window of its own, the recording's
    /// save, projected and run through the gallery's dispatcher; what was
    /// asked of the windows and the recording waits here for the shell.
    pub shell: super::super::dispatcher::Shell,
}

impl Panes {
    pub fn new(theme: &Theme) -> Self {
        let todo = todo::TodoPage::new();
        let surface = surface::SurfacePage::new(&todo);
        Self {
            themes: theme::ThemePage::new(),
            typography: typography::TypographyPage::new(),
            icons: icons::IconsPage::new(),
            effects: effects::EffectsPage::new(),
            avatar: avatar::AvatarPage::new(),
            badge: badge::BadgePage::new(),
            breadcrumb: breadcrumb::BreadcrumbPage::new(),
            button: button::ButtonPage::new(),
            card: card::CardPage::new(),
            chart: chart::ChartPage::new(),
            checkbox: checkbox::CheckboxPage::new(),
            chip: chip::ChipPage::new(),
            code_block: code_block::CodeBlockPage::new(),
            context_menu: context_menu::ContextMenuPage::new(),
            dialog: dialog::DialogPage::new(),
            empty: empty::EmptyPage::new(),
            graph: graph::GraphPage::new(),
            icon_button: icon_button::IconButtonPage::new(),
            input: input::InputPage::new(),
            navigation_menu: navigation_menu::NavigationMenuPage::new(),
            panel: panel::PanelPage::new(),
            panel_header: panel_header::PanelHeaderPage::new(),
            progress: progress::ProgressPage::new(),
            resizable: resizable::ResizablePage::new(theme),
            scroll_area: scroll_area::ScrollAreaPage::new(theme),
            select: select::SelectPage::new(),
            separator: separator::SeparatorPage::new(),
            sidebar: sidebar::SidebarPage::new(),
            skeleton: skeleton::SkeletonPage::new(),
            slider: slider::SliderPage::new(),
            spinner: spinner::SpinnerPage::new(),
            stat: stat::StatPage::new(),
            switch: switch::SwitchPage::new(theme),
            table: table::TablePage::new(theme),
            tabs: tabs::TabsPage::new(),
            toast: toast::ToastPage::new(),
            tooltip: tooltip::TooltipPage::new(),
            tree: tree::TreePage::new(),
            // task-104 front A, fields (#154)
            field: field::FieldPage::new(),
            // task-104 front B, overlays (#155)
            popover: popover::PopoverPage::new(),
            // task-104 front C, conversation and feedback (#156)
            message: message::MessagePage::new(),
            // task-104 front D, disclosure and display (#157)
            accordion: accordion::AccordionPage::new(theme),
            aspect_ratio: aspect_ratio::AspectRatioPage::new(),
            collapsible: collapsible::CollapsiblePage::new(theme),
            item: item::ItemPage::new(),
            kbd: kbd::KbdPage::new(),
            // task-104 front E, selection and dates (#158)
            toggle: toggle::TogglePage::new(),
            todo,
            surface,
            silences: silences::SilencesPage::new(),
            files: files::FilesPage::new(),
            capture: capture::CapturePage::new(theme),
            dock: dock::DockPage::new(),
            editor: editor::EditorPage::new(theme),
            git: git::GitPage::new(),
            builder: builder::BuilderPage::new(),
            detached: None,
            shell: super::super::dispatcher::Shell::new(),
        }
    }
}
