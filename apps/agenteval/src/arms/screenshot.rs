//! arm C, the volume of a screenshot: the same workbench, the same
//! repository, drawn without a window at scale 1 and 2, written as PNG,
//! with the bytes of each frame (as PNG and as raw RGBA) beside the bytes
//! of what an agent receives for the same state: a whole observation and
//! one scoped to `changes`, serialized exactly as the server writes them.
//!
//! it compares volume of data, not capability. a PNG carries pixels a
//! model must read; an observation carries the identity, the state and
//! the actions of every control. the report says so, and says nothing
//! about which is better at the task.

use std::path::Path;

use engine::drawing::headless::{Renderer, Viewport};
use accord::{SemId, Snapshot};
use serde_json::json;
use socket::{Reply, ServerMsg, SnapshotReply, encode_server};

use crate::args::Args;
use crate::bench::{self, Bench};
use crate::repo::TASK;
use crate::report::{self, DONE, ERROR, row};

/// the scales the frame is drawn at: a window and a retina window.
const SCALES: [f32; 2] = [1.0, 2.0];

pub fn run(args: &Args) -> i32 {
    match arm(args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("agenteval: {e}");
            ERROR
        }
    }
}

/// one drawn frame: where it was written and what it weighs.
struct Shot {
    scale: f32,
    width: u32,
    height: u32,
    png: usize,
    rgba: usize,
    path: String,
}

fn arm(args: &Args) -> Result<i32, String> {
    std::fs::create_dir_all(&args.out).map_err(|e| format!("{}: {e}", args.out.display()))?;
    // no port and no recorder here: the workbench is opened only to be
    // drawn, and the renderer of each scale is made below.
    let mut app = Bench::open(false)?;
    let snapshot = app
        .bench
        .snapshot()
        .map_err(|e| format!("the projection does not validate: {e}"))?;

    // one renderer, resized between the two scales, as a window is when
    // it moves to another display: a second renderer would be a second
    // GPU device, and the compositor's uploaded buffers belong to the
    // first one.
    let mut shots = Vec::new();
    match bench::headless(SCALES[0])? {
        Some(mut gpu) => {
            for scale in SCALES {
                if gpu.viewport().scale != scale {
                    let viewport = Viewport::new(bench::WIDTH, bench::HEIGHT, scale);
                    gpu.resize(viewport)
                        .map_err(|e| format!("the viewport at scale {scale}: {e}"))?;
                }
                shots.push(shoot(&mut app, &mut gpu, scale, &args.out)?);
            }
        }
        None => println!("agenteval: no GPU adapter; no frame was drawn"),
    }

    let whole = observation_bytes(&snapshot, None);
    let changes = SemId::parse("changes").map_err(|e| e.to_string())?;
    let scoped = observation_bytes(&snapshot, Some(&changes));
    let nodes = snapshot.len();

    let report = json!({
        "arm": "screenshot",
        "task": TASK,
        "viewport": { "width": bench::WIDTH, "height": bench::HEIGHT },
        "frames": shots.iter().map(|s| json!({
            "scale": s.scale,
            "width": s.width,
            "height": s.height,
            "png_bytes": s.png,
            "rgba_bytes": s.rgba,
            "path": s.path,
        })).collect::<Vec<_>>(),
        "observation": {
            "nodes": nodes,
            "whole_bytes": whole,
            "scope_changes_bytes": scoped,
        },
        "receipts": "none",
        "session": "none",
        "note": "volume of data, not capability",
        "exit": DONE,
    });
    report::write(&args.out, &report)?;

    let mut rows = Vec::new();
    for s in &shots {
        rows.push(row(
            &format!("frame at scale {} ({}x{})", s.scale, s.width, s.height),
            &format!("{} bytes PNG, {} bytes raw RGBA, {}", s.png, s.rgba, s.path),
        ));
    }
    if shots.is_empty() {
        rows.push(row("frame", "no GPU adapter on this machine"));
    }
    rows.push(row(
        &format!("plev observe ({nodes} nodes)"),
        &format!("{whole} bytes"),
    ));
    rows.push(row(
        "plev observe --scope changes",
        &format!("{scoped} bytes"),
    ));
    if let Some(s) = shots.first() {
        rows.push(row(
            "the whole observation against the PNG at scale 1",
            &format!("{:.1}x smaller", s.png as f64 / whole.max(1) as f64),
        ));
    }
    println!();
    report::print_table("volume of data, not capability", &rows);
    println!("report: {}", args.out.join("report.json").display());
    Ok(DONE)
}

/// draws the workbench at `scale` and writes `frame@<scale>.png`.
fn shoot(app: &mut Bench, gpu: &mut Renderer, scale: f32, out: &Path) -> Result<Shot, String> {
    app.compositor.begin_frame();
    app.bench.render(&mut app.compositor);
    let clear = app.clear();
    let frame = gpu
        .render(&mut app.compositor, clear)
        .map_err(|e| format!("the frame at scale {scale}: {e}"))?;
    let png = frame
        .encode_png()
        .map_err(|e| format!("the PNG at scale {scale}: {e}"))?;
    let path = out.join(format!("frame@{scale:.0}.png"));
    std::fs::write(&path, &png).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Shot {
        scale,
        width: frame.width,
        height: frame.height,
        png: png.len(),
        rgba: frame.rgba.len(),
        path: path.display().to_string(),
    })
}

/// the bytes of the line the server writes for one observation of
/// `snapshot`, whole or under `scope`: the same `Reply` through the same
/// codec, `source` left out as the port leaves it out.
fn observation_bytes(snapshot: &Snapshot, scope: Option<&SemId>) -> usize {
    let seen = match scope {
        Some(id) => snapshot.subtree(id),
        None => snapshot.clone(),
    };
    let reply = Reply::Snapshot(SnapshotReply {
        revision: snapshot.revision(),
        nodes: seen
            .nodes()
            .values()
            .map(|node| {
                let mut node = node.clone();
                node.source = None;
                node
            })
            .collect(),
        focused: seen.focused().cloned(),
        bounds: None,
    });
    encode_server(&ServerMsg {
        id: Some("1".to_owned()),
        reply,
    })
    .len()
        + 1
}
