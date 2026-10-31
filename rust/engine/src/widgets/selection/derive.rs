//! The selection recipe of every theme but the core kit, derived from
//! the theme built without it: the kit's relations on the theme's
//! palette.

use crate::styling::color::Color;
use crate::styling::theme::{ButtonRecipe, ButtonSurface, Theme};

use super::legible::{MARK_CONTRAST, full, legible, legible_at};
use super::tokens::*;

impl SelectionTokens {
    /// The recipe of any other theme, from the theme built without it
    /// (`Theme::derive_front_slots` fills the slots last): the well is the
    /// hover tone ringed by the divider, the thumb the background ringed
    /// by the active border, the labels the theme's text and its mid tone.
    pub fn derive(theme: &Theme) -> Self {
        let kit = Self::hoff_core();
        let c = &theme.colors;
        let track = ButtonSurface {
            fill: [c.bg_hover; 2],
            overlay: NONE,
            shadow: vec![ring(c.divider)],
            text: c.text,
        };
        let thumb = ButtonSurface {
            fill: [c.bg; 2],
            overlay: NONE,
            shadow: vec![ring(c.border_active)],
            text: c.text,
        };
        let button_group = ButtonGroupTokens {
            shell: ButtonSurface {
                fill: [c.surface; 2],
                overlay: NONE,
                shadow: vec![ring(c.divider)],
                text: c.text,
            },
            separator: c.divider,
            ..kit.button_group
        };
        // the kit's call to action is ink on paper, the page's colors
        // turned over: so is a derived theme's
        let chosen = ButtonSurface {
            fill: [c.text; 2],
            overlay: NONE,
            shadow: Vec::new(),
            text: c.bg,
        };
        // the kit's well is a step off the page and its on the ink at
        // 70%: the hover tone ringed by the divider, and the text at the
        // same strength; the knob is the page
        let strength = kit.switch.on.fill[0].0[3];
        let on = Color::rgba(
            c.text.0[0],
            c.text.0[1],
            c.text.0[2],
            c.text.0[3] * strength,
        );
        // the well off and the thumb ringed by the divider where it reads
        // at 3:1 on all they border (1.4.11), else the mid tone, else the
        // text
        let edges = [c.divider, c.text_mid, c.text];
        let well_edge = edge_on(&edges, &[&[c.bg], &[c.bg, c.bg_hover]]);
        let switch = SwitchTokens {
            off: ButtonSurface {
                fill: [c.bg_hover; 2],
                overlay: NONE,
                shadow: vec![ring(well_edge)],
                text: c.text,
            },
            on: ButtonSurface {
                fill: [on; 2],
                overlay: NONE,
                shadow: Vec::new(),
                text: c.text,
            },
            knob: ButtonSurface {
                fill: [c.bg; 2],
                overlay: NONE,
                shadow: vec![ring(c.divider)],
                text: c.text,
            },
            ..kit.switch
        };
        // the well a step off the page, filled with the quiet text at the
        // kit's strength (on its own alpha, as the switch's ink); the thumb
        // is the page, ringed where it reads
        let part = kit.slider.fill.0[3];
        let fill = Color::rgba(
            c.text_mid.0[0],
            c.text_mid.0[1],
            c.text_mid.0[2],
            c.text_mid.0[3] * part,
        );
        let thumb_edge = edge_on(&edges, &[&[c.bg, c.bg_hover], &[c.bg, c.bg_hover, fill]]);
        let slider = SliderTokens {
            track: ButtonSurface {
                fill: [c.bg_hover; 2],
                overlay: NONE,
                shadow: Vec::new(),
                text: c.text,
            },
            fill,
            thumb: ButtonSurface {
                shadow: vec![ring(thumb_edge)],
                ..switch.knob.clone()
            },
            ..kit.slider
        };
        // the tag on the page ringed by the divider, its quiet text where
        // it reads; chosen, the hover tone
        let tag = |fill: Color, edge: Color, text: Color| ButtonSurface {
            fill: [fill; 2],
            overlay: NONE,
            shadow: vec![border(edge)],
            text,
        };
        let chip = ChipTokens {
            rest: tag(c.bg, c.divider, legible(c.text_mid, c.text, c.bg, c.bg)),
            rest_hover: tag(c.bg, c.divider, c.text),
            on: tag(c.bg_hover, NONE, c.text),
            on_hover: tag(c.divider, NONE, c.text),
            ..kit.chip
        };
        let edge = legible_at(MARK_CONTRAST, c.divider, c.text_mid, c.bg_hover, c.bg);
        let checkbox = CheckboxTokens {
            off: ButtonSurface {
                shadow: vec![ring(edge)],
                ..switch.off.clone()
            },
            off_hover: ButtonSurface {
                overlay: c.bg_hover,
                shadow: vec![ring(edge)],
                ..switch.off.clone()
            },
            on: full(&switch.on),
            mark: switch.knob.fill[0],
            label: c.text,
            ..kit.checkbox
        };
        let radio = RadioTokens {
            row_hover: c.bg_hover,
            label: c.text,
            ..kit.radio
        };
        Self {
            segmented: SegmentedTokens {
                text: legible(c.text_mid, c.text, track.fill[0], c.bg),
                track,
                thumb,
                text_hover: c.text,
                ..kit.segmented
            },
            button_group,
            switch,
            checkbox,
            slider,
            chip,
            radio,
            calendar: CalendarTokens {
                weekday: legible(c.text_mid, c.text, c.surface, c.bg),
                today: legible_at(MARK_CONTRAST, c.border_active, c.text, c.surface, c.bg),
                today_w: theme.control.edge_width,
                chosen: ButtonRecipe {
                    rest: chosen.clone(),
                    hover: chosen.clone(),
                    active: chosen,
                    ..kit.calendar.chosen
                },
                ..kit.calendar
            },
        }
    }
}
