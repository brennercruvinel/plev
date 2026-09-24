//! Two frames of one page, theme, viewport and scale, drawn by two
//! targets or twice by one, and what their difference is (task-77):
//! equal in every bit, different where a decision of the product says
//! so, or different where nothing says so, measured.
//!
//! A pixel's delta is the largest of its four channel differences, 0 to
//! 255. A differing pixel is explained when its center falls in the
//! area of a declared decision ([`Decision`], read from the table of
//! requested differences); the verdict is [`Verdict::Requested`] when
//! every differing pixel is explained, [`Verdict::Silent`] when one is
//! not, measured over the unexplained ones only. Frames of two sizes
//! are not compared pixel by pixel: [`Verdict::Resized`], with the
//! reason when a decision declares a size difference.

use engine::headless::Frame;
use serde::{Deserialize, Serialize};

use crate::pixels::centers;

/// Where a decision of the product lets two frames differ, in logical
/// px (the frame's physical size divided by its scale).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", deny_unknown_fields)]
pub enum Area {
    /// A band along one side, `px` deep, the whole side long: a status
    /// bar, a home indicator, a system navigation bar.
    Edge { edge: Edge, px: f32 },
    /// A rect `[x, y, w, h]`.
    Rect { rect: [f32; 4] },
    /// The whole frame: the two frames may differ in size (a scale, a
    /// viewport the platform decides). Never explains a pixel.
    Size,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Edge {
    Top,
    Bottom,
    Left,
    Right,
}

impl Area {
    /// The physical pixels of the area in a `width x height` frame at
    /// `scale`, by pixel center: `[x0, y0, x1, y1)`. `None` for
    /// [`Area::Size`] and for an area outside the frame.
    pub fn physical(&self, width: u32, height: u32, scale: f32) -> Option<[u32; 4]> {
        let (w, h) = (width as f32 / scale, height as f32 / scale);
        let rect = match *self {
            Self::Edge { edge, px } => match edge {
                Edge::Top => [0.0, 0.0, w, px],
                Edge::Bottom => [0.0, h - px, w, px],
                Edge::Left => [0.0, 0.0, px, h],
                Edge::Right => [w - px, 0.0, px, h],
            },
            Self::Rect { rect } => rect,
            Self::Size => return None,
        };
        centers(rect, scale, width, height)
    }
}

/// A decision of the product that lets two frames differ.
#[derive(Clone, Copy, Debug)]
pub struct Decision<'a> {
    /// A short name, `status-bar`.
    pub name: &'a str,
    /// Why, in one line.
    pub reason: &'a str,
    pub area: &'a Area,
}

/// What the difference between two frames is.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "class")]
pub enum Verdict {
    /// Every byte the same.
    Equal,
    /// They differ, and every differing pixel falls in a declared area.
    Requested { reasons: Vec<Explained> },
    /// Some differing pixel falls in no declared area.
    Silent(Silent),
    /// Two sizes, physical px; the reason when a decision declares it.
    Resized {
        base: [u32; 2],
        other: [u32; 2],
        reason: Option<String>,
    },
}

/// The pixels one decision explains.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Explained {
    pub name: String,
    pub reason: String,
    pub pixels: u64,
}

/// A silent difference, measured over the pixels no decision explains.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Silent {
    /// Differing pixels in no declared area.
    pub pixels: u64,
    /// The largest channel difference among them, 1 to 255.
    pub max_delta: u8,
    /// Their mean delta.
    pub mean_delta: f64,
    /// The smallest physical rect holding them, `[x, y, w, h]`.
    pub bbox: [u32; 4],
    /// The differing pixels a decision does explain, apart.
    pub explained: Vec<Explained>,
}

impl Verdict {
    /// Equal, or different only as asked.
    pub fn held(&self) -> bool {
        matches!(self, Self::Equal | Self::Requested { .. })
    }

    /// The verdict in a few words, for a table cell.
    pub fn short(&self) -> String {
        match self {
            Self::Equal => "equal".to_string(),
            Self::Requested { reasons } => {
                let names: Vec<&str> = reasons.iter().map(|r| r.name.as_str()).collect();
                format!("requested ({})", names.join(", "))
            }
            Self::Silent(s) => format!("silent {} px, max {}", s.pixels, s.max_delta),
            Self::Resized { .. } => "resized".to_string(),
        }
    }

    /// The verdict in a line: every number it has.
    pub fn line(&self) -> String {
        match self {
            Self::Equal => "equal in every byte".to_string(),
            Self::Requested { reasons } => {
                let why: Vec<String> = reasons
                    .iter()
                    .map(|r| format!("{} ({} px): {}", r.name, r.pixels, r.reason))
                    .collect();
                format!("requested: {}", why.join("; "))
            }
            Self::Silent(s) => {
                let [x, y, w, h] = s.bbox;
                let mut line = format!(
                    "silent: {} px differ in no declared area, delta max {} mean {:.2}, \
                     at {x},{y} {w}x{h} (physical px)",
                    s.pixels, s.max_delta, s.mean_delta
                );
                for r in &s.explained {
                    line.push_str(&format!("; {} px explained by {}", r.pixels, r.name));
                }
                line
            }
            Self::Resized {
                base,
                other,
                reason,
            } => {
                let why = reason.as_deref().unwrap_or("no decision declares it");
                format!(
                    "resized: {}x{} against {}x{}, not compared by pixel ({why})",
                    base[0], base[1], other[0], other[1]
                )
            }
        }
    }
}

/// The largest channel difference of two RGBA pixels.
fn delta(a: &[u8], b: &[u8]) -> u8 {
    a.iter()
        .zip(b)
        .map(|(p, q)| p.abs_diff(*q))
        .max()
        .unwrap_or(0)
}

/// The silent pixels as they are counted.
#[derive(Default)]
struct Tally {
    pixels: u64,
    sum: u64,
    max: u8,
    lo: [u32; 2],
    hi: [u32; 2],
}

impl Tally {
    fn add(&mut self, x: u32, y: u32, d: u8) {
        if self.pixels == 0 {
            self.lo = [x, y];
            self.hi = [x + 1, y + 1];
        }
        self.pixels += 1;
        self.sum += u64::from(d);
        self.max = self.max.max(d);
        self.lo = [self.lo[0].min(x), self.lo[1].min(y)];
        self.hi = [self.hi[0].max(x + 1), self.hi[1].max(y + 1)];
    }
}

/// Classify `other` against `base`, both at `scale`, under `decisions`
/// (the target's rows of the table; an empty slice declares nothing).
pub fn classify(base: &Frame, other: &Frame, scale: f32, decisions: &[Decision<'_>]) -> Verdict {
    let whole = |f: &Frame| f.rgba.len() as u64 == u64::from(f.width) * u64::from(f.height) * 4;
    if (base.width, base.height) != (other.width, other.height) || !whole(base) || !whole(other) {
        return Verdict::Resized {
            base: [base.width, base.height],
            other: [other.width, other.height],
            reason: decisions
                .iter()
                .find(|d| matches!(d.area, Area::Size))
                .map(|d| d.reason.to_string()),
        };
    }
    if base.rgba == other.rgba {
        return Verdict::Equal;
    }
    let areas: Vec<(usize, [u32; 4])> = decisions
        .iter()
        .enumerate()
        .filter_map(|(i, d)| Some((i, d.area.physical(base.width, base.height, scale)?)))
        .collect();
    let mut by_decision = vec![0u64; decisions.len()];
    let mut silent = Tally::default();
    let w = base.width as usize;
    let pairs = base.rgba.chunks_exact(4).zip(other.rgba.chunks_exact(4));
    for (i, (a, b)) in pairs.enumerate() {
        if a == b {
            continue;
        }
        let (x, y) = ((i % w) as u32, (i / w) as u32);
        let inside = |r: &[u32; 4]| x >= r[0] && x < r[2] && y >= r[1] && y < r[3];
        match areas.iter().find(|(_, r)| inside(r)) {
            Some((at, _)) => by_decision[*at] += 1,
            None => silent.add(x, y, delta(a, b)),
        }
    }
    let explained: Vec<Explained> = decisions
        .iter()
        .zip(&by_decision)
        .filter(|(_, n)| **n > 0)
        .map(|(d, n)| Explained {
            name: d.name.to_string(),
            reason: d.reason.to_string(),
            pixels: *n,
        })
        .collect();
    if silent.pixels == 0 {
        return Verdict::Requested { reasons: explained };
    }
    Verdict::Silent(Silent {
        pixels: silent.pixels,
        max_delta: silent.max,
        mean_delta: silent.sum as f64 / silent.pixels as f64,
        bbox: [
            silent.lo[0],
            silent.lo[1],
            silent.hi[0] - silent.lo[0],
            silent.hi[1] - silent.lo[1],
        ],
        explained,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `w x h` frame of one opaque gray.
    fn frame(w: u32, h: u32) -> Frame {
        Frame {
            width: w,
            height: h,
            rgba: [48, 48, 48, 255].repeat((w * h) as usize),
        }
    }

    /// Add `by` to the red channel of the physical block `[x0, x1) x [y0, y1)`.
    fn nudge(f: &mut Frame, [x0, y0, x1, y1]: [u32; 4], by: u8) {
        for y in y0..y1 {
            for x in x0..x1 {
                let i = ((y * f.width + x) * 4) as usize;
                f.rgba[i] = f.rgba[i].wrapping_add(by);
            }
        }
    }

    const STATUS_BAR: Area = Area::Edge {
        edge: Edge::Top,
        px: 10.0,
    };
    const SCALE: Area = Area::Size;

    fn decisions<'a>(areas: &'a [(&'a str, &'a Area)]) -> Vec<Decision<'a>> {
        areas
            .iter()
            .map(|(name, area)| Decision {
                name,
                reason: "the platform draws there",
                area,
            })
            .collect()
    }

    #[test]
    fn the_same_bytes_are_equal_whatever_is_declared() {
        let d = decisions(&[("status-bar", &STATUS_BAR)]);
        assert_eq!(
            classify(&frame(40, 30), &frame(40, 30), 1.0, &d),
            Verdict::Equal
        );
        assert_eq!(
            classify(&frame(40, 30), &frame(40, 30), 2.0, &[]),
            Verdict::Equal
        );
        assert!(Verdict::Equal.held());
    }

    #[test]
    fn a_difference_inside_a_declared_area_is_requested_with_its_reason() {
        let base = frame(40, 60);
        let mut other = frame(40, 60);
        // the status bar is 10 logical px, 20 physical rows at scale 2
        nudge(&mut other, [0, 0, 40, 20], 90);
        let d = decisions(&[("status-bar", &STATUS_BAR)]);
        let v = classify(&base, &other, 2.0, &d);
        assert_eq!(
            v,
            Verdict::Requested {
                reasons: vec![Explained {
                    name: "status-bar".to_string(),
                    reason: "the platform draws there".to_string(),
                    pixels: 40 * 20,
                }]
            }
        );
        assert!(v.held());
        assert_eq!(v.short(), "requested (status-bar)");
    }

    #[test]
    fn a_difference_outside_every_area_is_silent_and_measured() {
        let base = frame(40, 60);
        let mut other = frame(40, 60);
        // one row past the band is silent; the band itself is explained
        nudge(&mut other, [0, 0, 40, 20], 200);
        nudge(&mut other, [5, 20, 9, 22], 1);
        nudge(&mut other, [30, 50, 31, 51], 3);
        let d = decisions(&[("status-bar", &STATUS_BAR)]);
        let Verdict::Silent(s) = classify(&base, &other, 2.0, &d) else {
            panic!("not silent");
        };
        assert_eq!(s.pixels, 4 * 2 + 1);
        assert_eq!(s.max_delta, 3);
        assert!((s.mean_delta - 11.0 / 9.0).abs() < 1e-9, "{}", s.mean_delta);
        assert_eq!(s.bbox, [5, 20, 26, 31]);
        assert_eq!(s.explained.len(), 1);
        assert_eq!(s.explained[0].pixels, 40 * 20);
        // nothing declared: every differing pixel is silent
        let Verdict::Silent(all) = classify(&base, &other, 2.0, &[]) else {
            panic!("not silent");
        };
        assert_eq!((all.pixels, all.max_delta), (40 * 20 + 9, 200));
        assert!(all.explained.is_empty());
    }

    #[test]
    fn frames_of_two_sizes_are_resized_with_the_declared_reason_or_none() {
        let v = classify(&frame(40, 30), &frame(80, 60), 1.0, &[]);
        assert_eq!(
            v,
            Verdict::Resized {
                base: [40, 30],
                other: [80, 60],
                reason: None
            }
        );
        assert!(!v.held());
        assert!(v.line().contains("no decision declares it"), "{}", v.line());
        let d = decisions(&[("scale", &SCALE), ("status-bar", &STATUS_BAR)]);
        match classify(&frame(40, 30), &frame(80, 60), 1.0, &d) {
            Verdict::Resized { reason, .. } => {
                assert_eq!(reason.as_deref(), Some("the platform draws there"))
            }
            other => panic!("{other:?}"),
        }
        // a frame whose bytes do not fill its size is not compared either
        let mut short = frame(40, 30);
        short.rgba.truncate(40);
        assert!(matches!(
            classify(&frame(40, 30), &short, 1.0, &[]),
            Verdict::Resized { .. }
        ));
    }

    #[test]
    fn areas_map_to_physical_pixels_by_center_on_every_edge() {
        let (w, h) = (125, 75); // 100 x 60 logical at 1.25
        let at = |edge| Area::Edge { edge, px: 10.0 }.physical(w, h, 1.25);
        assert_eq!(at(Edge::Top), Some([0, 0, 125, 12]));
        assert_eq!(at(Edge::Bottom), Some([0, 62, 125, 75]));
        assert_eq!(at(Edge::Left), Some([0, 0, 12, 75]));
        assert_eq!(at(Edge::Right), Some([112, 0, 125, 75]));
        let rect = Area::Rect {
            rect: [10.0, 10.0, 20.0, 8.0],
        };
        assert_eq!(rect.physical(w, h, 1.25), Some([12, 12, 37, 22]));
        assert_eq!(Area::Size.physical(w, h, 1.25), None);
        let outside = Area::Rect {
            rect: [200.0, 0.0, 5.0, 5.0],
        };
        assert_eq!(outside.physical(w, h, 1.25), None);
    }

    #[test]
    fn an_area_reads_from_json_by_kind() {
        let edge: Area = match serde_json::from_str(r#"{"kind":"edge","edge":"top","px":62}"#) {
            Ok(a) => a,
            Err(e) => panic!("{e}"),
        };
        assert_eq!(
            edge,
            Area::Edge {
                edge: Edge::Top,
                px: 62.0
            }
        );
        assert!(matches!(
            serde_json::from_str::<Area>(r#"{"kind":"size"}"#),
            Ok(Area::Size)
        ));
        assert!(serde_json::from_str::<Area>(r#"{"kind":"edge","edge":"middle","px":1}"#).is_err());
        assert!(serde_json::from_str::<Area>(r#"{"kind":"rect","px":1}"#).is_err());
    }
}
