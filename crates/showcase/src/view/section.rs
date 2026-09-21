//! The gallery's sections: one per comps category plus the embedded apps,
//! with the title, icon and blurb the chrome shows for each.

// ---------------------------------------------------------------------------
// Sections
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Cards,
    Buttons,
    Forms,
    Overlays,
    Lists,
    Icons,
    Theme,
    Charts,
    Dock,
    App,
    Builder,
    Extras,
    Typography,
    Effects,
    Chrome,
    Editor,
    Git,
}

impl Section {
    pub const ALL: [Section; 17] = [
        Section::Cards,
        Section::Buttons,
        Section::Forms,
        Section::Overlays,
        Section::Lists,
        Section::Icons,
        Section::Theme,
        Section::Charts,
        Section::Dock,
        Section::App,
        Section::Builder,
        Section::Extras,
        Section::Typography,
        Section::Effects,
        Section::Chrome,
        Section::Editor,
        Section::Git,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Section::Cards => "Cards",
            Section::Buttons => "Buttons",
            Section::Forms => "Forms",
            Section::Overlays => "Overlays",
            Section::Lists => "Lists",
            Section::Icons => "Icons",
            Section::Theme => "Theme",
            Section::Charts => "Charts",
            Section::Dock => "Dock",
            Section::App => "App",
            Section::Builder => "Builder",
            Section::Extras => "Extras",
            Section::Typography => "Typography",
            Section::Effects => "Effects",
            Section::Chrome => "Chrome",
            Section::Editor => "Editor",
            Section::Git => "Git",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Section::Cards => "clipboard",
            Section::Buttons => "square",
            Section::Forms => "settings",
            Section::Overlays => "copy",
            Section::Lists => "file",
            Section::Icons => "eye",
            Section::Theme => "sun",
            Section::Charts => "git-branch",
            Section::Dock => "terminal",
            Section::App => "check",
            Section::Builder => "code",
            Section::Extras => "plus",
            Section::Typography => "info",
            Section::Effects => "moon",
            Section::Chrome => "layout-grid",
            Section::Editor => "code",
            Section::Git => "git-commit",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Section::Cards => {
                "The HOFF card deck: one glass shell, six preview families with live data."
            }
            Section::Buttons => {
                "The hoff core buttons: primary, call to action, warning, icon, like, toolbar, submit; one card per state."
            }
            Section::Forms => "Checkbox, switch, slider, progress, select and tabs.",
            Section::Overlays => {
                "Modal, context menu, tooltip and toasts — spring physics per intent."
            }
            Section::Lists => "Virtualized list with 10,000 rows, and a tree view.",
            Section::Icons => "Lucide icon set, tessellated to GPU paths with per-size caching.",
            Section::Theme => "HOFF tokens and every built-in palette. Click a card to apply it.",
            Section::Charts => {
                "Line, bars, stacked area and donut, drawn from a pure tested geometry core."
            }
            Section::Dock => "Floating message dock: glass blur, hover lift, width morph on click.",
            Section::App => "A complete small todo app: domain state, per-item animation, input.",
            Section::Builder => {
                "Builder tour with real hit regions and measured, content-driven layout."
            }
            Section::Extras => {
                "App-level widgets: chips, icon buttons, spinners, empty state, split pane."
            }
            Section::Typography => {
                "The HOFF scale set in Inclusive Sans — every weight embedded, every readout measured live."
            }
            Section::Effects => {
                "Analytic shadows, gradients, backdrop blur, clip stack and tessellated vector shapes."
            }
            Section::Chrome => {
                "The app frame: avatars, badges, panel header, nav links, breadcrumb, stats, code, skeleton, table."
            }
            Section::Editor => {
                "The multi-cursor text editor (comps::editor): alt+click adds cursors, cmd-z undoes."
            }
            Section::Git => {
                "The ide workspace on this repository: changes, stacks and diff. s stages, c commits, right-click for more."
            }
        }
    }
}
