//! A report as text: one line for a baseline that held; for one that
//! moved, each control that changed (meaning, then drawing), the layers
//! that moved and what their pixels say.

use crate::compare::Status;

use super::{ControlReport, Report, Visual};

/// The report as text: the baseline and its timings, then [`body`].
pub fn text(r: &Report) -> String {
    let t = &r.timings_ms;
    let mut times = format!(
        "stage {:.1} ms, hash {:.1} ms, compare {:.1} ms",
        t.stage, t.capture, t.compare
    );
    for (what, v) in [("render", t.render), ("decode", t.decode), ("diff", t.diff)] {
        if let Some(v) = v {
            times.push_str(&format!(", {what} {v:.1} ms"));
        }
    }
    let head = if r.changed { "CHANGED " } else { "ok      " };
    format!("{head} {} ({times})\n{}", r.baseline, body(r))
}

/// Each control that changed, the layers that moved and what their
/// pixels say; empty when nothing moved.
pub fn body(r: &Report) -> String {
    let mut out = String::new();
    if !r.changed {
        return out;
    }
    for line in &r.page {
        out.push_str(&format!("  (page): {line}\n"));
    }
    let listed = r.controls.iter().filter(|c| c.changed());
    let width = listed.clone().map(|c| c.id.len()).max().unwrap_or(0);
    for c in listed {
        let meaning = if c.semantics.is_empty() {
            "semantics unchanged".to_string()
        } else {
            format!("semantics changed: {}", c.semantics.join("; "))
        };
        out.push_str(&format!("  {:width$}  {meaning}; {}\n", c.id, visual(c)));
    }
    let quiet = r.controls.iter().filter(|c| !c.changed()).count();
    if quiet > 0 {
        out.push_str(&format!("  {quiet} other controls unchanged\n"));
    }
    for l in r.layers.iter().filter(|l| l.status != Status::Unchanged) {
        let status = match l.status {
            Status::Changed => "changed",
            Status::Added => "added",
            Status::Removed => "removed",
            Status::Unchanged => "unchanged",
        };
        let hash = match (&l.was, &l.hash) {
            (Some(was), Some(now)) => format!(" ({was} -> {now})"),
            _ => String::new(),
        };
        let mut line = format!("  layer {}: {status}{hash}", l.name);
        if let Some(px) = l.pixels {
            line.push_str(&format!(", {px} px{}", at(l.region)));
        }
        if let Some(px) = l.unbound_pixels.filter(|px| *px > 0) {
            line.push_str(&format!(
                "; {px} px outside any bound control{}",
                at(l.unbound_region)
            ));
        }
        out.push_str(&line);
        out.push('\n');
    }
    match r.pixels.status {
        "diffed" => out.push_str(&format!(
            "  pixels: {} changed of {} compared{}\n",
            r.pixels.changed.unwrap_or(0),
            r.pixels.compared.unwrap_or(0),
            at(r.pixels.region)
        )),
        "no_baseline_image" => out.push_str(
            "  pixels: visual changed, no baseline image to diff (bless --png keeps one)\n",
        ),
        "no_adapter" => out.push_str(
            "  pixels: visual changed, no GPU adapter here to draw the frame; not compared\n",
        ),
        _ => {}
    }
    out
}

fn visual(c: &ControlReport) -> String {
    let layer = c
        .layer
        .as_deref()
        .map_or(String::new(), |l| format!(", layer {l}"));
    let moved = match c.moved {
        Some([None, Some(now)]) => format!(", newly bound at {}", region(now)),
        Some([Some(was), None]) => format!(", no longer bound (was {})", region(was)),
        Some([Some(was), Some(now)]) => format!(", bound {} -> {}", region(was), region(now)),
        _ => String::new(),
    };
    match c.visual {
        Visual::Unchanged => format!("visual unchanged{layer}"),
        Visual::Unbound => "visual not bound".to_string(),
        Visual::LayerChanged => {
            format!("visual changed{layer} (the layer moved; its pixels were not compared){moved}")
        }
        Visual::Changed => match c.pixels {
            Some(px) if px > 0 => format!("visual changed{layer}, {px} px{}{moved}", at(c.region)),
            _ => format!("visual changed{layer}{moved}"),
        },
    }
}

fn at(r: Option<[f32; 4]>) -> String {
    r.map_or(String::new(), |r| format!(" at {}", region(r)))
}

/// A logical rect as `x,y wxh`.
pub fn region(r: [f32; 4]) -> String {
    format!("{},{} {}x{}", r[0], r[1], r[2], r[3])
}
