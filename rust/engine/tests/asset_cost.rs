//! What the engine parses, validates and decodes at runtime, measured
//! (task-72 phase 1, the probe behind the inventory in
//! docs/mission/steps/checked/task-72-ui-asset-compiler-done.md): each WGSL
//! shader through naga (the parse and the validation `wgpu-core` runs in
//! `create_shader_module` before the backend sees the module), the
//! phases of a headless renderer (device, the context's pipelines, the
//! text system, the effects), the embedded fonts through fontdb and a
//! font system as the text system and the measurer build one, and the one
//! PNG of the engine's assets through `image`.
//!
//! ```text
//! cargo test -p engine --test asset_cost -- --ignored --nocapture
//! ```
//!
//! The numbers are the debug profile's (the workspace at opt-level 0, its
//! dependencies at 2, `[profile.dev.package."*"]`); a release build
//! measures lower and is not what this prints.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use engine::drawing::effects::EffectProcessor;
use engine::drawing::gpu::{GpuContext, RenderConfig};
use engine::runtime::perf::process_rss_bytes;
use engine::drawing::text::{TextMeasurer, TextSystem};
use wgpu::naga;

/// Repetitions of a CPU step; the best one is reported.
const ROUNDS: usize = 20;

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The files in `dir` with extension `ext`, sorted by name.
fn files(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == ext))
        .collect();
    out.sort();
    out
}

fn name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The fastest of `ROUNDS` runs of `f`.
fn best<T>(mut f: impl FnMut() -> T) -> Duration {
    (0..ROUNDS)
        .map(|_| {
            let start = Instant::now();
            std::hint::black_box(f());
            start.elapsed()
        })
        .min()
        .unwrap_or_default()
}

fn us(d: Duration) -> f64 {
    d.as_secs_f64() * 1e6
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1e3
}

fn mib(bytes: i64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

fn rss() -> i64 {
    process_rss_bytes().map_or(0, |b| b as i64)
}

/// Every `.wgsl` the engine ships, parsed and validated as `wgpu-core`
/// does before the backend translates it.
fn shaders() {
    println!("\n## shaders (naga on the CPU, best of {ROUNDS})\n");
    println!("| shader | bytes | parse (us) | validate (us) |");
    println!("| --- | ---: | ---: | ---: |");
    let (mut bytes, mut parse, mut validate) = (0, Duration::ZERO, Duration::ZERO);
    for path in files(&manifest().join("src/gpu/shaders"), "wgsl") {
        let source = std::fs::read_to_string(&path).unwrap_or_default();
        let module = naga::front::wgsl::parse_str(&source)
            .unwrap_or_else(|e| panic!("{}", e.emit_to_string_with_path(&source, &path)));
        let p = best(|| naga::front::wgsl::parse_str(&source).is_ok());
        let v = best(|| {
            naga::valid::Validator::new(
                naga::valid::ValidationFlags::all(),
                naga::valid::Capabilities::empty(),
            )
            .validate(&module)
            .is_ok()
        });
        println!(
            "| {} | {} | {:.0} | {:.0} |",
            name(&path),
            source.len(),
            us(p),
            us(v)
        );
        bytes += source.len();
        parse += p;
        validate += v;
    }
    println!(
        "| total | {bytes} | {:.0} | {:.0} |",
        us(parse),
        us(validate)
    );
}

/// A device on the headless context's backends, with nothing on it.
fn bare_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()
}

/// The phases of `engine::headless::Renderer::new`, one by one, in a
/// process that opened no device before: the context's constructor is
/// the device plus the seven pipelines of `GpuContext::from_device`.
fn renderer() {
    println!("\n## the headless renderer, phase by phase (one run each)\n");
    let t = Instant::now();
    let Some(first) = bare_device() else {
        println!("no GPU adapter; skipped");
        return;
    };
    let cold = t.elapsed();
    drop(first);
    let t = Instant::now();
    let warm = bare_device();
    let warm_t = t.elapsed();
    drop(warm);

    let before = rss();
    let t = Instant::now();
    let Some(gpu) =
        pollster::block_on(GpuContext::new_headless(1200, 800, RenderConfig::default()))
    else {
        println!("no GPU adapter; skipped");
        return;
    };
    let context = t.elapsed();
    let after_context = rss();
    let t = Instant::now();
    let text = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
    let text_t = t.elapsed();
    let after_text = rss();
    let t = Instant::now();
    let effects = EffectProcessor::new(&gpu.device, gpu.surface_format());
    let effects_t = t.elapsed();
    let after_effects = rss();
    drop((text, effects));

    println!("| phase | ms | rss delta (MiB) |");
    println!("| --- | ---: | ---: |");
    println!("| bare device, first in the process | {:.1} | |", ms(cold));
    println!("| bare device, again | {:.1} | |", ms(warm_t));
    println!(
        "| `GpuContext::new_headless` (device + 7 pipelines) | {:.1} | {:.1} |",
        ms(context),
        mib(after_context - before)
    );
    println!(
        "| `TextSystem::new` (font system + atlas) | {:.1} | {:.1} |",
        ms(text_t),
        mib(after_text - after_context)
    );
    println!(
        "| `EffectProcessor::new` (4 pipelines) | {:.1} | {:.1} |",
        ms(effects_t),
        mib(after_effects - after_text)
    );
}

/// The embedded faces (`src/text/fonts.rs` registers every `.ttf` of
/// `assets/fonts`), a font system as the text system builds one, and the
/// measurer's own on a thread of its own.
fn fonts() {
    println!("\n## fonts (fontdb, best of {ROUNDS})\n");
    println!("| face | bytes | load_font_data (us) |");
    println!("| --- | ---: | ---: |");
    let faces: Vec<(String, Vec<u8>)> = files(&manifest().join("assets/fonts"), "ttf")
        .iter()
        .map(|p| (name(p), std::fs::read(p).unwrap_or_default()))
        .collect();
    let (mut bytes, mut load) = (0, Duration::ZERO);
    for (face, data) in &faces {
        let d = best(|| {
            let mut db = cosmic_text::fontdb::Database::new();
            db.load_font_data(data.clone());
            db.len()
        });
        println!("| {face} | {} | {:.0} |", data.len(), us(d));
        bytes += data.len();
        load += d;
    }
    println!("| total | {bytes} | {:.0} |", us(load));

    for round in ["first in the process", "again"] {
        let before = rss();
        let t = Instant::now();
        let system = cosmic_text::FontSystem::new();
        let scan = t.elapsed();
        let after = rss();
        println!(
            "\n`FontSystem::new()`, {round} (the desktop's system faces, before \
             the embedded ones): {:.1} ms, {} faces, rss delta {:.1} MiB",
            ms(scan),
            system.db().len(),
            mib(after - before)
        );
    }

    let first = std::thread::spawn(|| {
        let t = Instant::now();
        TextMeasurer::measure("Theme", 14.0, None);
        t.elapsed()
    })
    .join()
    .unwrap_or_default();
    println!(
        "first `TextMeasurer::measure` on a fresh thread (its font system, \
         system and embedded faces, and one shaping): {:.1} ms",
        ms(first)
    );
}

/// The one PNG of the engine's assets, decoded as `load_image_bytes`
/// decodes (`image::load_from_memory`).
fn images() {
    println!("\n## images (image, best of {ROUNDS})\n");
    println!("| image | bytes | decode (us) | rgba bytes |");
    println!("| --- | ---: | ---: | ---: |");
    for path in files(&manifest().join("assets"), "png") {
        let data = std::fs::read(&path).unwrap_or_default();
        let Ok(decoded) = image::load_from_memory(&data) else {
            println!("| {} | {} | undecodable | |", name(&path), data.len());
            continue;
        };
        let d = best(|| image::load_from_memory(&data).is_ok());
        println!(
            "| {} | {} | {:.0} | {} |",
            name(&path),
            data.len(),
            us(d),
            decoded.width() as usize * decoded.height() as usize * 4
        );
    }
}

#[test]
#[ignore = "a measurement: cargo test -p engine --test asset_cost -- --ignored --nocapture"]
fn what_the_runtime_parses_validates_and_decodes() {
    fonts();
    renderer();
    shaders();
    images();
}
