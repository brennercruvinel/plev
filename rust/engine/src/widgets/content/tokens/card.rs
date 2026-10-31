//! The card's marks (`theme.disclosure.card`): the deck's chart and its
//! legend dots. The kit has no chart: the core draws them in the kit's
//! ink and gray, which read at 3:1 on its paper (WCAG 1.4.11), and every
//! other theme as it drew them before the kit, from its glass.

use super::*;
use crate::widgets::widget::with_alpha;

/// The chart's and the legend's marks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardRecipe {
    /// The two bars of a group, first and second.
    pub series: [Color; 2],
    /// The highlighted bar, top to bottom.
    pub highlight: [Color; 2],
    /// The cap floating over the highlighted bar.
    pub cap: Color,
    /// A solid legend dot, top to bottom.
    pub dot_solid: [Color; 2],
    /// A hollow legend dot: its fill and its ring.
    pub dot_fill: Color,
    pub dot_ring: Color,
    /// The chart's baseline and its ticks.
    pub axis: Color,
    pub tick: Color,
}

impl CardRecipe {
    /// The kit's palette on a chart: the gray (`#7B7B7B`, the kit's
    /// secondary text and icons, Dropdown/index.tsx:37) for the first
    /// bar and the ink (`#121212`, app/layout.tsx:108) for the second;
    /// the highlighted bar fades from the ink to the gray, its cap and a
    /// solid dot in the ink; a hollow dot the paper ringed in the gray;
    /// the baseline the kit's line (`#ECECEC`, Card/index.tsx:36), the
    /// ticks in the gray.
    pub fn hoff_core() -> Self {
        let (ink, gray) = (hex(0x121212), hex(0x7B7B7B));
        Self {
            series: [gray, ink],
            highlight: [ink, gray],
            cap: ink,
            dot_solid: [ink, ink],
            dot_fill: hex(0xFCFCFC),
            dot_ring: gray,
            axis: hex(0xECECEC),
            tick: gray,
        }
    }

    /// The marks every theme drew before the kit: the glass washes for
    /// the bars, the text at the hovered wash fading out for the
    /// highlight, the dim text for its cap; the knob's gradient for a
    /// solid dot, the hovered surface ringed in the focus border for a
    /// hollow one; the active surface for the baseline, the placeholder
    /// text for the ticks.
    pub fn derive(theme: &Theme) -> Self {
        let (glass, text) = (&theme.glass, theme.colors.text);
        Self {
            series: [glass.surface_hover, glass.surface_active],
            highlight: [
                Color(with_alpha(text, glass.wash_hover_alpha)),
                Color(with_alpha(text, 0.0)),
            ],
            cap: theme.colors.text_dim,
            dot_solid: glass.knob_gradient,
            dot_fill: glass.surface_hover,
            dot_ring: glass.field_focus_border,
            axis: glass.surface_active,
            tick: glass.text_placeholder,
        }
    }
}
