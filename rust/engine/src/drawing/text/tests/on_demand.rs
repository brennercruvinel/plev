//! The Noto faces a web build fetches when its text first needs them
//! (`super::super::on_demand`, issue #137), exercised on the desktop: the
//! tests turn fetching on for their thread and deliver the packs
//! themselves, from the files the desktop embeds.

use cosmic_text::fontdb::Database;
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping};

use super::super::backend::TextStyle;
use super::super::fonts::{NOTO, NOTO_EMOJI};
use super::super::measure::TextMeasurer;
use super::super::on_demand::{self, PACKS, packs_of};

fn pack(name: &str) -> usize {
    PACKS.iter().position(|p| p.name == name).unwrap()
}

fn bit(name: &str) -> u8 {
    1 << pack(name)
}

/// The files of pack `name`, read from the faces the desktop embeds.
fn files_of(name: &str) -> Vec<Vec<u8>> {
    let all: Vec<&[u8]> = NOTO.iter().copied().chain([NOTO_EMOJI]).collect();
    let start: usize = PACKS[..pack(name)].iter().map(|p| p.files.len()).sum();
    let len = PACKS[pack(name)].files.len();
    all[start..start + len].iter().map(|b| b.to_vec()).collect()
}

#[test]
fn a_text_asks_for_the_packs_of_its_scripts() {
    assert_eq!(packs_of("Expense Tracker 1,632"), 0, "latin asks nothing");
    assert_eq!(
        packs_of("café, naïve"),
        0,
        "latin beyond ascii asks nothing"
    );
    assert_eq!(packs_of("مرحبا"), bit("arabic"));
    assert_eq!(packs_of("שלום"), bit("hebrew"));
    assert_eq!(packs_of("नमस्ते"), bit("devanagari"));
    assert_eq!(packs_of("汉字"), bit("sc"));
    assert_eq!(packs_of("ひらがな"), bit("jp"));
    assert_eq!(packs_of("日本語のテキスト"), bit("jp") | bit("sc"));
    assert_eq!(packs_of("한국어"), bit("kr"));
    assert_eq!(packs_of("ok 🙂"), bit("emoji"));
    assert_eq!(packs_of("abc مرحبا 汉字"), bit("arabic") | bit("sc"));
}

/// The sha256 the web build checks are the files the desktop embeds, in
/// the order the desktop loads them, and the files the web serves.
#[test]
fn the_table_is_the_embedded_files_in_their_order() {
    use sha2::{Digest, Sha256};
    let hex = |b: &[u8]| -> String {
        Sha256::digest(b)
            .iter()
            .map(|x| format!("{x:02x}"))
            .collect()
    };
    let embedded: Vec<&[u8]> = NOTO.iter().copied().chain([NOTO_EMOJI]).collect();
    let table: Vec<(&str, &str)> = PACKS.iter().flat_map(|p| p.files.iter().copied()).collect();
    assert_eq!(embedded.len(), table.len());
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts");
    for (bytes, (name, sha)) in embedded.iter().zip(&table) {
        assert_eq!(hex(bytes), *sha, "{name}: the embedded face");
        let served = std::fs::read(format!("{dir}/{name}")).unwrap();
        assert_eq!(hex(&served), *sha, "{name}: the file the web serves");
    }
}

#[test]
fn a_pack_is_taken_only_whole_and_as_hashed() {
    on_demand::fetching();
    let before = on_demand::generation();
    let mut arabic = files_of("arabic");
    arabic[1][100] ^= 1;
    let err = on_demand::arrive(pack("arabic"), arabic).unwrap_err();
    assert!(err.contains("NotoSansArabic-Bold.ttf"), "{err}");
    let err = on_demand::arrive(pack("arabic"), files_of("arabic")[..1].to_vec()).unwrap_err();
    assert!(err.contains("1 files, 2 expected"), "{err}");
    assert_eq!(on_demand::generation(), before, "nothing arrived");
    on_demand::arrive(pack("arabic"), files_of("arabic")).unwrap();
    assert_eq!(on_demand::generation(), before + 1);
}

/// The family that shapes the first glyph of `text` in a font system of
/// the latin faces alone plus the packs that arrived, as the web build's.
fn family_of_first_glyph(text: &str) -> String {
    let mut db = Database::new();
    db.load_font_data(
        include_bytes!("../../../../assets/fonts/InclusiveSans-Regular.ttf").to_vec(),
    );
    db.set_sans_serif_family("Inclusive Sans");
    on_demand::load(&mut db, &[]);
    let mut fs = FontSystem::new_with_locale_and_db("en-US".to_string(), db);
    let mut buffer = Buffer::new(&mut fs, Metrics::new(16.0, 20.0));
    buffer.set_text(&mut fs, text, &Attrs::new(), Shaping::Advanced, None);
    buffer.shape_until_scroll(&mut fs, false);
    let run = buffer.layout_runs().next().unwrap();
    let id = run.glyphs[0].font_id;
    fs.db().face(id).unwrap().families[0].0.clone()
}

/// A Han-only text is SC's on the web as on the desktop, whatever the
/// order the packs arrived in. (Loading them in the table's order does not
/// decide it here: with the load reversed this still holds, so cosmic-text
/// picks SC by another rule; the order is kept for determinism.)
#[test]
fn han_is_sc_even_when_jp_arrived_first() {
    on_demand::fetching();
    on_demand::arrive(pack("jp"), files_of("jp")).unwrap();
    on_demand::arrive(pack("sc"), files_of("sc")).unwrap();
    assert_eq!(family_of_first_glyph("汉字"), "Noto Sans SC");
    assert_eq!(family_of_first_glyph("ひらがな"), "Noto Sans JP");
}

#[test]
fn measuring_a_text_asks_for_its_packs_once() {
    on_demand::fetching();
    let style = TextStyle::new(16.0);
    TextMeasurer::measure_styled("مرحبا", &style, None);
    TextMeasurer::measure_styled("한국어", &style, None);
    assert_eq!(on_demand::asked(), bit("arabic") | bit("kr"));
    let width = TextMeasurer::measure_styled("مرحبا", &style, None).0;
    on_demand::arrive(pack("arabic"), files_of("arabic")).unwrap();
    let again = TextMeasurer::measure_styled("مرحبا", &style, None).0;
    assert!(
        (width - again).abs() < 0.01,
        "the faces that arrived are the ones the desktop embeds: {width} then {again}"
    );
}

/// On a desktop, the system's faces for these scripts (Geeza Pro, Hiragino,
/// Apple Color Emoji, ...) were found before the embedded ones: every
/// machine drew them with its own fonts and the web measured them
/// otherwise. Each script shapes from its embedded Noto face, the spaces
/// from the UI face.
#[test]
fn every_script_shapes_from_its_embedded_face_on_the_desktop() {
    let style = TextStyle::new(14.0);
    for (text, family) in [
        ("مرحبا بالعالم", "Noto Sans Arabic"),
        ("שלום עולם", "Noto Sans Hebrew"),
        ("नमस्ते दुनिया", "Noto Sans Devanagari"),
        ("你好世界", "Noto Sans SC"),
        ("こんにちは", "Noto Sans JP"),
        ("안녕하세요 세계", "Noto Sans KR"),
        ("🙂 🎉 🚀", "Noto Color Emoji"),
    ] {
        let faces = TextMeasurer::resolved_faces(text, &style);
        assert!(
            faces.iter().any(|(f, _)| f == family),
            "{text}: no {family} in {faces:?}"
        );
        assert!(
            faces
                .iter()
                .all(|(f, _)| f == family || f == "Inclusive Sans"),
            "{text}: shaped from {faces:?}"
        );
    }
}
