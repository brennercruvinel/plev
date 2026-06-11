//! lottie json scan for the dense benches (`dense.rs`): a generic walk
//! over one layer collecting animated numeric properties (keyframe
//! boundaries, values, outgoing easing), path morph keyframes (vertex
//! counts), static path weights, paint counts and the text document
//! weight. mapping honesty notes live in dense.rs next to the tests.

use anm::Easing;
use serde_json::Value as J;

pub fn f(v: &J) -> f32 {
    v.as_f64().unwrap_or(0.0) as f32
}

/// First component of a number-or-array easing coordinate.
fn num1(v: &J) -> f32 {
    match v {
        J::Array(a) => a.first().map(f).unwrap_or(0.0),
        _ => f(v),
    }
}

/// One numeric keyframe: time in seconds (instance offset applied),
/// value vector as authored, easing toward the next keyframe.
pub struct NumKf {
    pub t: f32,
    pub v: Vec<f32>,
    pub ez: Easing,
}

/// One path morph keyframe: time in seconds and the vertex count of
/// the shape it lands on.
pub struct MorphKf {
    pub t: f32,
    pub verts: usize,
}

/// Everything one layer contributes to the IR mapping. `ip_s`/`op_s`
/// are the visibility window in seconds, clamped to the composition.
#[derive(Default)]
pub struct Scan {
    pub ip_s: f32,
    pub op_s: f32,
    pub nums: Vec<Vec<NumKf>>,
    pub morphs: Vec<Vec<MorphKf>>,
    pub static_paths: Vec<usize>,
    pub paints: usize,
    pub text_bytes: usize,
}

/// Outgoing bezier of `kf`, snapped to the linear and ae-default
/// presets when it matches (the starfish bench precedent), control
/// points clamped to the wire range of `anm::quant`.
fn ease_of(kf: &J) -> Easing {
    if f(&kf["h"]) == 1.0 {
        return Easing::Hold;
    }
    let (o, i) = (&kf["o"], &kf["i"]);
    if o.is_null() || i.is_null() {
        return Easing::Linear;
    }
    let (x1, y1) = (num1(&o["x"]), num1(&o["y"]));
    let (x2, y2) = (num1(&i["x"]), num1(&i["y"]));
    let near = |a: f32, b: f32| (a - b).abs() < 1e-3;
    if near(x1, 0.0) && near(y1, 0.0) && near(x2, 1.0) && near(y2, 1.0) {
        Easing::Linear
    } else if near(x1, 0.167) && near(y1, 0.167) && near(x2, 0.833) && near(y2, 0.833) {
        Easing::EaseInOut
    } else {
        Easing::CustomBezier {
            x1: x1.clamp(0.0, 1.0),
            y1: y1.clamp(-0.5, 1.5),
            x2: x2.clamp(0.0, 1.0),
            y2: y2.clamp(-0.5, 1.5),
        }
    }
}

/// Scan one layer: `fr` frames per second, `t_off` seconds of precomp
/// instance offset, visibility clamped to `[lo, hi]` seconds.
pub fn scan_layer(layer: &J, fr: f32, t_off: f32, lo: f32, hi: f32) -> Scan {
    let mut out = Scan {
        ip_s: (t_off + f(&layer["ip"]) / fr).clamp(lo, hi),
        op_s: (t_off + f(&layer["op"]) / fr).clamp(lo, hi),
        ..Scan::default()
    };
    if f(&layer["ty"]) == 5.0 {
        let s = &layer["t"]["d"]["k"][0]["s"];
        let len = |v: &J| v.as_str().map_or(0, str::len);
        out.text_bytes = len(&s["t"]) + len(&s["f"]) + 8;
    }
    walk(layer, fr, t_off, &mut out);
    out
}

/// Is this an animated numeric property: `a == 1` and `k` a keyframe
/// list whose entries carry a time and a numeric value vector?
fn numeric_kfs(v: &J) -> Option<&Vec<J>> {
    if f(&v["a"]) != 1.0 {
        return None;
    }
    let kfs = v["k"].as_array()?;
    let first = kfs.first()?;
    (first["t"].is_number() && first["s"][0].is_number()).then_some(kfs)
}

fn walk(v: &J, fr: f32, t_off: f32, out: &mut Scan) {
    let J::Object(map) = v else {
        if let J::Array(items) = v {
            for item in items {
                walk(item, fr, t_off, out);
            }
        }
        return;
    };
    let ty = map.get("ty").and_then(J::as_str);
    if ty == Some("sh") {
        let ks = &v["ks"];
        if f(&ks["a"]) == 1.0 {
            let kfs = ks["k"].as_array().map_or(Vec::new(), |kfs| {
                kfs.iter()
                    .map(|kf| MorphKf {
                        t: t_off + f(&kf["t"]) / fr,
                        verts: kf["s"][0]["v"].as_array().map_or(0, Vec::len),
                    })
                    .collect()
            });
            out.morphs.push(kfs);
        } else {
            let verts = ks["k"]["v"].as_array().map_or(0, Vec::len);
            out.static_paths.push(verts);
        }
        return;
    }
    if matches!(ty, Some("fl" | "st" | "gf" | "gs")) {
        // static paint weight; an animated color inside the paint is
        // still collected by the numeric branch while descending.
        out.paints += 1;
    }
    if let Some(kfs) = numeric_kfs(v) {
        out.nums.push(
            kfs.iter()
                .map(|kf| NumKf {
                    t: t_off + f(&kf["t"]) / fr,
                    v: kf["s"]
                        .as_array()
                        .map_or(Vec::new(), |s| s.iter().map(f).collect()),
                    ez: ease_of(kf),
                })
                .collect(),
        );
        return;
    }
    for item in map.values() {
        walk(item, fr, t_off, out);
    }
}
