use super::*;

#[test]
fn every_icon_tessellates_with_vertices() {
    for name in icon_names() {
        let path = icon_path(name, 16.0, WHITE, 0.0, 0.0).unwrap();
        assert!(
            !path.vertices.is_empty(),
            "icon '{name}' produced no vertices"
        );
        assert!(
            !path.indices.is_empty(),
            "icon '{name}' produced no indices"
        );
        assert_eq!(path.indices.len() % 3, 0, "icon '{name}' not triangles");
    }
}

/// `icon_names` documents its result as sorted, and it returns the table
/// verbatim - so the table is what has to stay sorted. Nothing enforced
/// that before, and an icon appended at the end would have made the doc
/// quietly false for every caller that renders the set as a list.
#[test]
fn the_icon_table_is_sorted_by_name() {
    let names = icon_names();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    assert_eq!(names, sorted, "ICONS must stay in alphabetical order");
}

#[test]
fn unknown_icon_returns_none() {
    assert!(icon("definitely-not-an-icon", 16.0, WHITE).is_none());
    assert!(!has_icon("definitely-not-an-icon"));
    assert!(has_icon("folder"));
}

#[test]
fn icon_geometry_stays_within_viewbox_bounds() {
    // Stroke extends half a width past the 24x24 box; allow that margin.
    let size = 24.0;
    let margin = STROKE_WIDTH;
    for name in icon_names() {
        let path = icon_path(name, size, WHITE, 0.0, 0.0).unwrap();
        for v in &path.vertices {
            assert!(
                v.position[0] >= -margin && v.position[0] <= size + margin,
                "icon '{name}' x={} outside bounds",
                v.position[0]
            );
            assert!(
                v.position[1] >= -margin && v.position[1] <= size + margin,
                "icon '{name}' y={} outside bounds",
                v.position[1]
            );
        }
    }
}

#[test]
fn icon_at_translates_geometry() {
    let at_origin = icon_path("x", 16.0, WHITE, 0.0, 0.0).unwrap();
    let moved = icon_path("x", 16.0, WHITE, 100.0, 50.0).unwrap();
    assert_eq!(at_origin.vertices.len(), moved.vertices.len());
    for (a, b) in at_origin.vertices.iter().zip(moved.vertices.iter()) {
        assert!((b.position[0] - a.position[0] - 100.0).abs() < 1e-4);
        assert!((b.position[1] - a.position[1] - 50.0).abs() < 1e-4);
    }
}

#[test]
fn icon_color_is_applied() {
    let tint = [0.2, 0.4, 0.6, 0.8];
    let path = icon_path("check", 16.0, tint, 0.0, 0.0).unwrap();
    assert!(path.vertices.iter().all(|v| v.color == tint));
}

#[test]
fn hash_distinguishes_position_and_color() {
    let a = icon_path("folder", 16.0, WHITE, 0.0, 0.0).unwrap();
    let b = icon_path("folder", 16.0, WHITE, 1.0, 0.0).unwrap();
    let c = icon_path("folder", 16.0, [1.0, 0.0, 0.0, 1.0], 0.0, 0.0).unwrap();
    let d = icon_path("folder", 16.0, WHITE, 0.0, 0.0).unwrap();
    assert_ne!(a.hash, b.hash);
    assert_ne!(a.hash, c.hash);
    assert_eq!(a.hash, d.hash, "same request must be hash-stable");
}

#[test]
fn cache_returns_identical_geometry() {
    let first = icon_path("settings", 20.0, WHITE, 0.0, 0.0).unwrap();
    let second = icon_path("settings", 20.0, WHITE, 0.0, 0.0).unwrap();
    assert_eq!(first.vertices.len(), second.vertices.len());
    assert_eq!(first.indices, second.indices);
}

#[test]
fn sizes_scale_geometry() {
    let small = icon_path("circle", 12.0, WHITE, 0.0, 0.0).unwrap();
    let max_small = small
        .vertices
        .iter()
        .map(|v| v.position[0].max(v.position[1]))
        .fold(f32::MIN, f32::max);
    let large = icon_path("circle", 48.0, WHITE, 0.0, 0.0).unwrap();
    let max_large = large
        .vertices
        .iter()
        .map(|v| v.position[0].max(v.position[1]))
        .fold(f32::MIN, f32::max);
    assert!(max_large > max_small * 3.0);
}

#[test]
fn arc_parser_handles_compact_flags() {
    // Flags glued to the next number, as some minified SVGs emit.
    let pb = parse_svg_path("M0 0a1 1 0 01-.7.7", 1.0).unwrap();
    let path = pb.stroke_round(WHITE, 2.0);
    assert!(!path.vertices.is_empty());
}

#[test]
fn parser_rejects_garbage() {
    assert!(parse_svg_path("M0 0 ? 3 4", 1.0).is_none());
}

/// What the icon set costs to tessellate today (task-72 phase 1, the
/// probe behind the inventory in
/// docs/tarefas/task-72-ui-asset-compiler.md): every icon at
/// every size the tokens name (`theme.control.icons`), the fastest of a
/// few runs of `tessellate_icon` each, the geometry it leaves in the
/// cache, and what a baked table of the same geometry would weigh in the
/// binary (positions as two `f32`, the tint applied per request as now,
/// indices as `u16` when they fit). The debug profile's numbers.
#[test]
#[ignore = "a measurement: cargo test -p engine --lib icons::tests::the_tessellation_cost -- --ignored --nocapture"]
fn the_tessellation_cost() {
    use std::time::{Duration, Instant};

    const ROUNDS: usize = 20;
    let sizes = crate::styling::theme::Theme::hoff_core().control.icons;
    let vertex = std::mem::size_of::<QuadVertex>();
    println!(
        "| size | icons | tessellate, sum (us) | median (us) | max (us) | vertices | indices | cache (bytes) | baked (bytes) |"
    );
    println!("| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |");
    let (mut all_time, mut all_cache, mut all_baked) = (Duration::ZERO, 0, 0);
    for size in sizes {
        let mut times = Vec::new();
        let (mut vertices, mut indices, mut widest) = (0, 0, 0u32);
        for (_, shapes) in ICONS {
            let mut best = Duration::MAX;
            let mut icon = None;
            for _ in 0..ROUNDS {
                let start = Instant::now();
                let tessellated = tessellate_icon(shapes, size);
                best = best.min(start.elapsed());
                icon = Some(tessellated);
            }
            let icon = icon.expect("ROUNDS > 0");
            times.push(best);
            vertices += icon.vertices.len();
            indices += icon.indices.len();
            widest = widest.max(icon.indices.iter().copied().max().unwrap_or(0));
        }
        times.sort();
        let sum: Duration = times.iter().sum();
        let index_bytes = if widest <= u32::from(u16::MAX) { 2 } else { 4 };
        let cache = vertices * vertex + indices * 4;
        let baked = vertices * 8 + indices * index_bytes;
        println!(
            "| {size} | {} | {:.0} | {:.1} | {:.1} | {vertices} | {indices} | {cache} | {baked} |",
            times.len(),
            sum.as_secs_f64() * 1e6,
            times[times.len() / 2].as_secs_f64() * 1e6,
            times[times.len() - 1].as_secs_f64() * 1e6,
        );
        all_time += sum;
        all_cache += cache;
        all_baked += baked;
    }
    println!(
        "| all | {} | {:.0} | | | | | {all_cache} | {all_baked} |",
        ICONS.len() * sizes.len(),
        all_time.as_secs_f64() * 1e6
    );

    // A cache hit: the copy with offset and tint every request pays,
    // baked or not.
    let _ = icon_path("settings", 20.0, WHITE, 0.0, 0.0);
    let start = Instant::now();
    for i in 0..1000 {
        std::hint::black_box(icon_path("settings", 20.0, WHITE, i as f32, 0.0));
    }
    println!(
        "\na cache hit (`settings` at 20, {} vertices): {:.2} us",
        icon_path("settings", 20.0, WHITE, 0.0, 0.0).map_or(0, |p| p.vertices.len()),
        start.elapsed().as_secs_f64() * 1e6 / 1000.0
    );
}
