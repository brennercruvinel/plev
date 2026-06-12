use plev::builder::{Element, div, text};

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
            .bg(SURFACE_1)
            .border(1.0)
            .border_color(SURFACE_3)
            .p(12)
            .gap(SPACE_XS)
            .grow(1.0);

        // Label
        col = col.child(
            text(&self.label)
                .font_size(FONT_SM)
                .uppercase()
                .tracking(0.15)
                .text_color(TEXT_MUTED),
        );

        // Value + unit row
        let mut value_row = div()
            .row()
            .gap(SPACE_XS)
            .align_items(plev::builder::Align::End);

        value_row = value_row.child(
            text(&self.value)
                .font_size(FONT_2XL)
                .bold()
                .text_color(TEXT_PRIMARY),
        );

        if !self.unit.is_empty() {
            value_row = value_row.child(text(&self.unit).font_size(FONT_XS).text_color(TEXT_MUTED));
        }

        col = col.child(value_row);

        // Subtitle
        if let Some(ref sub) = self.subtitle {
            col = col.child(text(sub).font_size(FONT_SM).text_color(TEXT_MUTED));
        }

        col
    }
}
