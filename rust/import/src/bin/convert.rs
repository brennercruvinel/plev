//! convert: a lottie json or an svg document to our .monster format on the
//! command line. The input is read exactly once, here; whatever plays the
//! output never sees it. An svg is a still image, so its output is a
//! single-keyframe .monster that the player renders like any other.
//!
//! Run: `cargo run --release -p import --bin convert -- in.json|in.svg [out.monster]`
//! (default output: in.monster next to the input). Prints the measured
//! byte table: input vs monster, asset/timeline (lottie) or path (svg)
//! breakdown. Then play it with `cargo run --release -p player -- in.monster`.

fn main() {
    env_logger::init();
    let mut args = std::env::args().skip(1);
    let Some(input) = args.next() else {
        eprintln!("usage: convert <in.json|in.svg> [out.monster]");
        std::process::exit(2);
    };
    let output = args.next().unwrap_or_else(|| {
        std::path::Path::new(&input)
            .with_extension("monster")
            .to_string_lossy()
            .into_owned()
    });
    let text = match std::fs::read_to_string(&input) {
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
    let is_svg = std::path::Path::new(&input)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("svg"));
    if is_svg {
        svg(&text, &name, &output);
    } else {
        lottie(&text, &name, &output);
    }
}

fn write(output: &str, bytes: &[u8]) {
    if let Err(e) = std::fs::write(output, bytes) {
        eprintln!("cannot write {output}: {e}");
        std::process::exit(1);
    }
}

fn lottie(json: &str, name: &str, output: &str) {
    let (bytes, stats) = match lot::convert(json, name) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("conversion failed: {e}");
            std::process::exit(1);
        }
    };
    write(output, &bytes);

    let json_kb = json.len() as f64 / 1024.0;
    let monster_kb = stats.monster_bytes as f64 / 1024.0;
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
        100.0 * stats.asset_bytes as f64 / stats.monster_bytes as f64
    );
    println!(
        "  bytes: json {:.1} KB -> monster {:.1} KB ({:.2}x)",
        json_kb,
        monster_kb,
        monster_kb / json_kb
    );
}

fn svg(svg: &str, name: &str, output: &str) {
    let (bytes, stats) = match svg::convert(svg, name) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("conversion failed: {e}");
            std::process::exit(1);
        }
    };
    write(output, &bytes);

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
