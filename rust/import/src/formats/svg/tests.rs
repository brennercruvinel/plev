//! End-to-end tests over the public boundary: svg text in, .monster bytes
//! out, decoded back with the real monster codec. No mocks; the fixtures
//! are real (small) svg documents plus, when present, one of the corpus
//! svgs under refs/ (gitignored study material, so the test skips when it
//! is absent rather than failing a clean checkout).

use crate::{SvgError, convert};
use monster::asset_path::unpack;
use monster::{decode, stage_size};

const RECT: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="80">
  <rect x="10" y="10" width="80" height="60" fill="#ff8800"/>
</svg>"##;

// happy path: a solid rect converts, round-trips through the codec, and
// carries the stage size in its description track.
#[test]
fn converts_a_solid_rect() {
    let (bytes, stats) = convert(RECT, "rect").expect("convert");
    assert_eq!((stats.width as u32, stats.height as u32), (100, 80));
    assert!(stats.assets >= 1, "one filled rect is one asset");
    assert!(stats.nodes >= 1);

    let doc = decode(&bytes).expect("decode round-trip");
    assert_eq!(stage_size(&doc.descs), Some((100.0, 80.0)));
    let path = unpack(&doc.assets[0].data).expect("unpack path asset");
    assert!(
        !path.vertices.is_empty(),
        "the rect tessellated to geometry"
    );
    // color is sRGB straight through: #ff8800 -> (1.0, ~0.53, 0.0).
    let c = path.vertices[0].color;
    assert!(c[0] > 0.9 && c[1] > 0.4 && c[1] < 0.6 && c[2] < 0.1);
}

// error path: nothing drawable means an honest error, not an empty file
// that would claim to be a valid still image.
#[test]
fn empty_svg_is_nothing_to_convert() {
    let empty = r##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"></svg>"##;
    assert!(matches!(
        convert(empty, "empty"),
        Err(SvgError::NothingToConvert)
    ));
}

// error path: malformed xml is a typed parse error, never a panic.
#[test]
fn malformed_svg_is_a_parse_error() {
    assert!(matches!(convert("<svg", "bad"), Err(SvgError::Parse(_))));
}

// edge: a gradient fill has no single color; it must approximate to a
// solid and still produce drawable geometry (no panic, no skip).
#[test]
fn gradient_fill_approximated_as_solid() {
    let grad = r##"<svg xmlns="http://www.w3.org/2000/svg" width="50" height="50">
      <defs><linearGradient id="g"><stop offset="0" stop-color="#000"/>
      <stop offset="1" stop-color="#fff"/></linearGradient></defs>
      <rect width="50" height="50" fill="url(#g)"/>
    </svg>"##;
    let (_, stats) = convert(grad, "grad").expect("gradient converts");
    assert!(stats.nodes >= 1);
}

// edge: a stroked open path with no fill still converts (stroke geometry
// alone is drawable content).
#[test]
fn stroke_only_path_converts() {
    let line = r##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="40">
      <path d="M5 5 L35 35" fill="none" stroke="#00ff00" stroke-width="4"/>
    </svg>"##;
    let (_, stats) = convert(line, "line").expect("stroke converts");
    assert!(stats.nodes >= 1);
}

// the dedup thesis: two identical shapes share one asset but are two
// nodes, so the asset table stays smaller than the scene.
#[test]
fn identical_shapes_dedup_to_one_asset() {
    let twins = r##"<svg xmlns="http://www.w3.org/2000/svg" width="60" height="30">
      <rect x="5" y="5" width="20" height="20" fill="#123456"/>
      <rect x="5" y="5" width="20" height="20" fill="#123456"/>
    </svg>"##;
    let (_, stats) = convert(twins, "twins").expect("convert");
    assert_eq!(stats.nodes, 2, "two rects, two placed nodes");
    assert_eq!(stats.assets, 1, "identical geometry dedups to one payload");
}

// real artifact, guarded: convert one of the corpus svgs if present.
#[test]
fn corpus_svg_converts_when_present() {
    let rel = "/../../refs/lottie/MONEY/interactivebototnbar/interativebtn/\
               53fdc2f8-6c41-465c-92e7-ab540d2c90f6.svg";
    let path = concat!(env!("CARGO_MANIFEST_DIR"));
    let full = format!("{path}{rel}");
    let Ok(svg) = std::fs::read_to_string(&full) else {
        eprintln!("skip: corpus svg not present at {full}");
        return;
    };
    let (bytes, stats) = convert(&svg, "money-btn").expect("corpus svg converts");
    assert!(stats.nodes > 0 && stats.assets > 0);
    assert!(decode(&bytes).is_ok(), "corpus output round-trips");
}
