//! Every WGSL file of the engine is valid WGSL, checked in the gate
//! (task-72 phase 1). `wgpu-core` parses and validates a WGSL module with
//! naga when the app creates it (`create_shader_module`,
//! wgpu-core-29.0.4/src/device/resource.rs:2278 and :2349), so a broken
//! shader otherwise surfaces in the first run that builds its pipeline.
//! Here the same naga (`wgpu::naga`, the re-export of
//! wgpu-29.0.4/src/lib.rs:172, on every native target) reads every
//! `.wgsl` under `src/` and `examples/` from disk, the files the hot
//! reload of a debug build reads too, and a shader that does not parse or
//! validate fails `cargo test` with its path, line and column.
//!
//! The walk finds the files, so a new shader is checked without being
//! listed anywhere. The validator gets no optional capability: what
//! `wgpu-core` gives a device with no optional features
//! (`create_validator`, wgpu-naga-bridge-29.0.4/src/lib.rs:182), so a
//! shader that passes here needs nothing a device may lack.

use std::path::{Path, PathBuf};

use wgpu::naga;

/// The engine crate's directory.
fn engine() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every `.wgsl` under `dir`, recursively.
fn wgsl_under(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            wgsl_under(&path, out);
        } else if path.extension().is_some_and(|e| e == "wgsl") {
            out.push(path);
        }
    }
}

fn shaders() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for dir in ["src", "examples"] {
        wgsl_under(&engine().join(dir), &mut out);
    }
    out.sort();
    out
}

/// `path` as the message shows it: from the repository root, so an
/// editor opens it as printed.
fn shown(path: &Path) -> String {
    match path.strip_prefix(engine()) {
        Ok(rel) => format!("crates/engine/{}", rel.to_string_lossy().replace('\\', "/")),
        Err(_) => path.display().to_string(),
    }
}

/// Parse and validate `source` as `wgpu-core` does before a backend sees
/// it. The error is naga's report: `path:line:column` and the line when
/// naga knows the place (every parse error, and a validation error that
/// carries a span); a validation error without one (a type, a stage) is
/// prefixed with the path, so the file is always named.
fn check(path: &str, source: &str) -> Result<(), String> {
    let named = |report: String| {
        if report.contains(path) {
            report
        } else {
            format!("{path}: {report}")
        }
    };
    let module = naga::front::wgsl::parse_str(source)
        .map_err(|e| named(e.emit_to_string_with_path(source, path)))?;
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .map_err(|e| named(e.emit_to_string_with_path(source, path)))?;
    Ok(())
}

#[test]
fn every_engine_shader_parses_and_validates() {
    let found = shaders();
    // The nine the engine loads (gpu/context.rs, effects/processor.rs)
    // are among them; the walk may only find more.
    for name in [
        "quad.wgsl",
        "rect_sdf.wgsl",
        "shadow_analytic.wgsl",
        "text.wgsl",
        "image.wgsl",
        "composite.wgsl",
        "backdrop.wgsl",
        "blur.wgsl",
        "shadow.wgsl",
    ] {
        assert!(
            found
                .iter()
                .any(|p| p.ends_with(Path::new("src/gpu/shaders").join(name))),
            "{name} is not under src/gpu/shaders: {found:?}"
        );
    }
    let failures: Vec<String> = found
        .iter()
        .filter_map(|path| match std::fs::read_to_string(path) {
            Ok(source) => check(&shown(path), &source).err(),
            Err(e) => Some(format!("{}: {e}", shown(path))),
        })
        .collect();
    assert!(
        failures.is_empty(),
        "{} of {} shaders are not valid WGSL:\n{}",
        failures.len(),
        found.len(),
        failures.join("\n")
    );
}

/// A missing `;` in a real shader: the report names the file as the
/// repository has it, the line and the column of the token naga stopped
/// at, and shows that line.
#[test]
fn a_syntax_error_names_the_file_the_line_and_the_column() {
    let path = engine().join("src/gpu/shaders/quad.wgsl");
    let source = std::fs::read_to_string(&path).expect("quad.wgsl is on disk");
    let statement = "out.color = in.color;";
    let line = source
        .lines()
        .position(|l| l.trim() == statement)
        .expect("quad.wgsl assigns the color in the vertex stage")
        + 1;
    let broken = source.replacen(statement, "out.color = in.color", 1);

    let report = check(&shown(&path), &broken).expect_err("a missing `;` is a parse error");
    // naga stops at the token after the statement: `return`, first
    // non-blank on the next line.
    let next = source.lines().nth(line).expect("a line follows");
    let column = next.len() - next.trim_start().len() + 1;
    let at = format!(
        "crates/engine/src/gpu/shaders/quad.wgsl:{}:{column}",
        line + 1
    );
    assert!(report.contains(&at), "expected {at} in:\n{report}");
    assert!(report.starts_with("error: "), "{report}");
    assert!(
        report.contains("return out;"),
        "the line is shown:\n{report}"
    );
}

/// A validation error with a place: two uniforms on one binding parse,
/// and the validator names the second declaration's line.
#[test]
fn a_validation_error_names_the_file_and_the_line() {
    let source = "\
@group(0) @binding(0)
var<uniform> scale: f32;
@group(0) @binding(0)
var<uniform> offset: f32;

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(scale + offset);
}
";
    let report = check("shaders/twice.wgsl", source).expect_err("one binding, two resources");
    assert!(report.contains("shaders/twice.wgsl:4:1"), "{report}");
    assert!(report.starts_with("error: "), "{report}");
}

/// A validation error naga reports without a span still names the
/// file: a shader that needs a capability no device is promised (64-bit
/// floats) fails here, not on the device that lacks it.
#[test]
fn a_validation_error_without_a_place_still_names_the_file() {
    let source = "\
var<private> wide: f64;

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0);
}
";
    let report = check("shaders/wide.wgsl", source).expect_err("f64 needs FLOAT64");
    assert!(report.starts_with("shaders/wide.wgsl: error: "), "{report}");
    assert!(report.contains("FLOAT64"), "{report}");
}

#[test]
fn a_valid_shader_passes() {
    let source = "\
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0);
}
";
    assert_eq!(check("shaders/ok.wgsl", source), Ok(()));
}
