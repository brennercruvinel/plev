//! Keyframe evaluation: vector values and bezier shape paths, with
//! cubic-bezier temporal easing (Newton solve of x(t)=u, bisection-free
//! with clamping; 8 iterations is plenty at frame granularity).

use crate::mdl::Prop;
use serde_json::Value;

fn to_vec(v: &Value) -> Vec<f64> {
    match v {
        Value::Number(n) => vec![n.as_f64().unwrap_or(0.0)],
        Value::Array(a) => a.iter().filter_map(Value::as_f64).collect(),
        _ => Vec::new(),
    }
}

fn is_keyframes(k: &Value) -> bool {
    matches!(k, Value::Array(a) if a.first().is_some_and(Value::is_object))
}

fn kf_t(k: &Value) -> f64 {
    k.get("t").and_then(Value::as_f64).unwrap_or(0.0)
}

/// First numeric component of a scalar-or-array easing handle.
fn handle(k: &Value, key: &str, axis: &str, default: f64) -> f64 {
    let Some(h) = k.get(key).and_then(|h| h.get(axis)) else {
        return default;
    };
    match h {
        Value::Number(n) => n.as_f64().unwrap_or(default),
        Value::Array(a) => a.first().and_then(Value::as_f64).unwrap_or(default),
        _ => default,
    }
}

fn bez(c1: f64, c2: f64, s: f64) -> f64 {
    let r = 1.0 - s;
    3.0 * c1 * s * r * r + 3.0 * c2 * s * s * r + s * s * s
}

fn bez_d(c1: f64, c2: f64, s: f64) -> f64 {
    let r = 1.0 - s;
    3.0 * r * r * c1 + 6.0 * r * s * (c2 - c1) + 3.0 * s * s * (1.0 - c2)
}

/// Eased progress for keyframe `k0` at linear progress `u` in [0,1].
pub fn ease_factor(k0: &Value, u: f64) -> f64 {
    if k0.get("h").and_then(Value::as_i64) == Some(1) {
        return 0.0;
    }
    let ox = handle(k0, "o", "x", 0.167).clamp(0.0, 1.0);
    let oy = handle(k0, "o", "y", 0.167);
    let ix = handle(k0, "i", "x", 0.833).clamp(0.0, 1.0);
    let iy = handle(k0, "i", "y", 0.833);
    // Solve bez(ox, ix, s) = u for s with Newton, clamped.
    let mut s = u;
    for _ in 0..8 {
        let err = bez(ox, ix, s) - u;
        if err.abs() < 1e-5 {
            break;
        }
        let d = bez_d(ox, ix, s);
        if d.abs() < 1e-6 {
            break;
        }
        s = (s - err / d).clamp(0.0, 1.0);
    }
    bez(oy, iy, s)
}

fn lerp_vec(a: &[f64], b: &[f64], e: f64) -> Vec<f64> {
    a.iter()
        .enumerate()
        .map(|(i, &x)| {
            let y = b.get(i).copied().unwrap_or(x);
            x + (y - x) * e
        })
        .collect()
}

fn start_val(k: &Value) -> Vec<f64> {
    k.get("s").map(to_vec).unwrap_or_default()
}

fn end_val(k0: &Value, k1: &Value) -> Vec<f64> {
    if let Some(e) = k0.get("e") {
        return to_vec(e);
    }
    start_val(k1)
}

/// Evaluate a (possibly animated) numeric property at frame `t`.
pub fn eval(prop: &Prop, t: f64) -> Vec<f64> {
    if !is_keyframes(&prop.k) {
        return to_vec(&prop.k);
    }
    let kfs = prop.k.as_array().expect("checked array");
    if t <= kf_t(&kfs[0]) {
        return start_val(&kfs[0]);
    }
    for w in kfs.windows(2) {
        let (t0, t1) = (kf_t(&w[0]), kf_t(&w[1]));
        if t < t1 {
            let v0 = start_val(&w[0]);
            let v1 = end_val(&w[0], &w[1]);
            if v1.is_empty() {
                return v0;
            }
            let u = ((t - t0) / (t1 - t0).max(1e-9)).clamp(0.0, 1.0);
            return lerp_vec(&v0, &v1, ease_factor(&w[0], u));
        }
    }
    // Past the last keyframe: hold its value (or the previous end).
    let last = kfs.last().expect("non-empty");
    let v = start_val(last);
    if !v.is_empty() {
        return v;
    }
    match kfs.len() {
        n if n >= 2 => end_val(&kfs[n - 2], last),
        _ => Vec::new(),
    }
}

/// Scalar convenience over [`eval`].
pub fn eval1(prop: Option<&Prop>, t: f64, default: f64) -> f64 {
    prop.and_then(|p| eval(p, t).first().copied())
        .unwrap_or(default)
}

/// 2-component convenience over [`eval`]; a 1-component value broadcasts.
pub fn eval2(prop: Option<&Prop>, t: f64, default: [f64; 2]) -> [f64; 2] {
    let Some(p) = prop else { return default };
    let v = eval(p, t);
    match v.len() {
        0 => default,
        1 => [v[0], v[0]],
        _ => [v[0], v[1]],
    }
}

/// Bezier path data: vertices plus in/out tangents (relative to the vertex).
#[derive(Clone)]
pub struct PathData {
    pub v: Vec<[f64; 2]>,
    pub i: Vec<[f64; 2]>,
    pub o: Vec<[f64; 2]>,
    pub c: bool,
}

fn pts(v: &Value, key: &str) -> Vec<[f64; 2]> {
    v.get(key)
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .map(|p| {
                    let q = to_vec(p);
                    [
                        q.first().copied().unwrap_or(0.0),
                        q.get(1).copied().unwrap_or(0.0),
                    ]
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_path(v: &Value) -> Option<PathData> {
    // Shape keyframe payloads wrap the object in a 1-element array.
    let obj = match v {
        Value::Array(a) => a.first()?,
        other => other,
    };
    if !obj.is_object() || obj.get("v").is_none() {
        return None;
    }
    Some(PathData {
        v: pts(obj, "v"),
        i: pts(obj, "i"),
        o: pts(obj, "o"),
        c: obj.get("c").and_then(Value::as_bool).unwrap_or(false),
    })
}

fn lerp_pts(a: &[[f64; 2]], b: &[[f64; 2]], e: f64) -> Vec<[f64; 2]> {
    a.iter()
        .enumerate()
        .map(|(j, p)| {
            let q = b.get(j).copied().unwrap_or(*p);
            [p[0] + (q[0] - p[0]) * e, p[1] + (q[1] - p[1]) * e]
        })
        .collect()
}

/// Evaluate a (possibly animated) bezier path property at frame `t`.
pub fn eval_path(prop: &Prop, t: f64) -> Option<PathData> {
    if !is_keyframes(&prop.k) {
        return parse_path(&prop.k);
    }
    let kfs = prop.k.as_array().expect("checked array");
    let seg_at = |k: &Value| k.get("s").and_then(parse_path);
    if t <= kf_t(&kfs[0]) {
        return seg_at(&kfs[0]);
    }
    for w in kfs.windows(2) {
        let (t0, t1) = (kf_t(&w[0]), kf_t(&w[1]));
        if t < t1 {
            let p0 = seg_at(&w[0])?;
            let p1 = w[0]
                .get("e")
                .and_then(parse_path)
                .or_else(|| seg_at(&w[1]));
            let Some(p1) = p1 else { return Some(p0) };
            let u = ((t - t0) / (t1 - t0).max(1e-9)).clamp(0.0, 1.0);
            let e = ease_factor(&w[0], u);
            return Some(PathData {
                v: lerp_pts(&p0.v, &p1.v, e),
                i: lerp_pts(&p0.i, &p1.i, e),
                o: lerp_pts(&p0.o, &p1.o, e),
                c: p0.c,
            });
        }
    }
    let last = kfs.last().expect("non-empty");
    seg_at(last).or_else(|| {
        (kfs.len() >= 2)
            .then(|| kfs[kfs.len() - 2].get("e").and_then(parse_path))
            .flatten()
    })
}
