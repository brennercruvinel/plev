//! Embedded font registration shared by the render-side [`super::TextSystem`]
//! and the GPU-free [`super::TextMeasurer`].
//!
//! Both must register the exact same faces: layout measures with the
//! `TextMeasurer` and the `TextSystem` rasterizes, so a face present on one
//! side only makes advances diverge from pixels.
//!
//! Inter ships in all UI weights (400/500/600/700) as static faces.
//! cosmic-text only keeps the requested family when a face matches the
//! requested weight *exactly* (`FontFallbackIter::default_font_match_key`
//! filters `font_weight_diff == 0`); with Inter-Regular alone, weight-600
//! text fell through the per-word platform fallback lists and picked
//! arbitrary families (Apple SD Gothic Neo, Menlo, ...) with huge advances.

use cosmic_text::fontdb::Database;

/// Register every embedded face and pin the generic family defaults.
pub(super) fn register_embedded_fonts(db: &mut Database) {
    // Inter (SIL OFL, assets/fonts/LICENSE-Inter-OFL.txt) — UI sans.
    db.load_font_data(include_bytes!("../../assets/fonts/Inter-Regular.ttf").to_vec());
    db.load_font_data(include_bytes!("../../assets/fonts/Inter-Medium.ttf").to_vec());
    db.load_font_data(include_bytes!("../../assets/fonts/Inter-SemiBold.ttf").to_vec());
    db.load_font_data(include_bytes!("../../assets/fonts/Inter-Bold.ttf").to_vec());

    // JetBrains Mono — code font.
    db.load_font_data(include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf").to_vec());
    db.load_font_data(include_bytes!("../../assets/fonts/JetBrainsMono-Bold.ttf").to_vec());

    // Codicons — VS Code icon font (MIT).
    db.load_font_data(include_bytes!("../../assets/fonts/codicons.ttf").to_vec());

    // `font_family: None` (the engine default) shapes as `Family::SansSerif`.
    // cosmic-text points that at "Open Sans", which is neither embedded nor
    // installed on most systems, so default-family text would resolve through
    // the per-word fallback chain (arbitrary faces once weight != 400). Pin
    // the generic families to the embedded faces for deterministic shaping.
    db.set_sans_serif_family("Inter");
    db.set_monospace_family("JetBrains Mono");
}
