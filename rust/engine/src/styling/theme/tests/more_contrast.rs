//! The theme adapted to more contrast (adr-0086, task-104 front M),
//! measured on the same pairs as `contrast.rs`: with the default
//! preferences the adapted theme is the base; with `Contrast::More` no
//! pair falls under AA, the palette's text reads at AAA (7:1) and its
//! marks at 4.5:1 on every surface, and the focus ring and the control's
//! edge are wider.

use super::contrast::{AAA, LARGE, TEXT, pairs, themes};
use crate::styling::theme::{Contrast, Prefs};

fn more() -> Prefs {
    Prefs::default().with_contrast(Contrast::More)
}

/// The root tokens `legibility::settle` holds: the palette's and the
/// glass's text, the intents, the accent.
/// The active border is an edge, held at 3:1 (`Bar::MORE.edge`).
fn is_palette(fg: &str) -> bool {
    let edge = fg == "colors.border_active";
    !edge
        && (fg.starts_with("colors.")
            || (fg.starts_with("glass.text_") && !fg.contains("gradient")))
}

#[test]
fn adapted_to_the_default_preferences_is_the_base() {
    for (name, t) in themes() {
        let same = format!("{:?}", t.adapted(&Prefs::default())) == format!("{t:?}");
        assert!(same, "{name}");
    }
}

#[test]
fn more_contrast_holds_every_pair_at_aa_and_the_palette_at_aaa() {
    for (name, base) in themes() {
        let t = base.adapted(&more());
        let mut texts = (0, 0);
        for p in pairs(name, &t) {
            assert!(
                p.ratio >= p.need,
                "{name} {} on {}: {:.2}",
                p.fg,
                p.bg,
                p.ratio
            );
            if p.need >= TEXT {
                texts.0 += 1;
                texts.1 += usize::from(p.ratio >= AAA);
            }
            if is_palette(&p.fg) {
                let bar = if p.need > LARGE { AAA } else { TEXT };
                assert!(
                    p.ratio >= bar,
                    "{name} {} on {}: {:.2}",
                    p.fg,
                    p.bg,
                    p.ratio
                );
            }
        }
        // measured at task-104 front M: the text pairs at AAA of all the
        // text pairs. The rest stay at AA: the core's kit copies (the
        // placeholders, the link, the helper and error lines, hoff's
        // modals), which have no adapt step yet, and what the tokens hold
        // at AA's bar whatever the preference (a solid button's label on
        // its fill, an error line)
        let least = match name {
            "core" => (125, 137),
            "hoff" => (124, 128),
            "dark" => (125, 128),
            _ => (121, 128),
        };
        assert_eq!(texts.0, least.1, "{name}: the text pairs");
        assert!(texts.1 >= least.0, "{name}: {} at AAA", texts.1);
    }
}

#[test]
fn more_contrast_widens_the_focus_ring_and_the_edge() {
    for (name, base) in themes() {
        let t = base.adapted(&more());
        assert!(
            t.control.focus_ring_width > base.control.focus_ring_width,
            "{name}"
        );
        assert!(
            t.control.edge_width >= base.control.edge_width_strong,
            "{name}"
        );
        let g = &t.glass;
        for c in [g.surface, g.popover, g.field, g.tooltip] {
            assert_eq!(c.0[3], 1.0, "{name}: the glass is opaque");
        }
    }
}
