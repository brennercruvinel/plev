//! Frame renderer: walks layers (parenting, precomps), resolves the
//! transform hierarchy on the CPU, and emits tessellated paths in
//! bottom-first paint order. Unsupported features log once and are
//! skipped; rendering never panics on unexpected content.

use std::collections::HashSet;

use crate::gem::{self, Mat, Sub};
use crate::kfr::{eval, eval_path, eval1, eval2};
use crate::mdl::{Animation, Layer, Shape, Transform};
use plev::path::TessellatedPath;

pub struct Player {
    pub anim: Animation,
    warned: HashSet<String>,
}

impl Player {
    pub fn new(anim: Animation) -> Self {
        Self {
            anim,
            warned: HashSet::new(),
        }
    }

    /// Render composition frame `t` under a root transform (fit/center).
    pub fn render(&mut self, t: f64, root: Mat) -> Vec<TessellatedPath> {
        let mut out = Vec::new();
        render_layers(
            &self.anim,
            &self.anim.layers,
            t,
            root,
            1.0,
            0,
            &mut out,
            &mut self.warned,
        );
        out
    }
}

fn warn_once(warned: &mut HashSet<String>, msg: &str) {
    if warned.insert(msg.to_string()) {
        log::warn!("lot: {msg}");
    }
}

fn transform_matrix(ks: &Transform, t: f64, warned: &mut HashSet<String>) -> Mat {
    let a = eval2(ks.a.as_ref(), t, [0.0, 0.0]);
    let p = eval2(ks.p.as_ref(), t, [0.0, 0.0]);
    let s = eval2(ks.s.as_ref(), t, [100.0, 100.0]);
    let r = eval1(ks.r.as_ref(), t, 0.0);
    if eval1(ks.sk.as_ref(), t, 0.0).abs() > 1e-3 {
        warn_once(warned, "skew (sk) unsupported, ignored");
    }
    gem::trs(a, p, s, r)
}

/// Composed matrix of a layer and its parent chain (same layer list).
fn layer_matrix(
    layers: &[Layer],
    layer: &Layer,
    t: f64,
    depth: u32,
    warned: &mut HashSet<String>,
) -> Mat {
    let tl = (t - layer.st) / layer.sr.max(1e-9);
    let own = transform_matrix(&layer.ks, tl, warned);
    if depth > 32 {
        return own;
    }
    match layer
        .parent
        .and_then(|pi| layers.iter().find(|l| l.ind == pi))
    {
        Some(par) => layer_matrix(layers, par, t, depth + 1, warned).mul(&own),
        None => own,
    }
}

#[allow(clippy::too_many_arguments)]
fn render_layers(
    anim: &Animation,
    layers: &[Layer],
    t: f64,
    parent_m: Mat,
    parent_alpha: f64,
    depth: u32,
    out: &mut Vec<TessellatedPath>,
    warned: &mut HashSet<String>,
) {
    if depth > 8 {
        warn_once(warned, "precomp nesting deeper than 8, truncated");
        return;
    }
    // layers[0] is topmost; paint bottom-first.
    for layer in layers.iter().rev() {
        if layer.hd || t < layer.ip || t >= layer.op {
            continue;
        }
        let tl = (t - layer.st) / layer.sr.max(1e-9);
        let m = parent_m.mul(&layer_matrix(layers, layer, t, 0, warned));
        let alpha = parent_alpha * (eval1(layer.ks.o.as_ref(), tl, 100.0) / 100.0);
        match layer.ty {
            4 => {
                let mut paths: Vec<Sub> = Vec::new();
                let mut draws: Vec<TessellatedPath> = Vec::new();
                walk_shapes(&layer.shapes, m, alpha, tl, &mut paths, &mut draws, warned);
                // Within a layer the first listed item paints on top.
                out.extend(draws.into_iter().rev());
            }
            0 => {
                let Some(asset) = layer.ref_id.as_deref().and_then(|id| anim.asset(id)) else {
                    warn_once(warned, "precomp with missing asset, skipped");
                    continue;
                };
                render_layers(anim, &asset.layers, tl, m, alpha, depth + 1, out, warned);
            }
            3 => {} // null: transform-only, consumed via parenting
            other => {
                warn_once(warned, &format!("layer type {other} unsupported, skipped"));
            }
        }
    }
}

fn style_color(s: &Shape, alpha: f64, t: f64) -> [f32; 4] {
    let c = s.c.as_ref().map(|p| eval(p, t)).unwrap_or_default();
    let a = (eval1(s.o.as_ref(), t, 100.0) / 100.0 * alpha).clamp(0.0, 1.0);
    [
        c.first().copied().unwrap_or(0.5) as f32,
        c.get(1).copied().unwrap_or(0.5) as f32,
        c.get(2).copied().unwrap_or(0.5) as f32,
        a as f32,
    ]
}

/// Solid approximation of a gradient: average of the rgb stops.
fn grad_color(s: &Shape, alpha: f64, t: f64) -> [f32; 4] {
    let stops =
        s.g.as_ref()
            .and_then(|g| g.k.as_ref())
            .map(|p| eval(p, t))
            .unwrap_or_default();
    let n = s.g.as_ref().map(|g| g.p.max(1) as usize).unwrap_or(1);
    let (mut r, mut gr, mut b, mut cnt) = (0.0, 0.0, 0.0, 0.0);
    for i in 0..n {
        let base = i * 4;
        if base + 3 < stops.len() {
            r += stops[base + 1];
            gr += stops[base + 2];
            b += stops[base + 3];
            cnt += 1.0;
        }
    }
    if cnt == 0.0 {
        return [0.5, 0.5, 0.5, alpha as f32];
    }
    let a = (eval1(s.o.as_ref(), t, 100.0) / 100.0 * alpha).clamp(0.0, 1.0);
    [
        (r / cnt) as f32,
        (gr / cnt) as f32,
        (b / cnt) as f32,
        a as f32,
    ]
}

#[allow(clippy::too_many_arguments)]
fn walk_shapes(
    items: &[Shape],
    m: Mat,
    alpha: f64,
    t: f64,
    paths: &mut Vec<Sub>,
    draws: &mut Vec<TessellatedPath>,
    warned: &mut HashSet<String>,
) {
    for item in items {
        if item.hd {
            continue;
        }
        match item.ty.as_str() {
            "gr" => {
                let (m2, a2) = match item.it.iter().find(|s| s.ty == "tr") {
                    Some(tr) => {
                        let anchor = eval2(tr.a.as_ref(), t, [0.0, 0.0]);
                        let pos = eval2(tr.p.as_ref(), t, [0.0, 0.0]);
                        let sc = eval2(tr.s.as_ref(), t, [100.0, 100.0]);
                        let rot = eval1(tr.r.as_ref(), t, 0.0);
                        if eval1(tr.sk.as_ref(), t, 0.0).abs() > 1e-3 {
                            warn_once(warned, "skew (sk) unsupported, ignored");
                        }
                        let a2 = alpha * eval1(tr.o.as_ref(), t, 100.0) / 100.0;
                        (m.mul(&gem::trs(anchor, pos, sc, rot)), a2)
                    }
                    None => (m, alpha),
                };
                let mut child = Vec::new();
                walk_shapes(&item.it, m2, a2, t, &mut child, draws, warned);
                // Styles in outer groups also cover inner geometry.
                paths.append(&mut child);
            }
            "sh" => {
                if let Some(pd) = item.ks.as_ref().and_then(|p| eval_path(p, t))
                    && let Some(sub) = gem::path_to_sub(&pd, &m)
                {
                    paths.push(sub);
                }
            }
            "el" => {
                let p = eval2(item.p.as_ref(), t, [0.0, 0.0]);
                let s = eval2(item.s.as_ref(), t, [0.0, 0.0]);
                paths.push(gem::ellipse_sub(p, s, &m));
            }
            "rc" => {
                let p = eval2(item.p.as_ref(), t, [0.0, 0.0]);
                let s = eval2(item.s.as_ref(), t, [0.0, 0.0]);
                if eval1(item.r.as_ref(), t, 0.0) > 1e-3 {
                    warn_once(warned, "rect roundness unsupported, square corners");
                }
                paths.push(gem::rect_sub(p, s, &m));
            }
            "fl" => {
                if !paths.is_empty() {
                    draws.push(gem::tess_fill(paths, style_color(item, alpha, t)));
                }
            }
            "st" => {
                if !paths.is_empty() {
                    let w = eval1(item.w.as_ref(), t, 1.0) * m.scale_factor();
                    draws.push(gem::tess_stroke(
                        paths,
                        style_color(item, alpha, t),
                        w.max(0.1) as f32,
                    ));
                }
            }
            "gf" => {
                warn_once(warned, "gradient fill (gf) approximated as solid color");
                if !paths.is_empty() {
                    draws.push(gem::tess_fill(paths, grad_color(item, alpha, t)));
                }
            }
            "gs" => {
                warn_once(warned, "gradient stroke (gs) approximated as solid color");
                if !paths.is_empty() {
                    let w = eval1(item.w.as_ref(), t, 1.0) * m.scale_factor();
                    draws.push(gem::tess_stroke(
                        paths,
                        grad_color(item, alpha, t),
                        w.max(0.1) as f32,
                    ));
                }
            }
            "tr" => {} // group transform, applied above
            other => {
                warn_once(
                    warned,
                    &format!("shape type '{other}' unsupported, skipped"),
                );
            }
        }
    }
}
