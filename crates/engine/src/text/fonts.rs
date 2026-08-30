//! Embedded font registration shared by the render-side [`super::TextSystem`]
//! and the GPU-free [`super::TextMeasurer`].
//!
//! Both must register the exact same faces: layout measures with the
//! `TextMeasurer` and the `TextSystem` rasterizes, so a face present on one
//! side only makes advances diverge from pixels.
//!
//! Inclusive Sans ships in every UI weight (300/400/500/600/700) as static
//! faces. cosmic-text only keeps the requested family when a face matches the
//! requested weight *exactly* (`FontFallbackIter::default_font_match_key`
//! filters `font_weight_diff == 0`); with a single weight present, weight-600
//! text fell through the per-word platform fallback lists and picked arbitrary
//! families (Apple SD Gothic Neo, Menlo, ...) with huge advances. Every UI
//! weight must be embedded.
//!
//! The generic sans-serif default is pinned to **Inclusive Sans**, the UI
//! typeface. (The HOFF reference historically used Rubik via `next/font`;
//! Inclusive Sans replaces it as the UI sans.) Upstream also ships italic
//! faces (fonts/ttf/InclusiveSans-Italic.ttf and per-weight italics); they are
//! deliberately NOT embedded because the typography scale uses no italics.
//!
//! The typographic scale requests weights 400/500/600/700 only; Light (300)
//! is embedded so the full upstream upright set is available, but no style
//! slot uses it.

use cosmic_text::fontdb::Database;

/// Register every embedded face and pin the generic family defaults.
pub(super) fn register_embedded_fonts(db: &mut Database) {
    // Inclusive Sans (SIL OFL, assets/fonts/LICENSE-InclusiveSans-OFL.txt) — UI sans.
    db.load_font_data(include_bytes!("../../assets/fonts/InclusiveSans-Light.ttf").to_vec());
    db.load_font_data(include_bytes!("../../assets/fonts/InclusiveSans-Regular.ttf").to_vec());
    db.load_font_data(include_bytes!("../../assets/fonts/InclusiveSans-Medium.ttf").to_vec());
    db.load_font_data(include_bytes!("../../assets/fonts/InclusiveSans-SemiBold.ttf").to_vec());
    db.load_font_data(include_bytes!("../../assets/fonts/InclusiveSans-Bold.ttf").to_vec());

    // JetBrains Mono — code font.
    db.load_font_data(include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf").to_vec());
    db.load_font_data(include_bytes!("../../assets/fonts/JetBrainsMono-Bold.ttf").to_vec());

    // Codicons — VS Code icon font (CC BY 4.0; texto em assets/fonts/).
    db.load_font_data(include_bytes!("../../assets/fonts/codicons.ttf").to_vec());

    // `font_family: None` (the engine default) shapes as `Family::SansSerif`.
    // cosmic-text points that at "Open Sans", which is neither embedded nor
    // installed on most systems, so default-family text would resolve through
    // the per-word fallback chain (arbitrary faces once weight != 400). Pin
    // sans-serif to Inclusive Sans for deterministic shaping.
    db.set_sans_serif_family("Inclusive Sans");
    db.set_monospace_family("JetBrains Mono");
}
