//! Embedded font registration shared by the render-side [`super::TextSystem`]
//! and the GPU-free [`super::TextMeasurer`].
//!
//! Both must register the exact same faces: layout measures with the
//! `TextMeasurer` and the `TextSystem` rasterizes, so a face present on one
//! side only makes advances diverge from pixels.
//!
//! Rubik (the HOFF reference UI typeface) and Inter ship in all UI weights
//! (400/500/600/700) as static faces. cosmic-text only keeps the requested
//! family when a face matches the requested weight *exactly*
//! (`FontFallbackIter::default_font_match_key` filters `font_weight_diff == 0`);
//! with a single weight present, weight-600 text fell through the per-word
//! platform fallback lists and picked arbitrary families (Apple SD Gothic Neo,
//! Menlo, ...) with huge advances. Every UI weight must be embedded.
//!
//! The generic sans-serif default is pinned to **Rubik** to match the
//! reference (its `app/layout.tsx` loads `next/font` Rubik). Inter stays
//! embedded as a named-family fallback.

use cosmic_text::fontdb::Database;

/// Register every embedded face and pin the generic family defaults.
pub(super) fn register_embedded_fonts(db: &mut Database) {
    // Rubik (SIL OFL, assets/fonts/LICENSE-Rubik-OFL.txt) — HOFF UI sans.
    db.load_font_data(include_bytes!("../../assets/fonts/Rubik-Regular.ttf").to_vec());
    db.load_font_data(include_bytes!("../../assets/fonts/Rubik-Medium.ttf").to_vec());
    db.load_font_data(include_bytes!("../../assets/fonts/Rubik-SemiBold.ttf").to_vec());
    db.load_font_data(include_bytes!("../../assets/fonts/Rubik-Bold.ttf").to_vec());

    // Inter (SIL OFL, assets/fonts/LICENSE-Inter-OFL.txt) — named-family fallback.
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
    // sans-serif to Rubik (reference typeface) for deterministic shaping.
    db.set_sans_serif_family("Rubik");
    db.set_monospace_family("JetBrains Mono");
}
