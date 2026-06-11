//! lot2anm: convert lottie json to our .anm format on the command
//! line. The json is read exactly once, here; whatever plays the
//! output never sees it.
//!
//! Run: `cargo run --release --example lot2anm -- in.json [out.anm]`
//! (default output: in.anm next to the input). Prints the measured
//! byte table: json vs anm, asset/timeline breakdown.

fn main() {
    env_logger::init();
    let mut args = std::env::args().skip(1);
    let Some(input) = args.next() else {
        eprintln!("usage: lot2anm <in.json> [out.anm]");
        std::process::exit(2);
    };
    let output = args.next().unwrap_or_else(|| {
        std::path::Path::new(&input)
            .with_extension("anm")
            .to_string_lossy()
            .into_owned()
    });
    let json = match std::fs::read_to_string(&input) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("cannot read {input}: {e}");
            std::process::exit(1);
        }
    };
    let name = std::path::Path::new(&input)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| input.clone());

    let (bytes, stats) = match lot::convert(&json, &name) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("conversion failed: {e}");
            std::process::exit(1);
        }
    };
    if let Err(e) = std::fs::write(&output, &bytes) {
        eprintln!("cannot write {output}: {e}");
        std::process::exit(1);
    }

    let json_kb = json.len() as f64 / 1024.0;
    let anm_kb = stats.anm_bytes as f64 / 1024.0;
    println!("{name} -> {output}");
    println!(
        "  stage {}x{}  {} frames @ {:.0} fps  {:.2}s",
        stats.width, stats.height, stats.frames, stats.fps, stats.duration_s
    );
    println!(
        "  timeline: {} keyframes, {} place / {} replace / {} remove",
        stats.keyframes, stats.places, stats.replaces, stats.removes
    );
    println!(
        "  assets: {} distinct payloads, {:.1} KB ({:.0}% of the file)",
        stats.assets,
        stats.asset_bytes as f64 / 1024.0,
        100.0 * stats.asset_bytes as f64 / stats.anm_bytes as f64
    );
    println!(
        "  bytes: json {:.1} KB -> anm {:.1} KB ({:.2}x)",
        json_kb,
        anm_kb,
        anm_kb / json_kb
    );
}
