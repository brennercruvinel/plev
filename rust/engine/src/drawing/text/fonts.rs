//! Embedded font registration shared by the render-side [`super::TextSystem`]
//! and the GPU-free [`super::TextMeasurer`].
//!
//! Both must register the exact same faces: layout measures with the
//! `TextMeasurer` and the `TextSystem` rasterizes, so a face present on one
//! side only makes advances diverge from pixels. [`font_system`] is the one
//! constructor both (and the text probe) go through.
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
//!
//! The non-Latin scripts (docs/adr/rendering/0071-embedded-fonts-by-script.md) come from
//! Noto Sans subsets, Regular and Bold: Arabic, Hebrew, Devanagari, SC, JP,
//! KR. No family names them: a word whose glyphs the requested face lacks
//! reaches cosmic-text's last fallback step, every face in the database
//! sorted by weight distance and then by load order, and the Noto face that
//! covers the script is the first hit (the step ignores weight, so 500 lands
//! on Regular and 600 on Bold). Load order therefore decides the unified
//! Han: SC loads before JP (the `en-US` choice cosmic-text makes too), and
//! KR carries no Han at all.
//!
//! Emoji come from a Noto Color Emoji subset in its CBDT build (PNG
//! bitmaps, drawn by the color path of the glyph atlas), loaded last. It
//! needs no place in the order: cosmic-text sorts a face whose PostScript
//! name holds "Emoji" before every other in that last step
//! (`FontMatchKey::not_emoji`), and its cmap holds nothing another
//! embedded face has but the heart (Inter's) and the joiner.

use std::cell::Cell;
use std::sync::Arc;

use cosmic_text::FontSystem;
use cosmic_text::fontdb::{Database, ID, Source};

/// The locale of the embedded path. cosmic-text reads it for Han
/// unification in its platform lists; the embedded faces are reached past
/// those lists, so today it picks nothing.
const EMBEDDED_LOCALE: &str = "en-US";

/// Noto Sans subsets (SIL OFL, assets/fonts/LICENSE-Noto-OFL.txt), made by
/// `tool/tasks/subsetter.py`. The order is the fallback order among faces of the
/// same weight: SC before JP is the Han order, see the module docs.
#[cfg(not(feature = "experiment-no-noto"))]
const NOTO: [&[u8]; 12] = [
    include_bytes!("../../../assets/fonts/NotoSansArabic-Regular.ttf"),
    include_bytes!("../../../assets/fonts/NotoSansArabic-Bold.ttf"),
    include_bytes!("../../../assets/fonts/NotoSansHebrew-Regular.ttf"),
    include_bytes!("../../../assets/fonts/NotoSansHebrew-Bold.ttf"),
    include_bytes!("../../../assets/fonts/NotoSansDevanagari-Regular.ttf"),
    include_bytes!("../../../assets/fonts/NotoSansDevanagari-Bold.ttf"),
    include_bytes!("../../../assets/fonts/NotoSansSC-Regular.otf"),
    include_bytes!("../../../assets/fonts/NotoSansSC-Bold.otf"),
    include_bytes!("../../../assets/fonts/NotoSansJP-Regular.otf"),
    include_bytes!("../../../assets/fonts/NotoSansJP-Bold.otf"),
    include_bytes!("../../../assets/fonts/NotoSansKR-Regular.otf"),
    include_bytes!("../../../assets/fonts/NotoSansKR-Bold.otf"),
];

/// Noto Color Emoji, a CBDT subset (SIL OFL,
/// assets/fonts/LICENSE-NotoColorEmoji-OFL.txt), made by `tool/tasks/subsetter.py`.
#[cfg(not(feature = "experiment-no-noto"))]
const NOTO_EMOJI: &[u8] = include_bytes!("../../../assets/fonts/NotoColorEmoji-CBDT.ttf");
// EXPERIMENT, not committed: the web build weighed without the Noto faces
#[cfg(feature = "experiment-no-noto")]
const NOTO: [&[u8]; 0] = [];
#[cfg(feature = "experiment-no-noto")]
const NOTO_EMOJI: &[u8] = &[];

/// The faces a font system holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FaceSet {
    /// What this target shapes with: the embedded faces over the system
    /// fonts on a desktop, the embedded faces alone on wasm, android, ios.
    Target,
    /// The embedded faces alone on every target, so a desktop can shape
    /// and draw what wasm and mobile ship. Only the text probe asks for it,
    /// and the probe does not exist on wasm.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    Embedded,
}

thread_local! {
    /// The set the next [`super::TextSystem`] built on this thread holds.
    /// The text probe switches it around a headless render of the embedded
    /// path; nothing else writes it.
    static TEXT_SYSTEM_FACES: Cell<FaceSet> = const { Cell::new(FaceSet::Target) };
}

/// Build a font system holding `set`, with every embedded face registered
/// and the generic families pinned. Returns the fontdb IDs of the embedded
/// faces so the rasterizer can detect silent fallbacks to system faces.
pub(super) fn font_system(set: FaceSet) -> (FontSystem, Vec<ID>) {
    let mut system = match set {
        FaceSet::Target => target_font_system(),
        FaceSet::Embedded => embedded_font_system(),
    };
    let embedded = register_embedded_fonts(system.db_mut());
    (system, embedded)
}

/// The set a [`super::TextSystem`] built now on this thread holds.
pub(super) fn text_system_faces() -> FaceSet {
    TEXT_SYSTEM_FACES.with(Cell::get)
}

/// Run `f` with every [`super::TextSystem`] built on this thread holding
/// `set`; the previous set comes back afterwards, panic or not.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn with_text_system_faces<R>(set: FaceSet, f: impl FnOnce() -> R) -> R {
    struct Restore(FaceSet);
    impl Drop for Restore {
        fn drop(&mut self) {
            TEXT_SYSTEM_FACES.with(|faces| faces.set(self.0));
        }
    }
    let _restore = Restore(TEXT_SYSTEM_FACES.with(|faces| faces.replace(set)));
    f()
}

/// System fonts (desktop) are fallback only; the embedded faces are the
/// source of truth so rasterization matches `TextMeasurer`.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
fn target_font_system() -> FontSystem {
    FontSystem::new()
}

#[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
fn target_font_system() -> FontSystem {
    embedded_font_system()
}

fn embedded_font_system() -> FontSystem {
    FontSystem::new_with_locale_and_db(EMBEDDED_LOCALE.to_string(), Database::new())
}

/// Register every embedded face and pin the generic family defaults.
/// Returns the fontdb IDs of the embedded faces.
pub(super) fn register_embedded_fonts(db: &mut Database) -> Vec<ID> {
    // load_font_data returns (); snapshot the face set to learn which IDs
    // the embedded faces got (desktop DBs already hold system faces).
    let before: std::collections::HashSet<ID> = db.faces().map(|f| f.id).collect();

    // Inclusive Sans (SIL OFL, assets/fonts/LICENSE-InclusiveSans-OFL.txt), the UI sans.
    db.load_font_data(include_bytes!("../../../assets/fonts/InclusiveSans-Light.ttf").to_vec());
    db.load_font_data(include_bytes!("../../../assets/fonts/InclusiveSans-Regular.ttf").to_vec());
    db.load_font_data(include_bytes!("../../../assets/fonts/InclusiveSans-Medium.ttf").to_vec());
    db.load_font_data(include_bytes!("../../../assets/fonts/InclusiveSans-SemiBold.ttf").to_vec());
    db.load_font_data(include_bytes!("../../../assets/fonts/InclusiveSans-Bold.ttf").to_vec());

    // Inter (SIL OFL, assets/fonts/LICENSE-Inter-OFL.txt), the hoff core
    // design system's face; themes select it through
    // `TypographyScale::family`.
    db.load_font_data(include_bytes!("../../../assets/fonts/Inter-Regular.ttf").to_vec());
    db.load_font_data(include_bytes!("../../../assets/fonts/Inter-Medium.ttf").to_vec());
    db.load_font_data(include_bytes!("../../../assets/fonts/Inter-SemiBold.ttf").to_vec());

    // Codicons, the VS Code icon font (CC BY 4.0; texto em assets/fonts/).
    db.load_font_data(include_bytes!("../../../assets/fonts/codicons.ttf").to_vec());

    // The Noto faces load last, after every Latin face: a glyph the Latin
    // faces carry never reaches them. Borrowed from the binary instead of
    // copied (6.6 MB, once per font system). The emoji face goes last.
    for face in NOTO {
        db.load_font_source(Source::Binary(Arc::new(face)));
    }
    if !NOTO_EMOJI.is_empty() {
        db.load_font_source(Source::Binary(Arc::new(NOTO_EMOJI)));
    }

    // `font_family: None` (the engine default) shapes as `Family::SansSerif`.
    // cosmic-text points that at "Open Sans", which is neither embedded nor
    // installed on most systems, so default-family text would resolve through
    // the per-word fallback chain (arbitrary faces once weight != 400). Pin
    // sans-serif to Inclusive Sans for deterministic shaping. Monospace is
    // pinned to the same family: no mono face is embedded, so leaving the
    // cosmic-text default would send `Family::Monospace` text through the
    // platform fallback chain.
    db.set_sans_serif_family("Inclusive Sans");
    db.set_monospace_family("Inclusive Sans");
    db.faces()
        .filter(|f| !before.contains(&f.id))
        .map(|f| f.id)
        .collect()
}
