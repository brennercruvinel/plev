//! Look at what the rasterizer actually draws, without a window.
//!
//! `cargo run -p engine --example text_probe -- out.png [raster_scale]`
//!
//! Set `PROBE_FILL=1` to push enough distinct glyphs through the atlas that
//! it has to grow mid-frame — the state in which the defects described in
//! `docs/adr/glyph-raster-identity-and-atlas-isolation.md` become visible.
//! A probe that draws a single line never reaches it.

use engine::text::TextStyle;
use engine::text::probe::{Specimen, render};

fn main() {
    let _ = env_logger::builder().filter_level(log::LevelFilter::Info).try_init();
    let mut args = std::env::args().skip(1);
    let out = args.next().unwrap_or_else(|| "text_probe.png".into());
    let scale: f32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(2.0);

    let ramp = engine::theme::TypographyScale::hoff();
    let mut specimens = vec![
        ("Builder".to_string(), ramp.title()),
        ("Expense Tracker".to_string(), ramp.title()),
        ("4K Video Streaming".to_string(), ramp.title()),
        ("Dock".to_string(), ramp.title()),
        ("2 active \u{b7} 1 done".to_string(), ramp.base_2r()),
        (
            "tree rebuilt from state 0 times".to_string(),
            ramp.base_2r(),
        ),
        (
            "The quick brown fox jumps over the lazy dog".to_string(),
            ramp.base_r(),
        ),
    ];

    if std::env::var("PROBE_FILL").is_ok() {
        specimens.extend(engine::text::probe::atlas_filling_specimens());
    }

    let scene: Vec<_> = specimens
        .into_iter()
        .enumerate()
        .map(|(i, (text, style))| Specimen::new(text, style, 20.0, 20.0 + i as f32 * 40.0))
        .collect();

    let height = 40 * scene.len() as u32 + 40;
    match render(&scene, 900, height, scale) {
        Some(img) => {
            img.write_png(&out);
            println!(
                "wrote {out} ({}x{}, raster_scale {scale}, {} inked px)",
                img.width,
                img.height,
                img.ink()
            );
        }
        None => eprintln!("no GPU adapter available"),
    }
    let _ = TextStyle::new(16.0);
}
