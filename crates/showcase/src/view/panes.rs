//! The pages' state: one struct per pane (`pages::Pane`), built once
//! with the theme and owned by the view for the app's whole life.

use engine::theme::Theme;

use super::apps::{app, builder, dock, editor, git, silences, surface_page};
use super::components::{
    avatar, badge, breadcrumb, button, card, chart, checkbox, chip, code_block, context_menu,
    dialog, empty, graph, icon_button, input, navigation_menu, panel, panel_header, progress,
    resizable, scroll_area, select, separator, sidebar, skeleton, slider, spinner, stat, switch,
    table, tabs, toast, tooltip, tree,
};
use super::foundations::{effects, icons, theme, typography};

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
    // -- apps demo
    pub todo: app::TodoPage,
    /// Materialized over the todo's capabilities: it operates `todo`.
    pub surface: surface_page::SurfacePage,
    /// The silences app; it dispatches through `todo`'s dispatcher, the
    /// gallery's one.
    pub silences: silences::SilencesPage,
    pub dock: dock::DockPage,
    pub editor: editor::EditorPage,
    pub git: git::GitPage,
    pub builder: builder::BuilderPage,
}

impl Panes {
    pub fn new(theme: &Theme) -> Self {
        let todo = app::TodoPage::new();
        let surface = surface_page::SurfacePage::new(&todo);
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
            todo,
            surface,
            silences: silences::SilencesPage::new(),
            dock: dock::DockPage::new(),
            editor: editor::EditorPage::new(theme),
            git: git::GitPage::new(),
            builder: builder::BuilderPage::new(),
        }
    }
}
