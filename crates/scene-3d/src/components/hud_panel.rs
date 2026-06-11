use plev::builder::{Align, Element, div, text};

use crate::theme::*;

/// HudPanel — the primary container component.
///
/// Replicates the React HudPanel: border surface-3, bg surface-1,
/// header bar with dot indicator + title + optional subtitle,
/// content area with padding.
pub fn hud_panel(title: &str, subtitle: &str) -> HudPanelBuilder {
    HudPanelBuilder {
        title: title.to_string(),
        subtitle: subtitle.to_string(),
        compact: false,
        children: Vec::new(),
    }
}

pub struct HudPanelBuilder {
    title: String,
    subtitle: String,
    compact: bool,
    children: Vec<Element>,
}

impl HudPanelBuilder {
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    pub fn child(mut self, child: Element) -> Self {
        self.children.push(child);
        self
    }

    pub fn children<I: IntoIterator<Item = Element>>(mut self, iter: I) -> Self {
        self.children.extend(iter);
        self
    }

    pub fn build(self) -> Element {
        let content_padding = if self.compact { 8.0 } else { 12.0 };

        // Header bar: dot + title | subtitle
        let left = div()
            .row()
            .gap(SPACE_SM)
            .align_items(Align::Center)
            .child(div().w(6).h(6).bg(WHITE_30).rounded(3.0))
            .child(
                text(&self.title)
                    .font_size(FONT_BASE)
                    .uppercase()
                    .tracking(0.2)
                    .text_color(WHITE_70),
            );

        let mut header = div()
            .row()
            .px(16)
            .py(10)
            .border_bottom(1.0, SURFACE_3)
            .align_items(Align::Center)
            .child(left);

        if !self.subtitle.is_empty() {
            header = header
                .child(
                    div().grow(1.0), // spacer
                )
                .child(
                    text(&self.subtitle)
                        .font_size(FONT_SM)
                        .text_color(TEXT_MUTED),
                );
        }

        // Content area
        let mut content = div().col().p(content_padding).gap(SPACE_SM).grow(1.0);
        for child in self.children {
            content = content.child(child);
        }

        // Assemble: border box → header → content
        div()
            .col()
            .bg(SURFACE_1)
            .border(1.0)
            .border_color(SURFACE_3)
            .grow(1.0)
            .child(header)
            .child(content)
    }
}
