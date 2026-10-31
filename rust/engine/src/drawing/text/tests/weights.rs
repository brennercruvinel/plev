//! A weight the named family has no face of draws in the family's nearest
//! one (css font matching), never in another family: the core theme's
//! Inter embeds 400, 500 and 600, so its 700 is the SemiBold.

use cosmic_text::fontdb::Database;

use super::super::backend::TextStyle;
use super::super::fonts::{matched_weight, register_embedded_fonts};
use super::super::measure::TextMeasurer;
use super::super::probe::{Faces, shape_report};
use crate::styling::theme::TypographyScale;

const TEXT: &str = "Sphinx of black quartz";

fn embedded() -> Database {
    let mut db = Database::new();
    register_embedded_fonts(&mut db);
    db
}

/// Every glyph of `style` comes from Inter, on the embedded faces and on
/// this machine's.
fn all_inter(style: &TextStyle) {
    for faces in [Faces::Embedded, Faces::Desktop] {
        let report = shape_report(TEXT, style, None, faces);
        assert_eq!(report.families(), ["Inter"], "{faces:?}");
        assert_eq!(report.notdef(), 0);
    }
}

#[test]
fn a_bold_core_style_is_inter_semibold() {
    let t = TypographyScale::hoff_core();
    let bold = t.exact(t.body, 20.0, 700);
    let semibold = t.exact(t.body, 20.0, 600);
    assert_eq!(
        TextMeasurer::measure_styled(TEXT, &bold, None),
        TextMeasurer::measure_styled(TEXT, &semibold, None),
    );
    all_inter(&bold);
    // `=base-2b`, the ramp's bold, too.
    all_inter(&t.base_2b());
}

#[test]
fn the_matched_weight_follows_css_font_matching() {
    let db = embedded();
    let inter = Some("Inter");
    // The weights it has stay.
    for w in [400, 500, 600] {
        assert_eq!(matched_weight(&db, inter, w), w);
    }
    // Above 500, heavier first, then lighter: 700 and 900 are the 600.
    assert_eq!(matched_weight(&db, inter, 700), 600);
    assert_eq!(matched_weight(&db, inter, 900), 600);
    // Below 400, lighter first, then heavier: 300 is the 400.
    assert_eq!(matched_weight(&db, inter, 300), 400);
    // Between 400 and 500, the gap the spec leaves: fontdb looks at 500
    // first below 450 and at 400 first from 450.
    assert_eq!(matched_weight(&db, inter, 420), 500);
    assert_eq!(matched_weight(&db, inter, 450), 400);
    // A family with every weight, no family, an unknown one: as asked.
    assert_eq!(matched_weight(&db, Some("Inclusive Sans"), 700), 700);
    assert_eq!(matched_weight(&db, None, 700), 700);
    assert_eq!(matched_weight(&db, Some("No Such Face"), 700), 700);
}
