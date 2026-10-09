use engine::builder::{Element, div, text};

use crate::theme::*;

/// StatCard — metric display with label, value, unit, optional subtitle.
pub fn stat_card(label: &str, value: &str, unit: &str) -> StatCardBuilder {
    StatCardBuilder {
        label: label.to_string(),
        value: value.to_string(),
        unit: unit.to_string(),
        subtitle: None,
    }
}

pub struct StatCardBuilder {
    label: String,
    value: String,
    unit: String,
    subtitle: Option<String>,
}

impl StatCardBuilder {
    pub fn subtitle(mut self, s: &str) -> Self {
        self.subtitle = Some(s.to_string());
        self
    }

    pub fn build(self) -> Element {
        let mut col = div()
            .col()
            .bg(hud().surface_1)
            .border(1.0)
            .border_color(hud().surface_3)
            .p(12)
            .gap(hud().space_xs)
            .grow(1.0);

        // Label
        col = col.child(
            text(&self.label)
                .font_size(FONT_SM)
                .uppercase()
                .tracking(0.15)
                .text_color(hud().text_muted),
        );

        // Value + unit row
        let mut value_row = div()
            .row()
            .gap(hud().space_xs)
            .align_items(engine::builder::Align::End);

        value_row = value_row.child(
            text(&self.value)
                .font_size(FONT_2XL)
                .bold()
                .text_color(hud().text_primary),
        );

        if !self.unit.is_empty() {
            value_row = value_row.child(
                text(&self.unit)
                    .font_size(FONT_XS)
                    .text_color(hud().text_muted),
            );
        }

        col = col.child(value_row);

        // Subtitle
        if let Some(ref sub) = self.subtitle {
            col = col.child(text(sub).font_size(FONT_SM).text_color(hud().text_muted));
        }

        col
    }
}
