//! svg2monster: convert an svg document to our .monster format on the
//! command line. The svg is read exactly once, here; whatever plays the
//! output never sees it. An svg is a still image, so the output is a
//! single-keyframe .monster that monster_player renders like any other.
//!
//! Run: `cargo run --release --example svg2monster -- in.svg [out.monster]`
//! (default output: in.monster next to the input). Prints the measured
//! byte table: svg vs monster, path/asset breakdown. Then play it with
//! `cargo run --release --example monster_player -- in.monster`.

fn main() {
    env_logger::init();
    let mut args = std::env::args().skip(1);
    let Some(input) = args.next() else {
        eprintln!("usage: svg2monster <in.svg> [out.monster]");
        std::process::exit(2);
    };
    let output = args.next().unwrap_or_else(|| {
        std::path::Path::new(&input)
            .with_extension("monster")
            .to_string_lossy()
            .into_owned()
    });
    let svg = match std::fs::read_to_string(&input) {
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

    let (bytes, stats) = match svg::convert(&svg, &name) {
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

    let svg_kb = svg.len() as f64 / 1024.0;
    let monster_kb = stats.monster_bytes as f64 / 1024.0;
    println!("{name} -> {output}");
    println!(
        "  stage {}x{}  {} paths",
        stats.width, stats.height, stats.paths
    );
    println!(
        "  scene: {} nodes, {} distinct payloads, {:.1} KB assets ({:.0}% of the file)",
        stats.nodes,
        stats.assets,
        stats.asset_bytes as f64 / 1024.0,
        100.0 * stats.asset_bytes as f64 / stats.monster_bytes.max(1) as f64
    );
    println!(
        "  bytes: svg {svg_kb:.1} KB -> monster {monster_kb:.1} KB ({:.2}x)",
        monster_kb / svg_kb
    );
}
