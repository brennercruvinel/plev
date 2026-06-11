//! bench helpers: a piecewise keyframed scalar mirroring one lottie
//! property (boundaries, values, easings), plus the starfish limb data
//! lifted verbatim from ref/anim/lottie-samples/starfish.json.

use anm::{Easing, Node, NodeKind, Prop, Props, Segment, Track, Value};

pub fn cb(x1: f32, y1: f32, x2: f32, y2: f32) -> Easing {
    Easing::CustomBezier { x1, y1, x2, y2 }
}

pub fn rect(id: u16, depth: u16, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) -> Node {
    Node {
        id,
        depth,
        kind: NodeKind::Rect,
        props: Props::new()
            .with(Prop::X, Value::Scalar(x))
            .with(Prop::Y, Value::Scalar(y))
            .with(Prop::W, Value::Scalar(w))
            .with(Prop::H, Value::Scalar(h))
            .with(Prop::Color, Value::Color(color)),
    }
}

/// Piecewise keyframed scalar: n+1 boundaries (seconds, values) and n
/// easings, mirroring one lottie property's keyframe list.
pub struct Kfs {
    pub t: Vec<f32>,
    pub v: Vec<f32>,
    pub e: Vec<Easing>,
}

impl Kfs {
    /// Boundaries given in frames at 60 fps, exactly as in the json.
    pub fn frames(times: &[f32], vals: &[f32], eases: &[Easing]) -> Self {
        assert_eq!(times.len(), vals.len());
        assert_eq!(eases.len(), times.len() - 1);
        Kfs {
            t: times.iter().map(|f| f / 60.0).collect(),
            v: vals.to_vec(),
            e: eases.to_vec(),
        }
    }

    pub fn eval(&self, t: f32) -> f32 {
        if t <= self.t[0] {
            return self.v[0];
        }
        for i in 0..self.e.len() {
            if t < self.t[i + 1] {
                let k = (t - self.t[i]) / (self.t[i + 1] - self.t[i]);
                return self.v[i] + (self.v[i + 1] - self.v[i]) * self.e[i].sample(k);
            }
        }
        *self.v.last().unwrap()
    }

    /// Sum of two keyframed properties (lottie parenting baked in):
    /// boundary union, values summed at each boundary, each merged
    /// segment keeping the easing of whichever source moves more there.
    pub fn plus(&self, other: &Kfs) -> Kfs {
        let mut t: Vec<f32> = self.t.iter().chain(other.t.iter()).copied().collect();
        t.sort_by(f32::total_cmp);
        t.dedup();
        let v: Vec<f32> = t.iter().map(|&b| self.eval(b) + other.eval(b)).collect();
        let e = t
            .windows(2)
            .map(|w| {
                let pick = |k: &Kfs| -> (f32, Easing) {
                    let (mut d, mut ease) = (0.0, Easing::Hold);
                    for i in 0..k.e.len() {
                        if w[0] >= k.t[i] && w[0] < k.t[i + 1] {
                            d = (k.v[i + 1] - k.v[i]).abs();
                            ease = k.e[i];
                        }
                    }
                    (d, ease)
                };
                let (da, ea) = pick(self);
                let (db, eb) = pick(other);
                if da >= db { ea } else { eb }
            })
            .collect();
        Kfs { t, v, e }
    }

    /// (snapshot value, segment chain) for one node prop.
    pub fn track(&self, node_id: u16, prop: Prop) -> (f32, Track) {
        let segments = (0..self.e.len())
            .map(|i| Segment {
                target: Value::Scalar(self.v[i + 1]),
                easing: self.e[i],
                dur_s: self.t[i + 1] - self.t[i],
            })
            .collect();
        (
            self.v[0],
            Track {
                node_id,
                prop,
                start_t: self.t[0],
                segments,
            },
        )
    }
}

/// (times, xs, ys, eases) of one limb null, straight from the json; the
/// last target repeats the bob (the final keyframe carries no s value).
pub type Limb = (&'static [f32], &'static [f32], &'static [f32], Vec<Easing>);

pub fn starfish_limbs() -> Vec<Limb> {
    vec![
        (
            &[0.0, 60.0, 77.0, 124.0, 165.0, 186.0, 215.0, 294.0][..],
            &[
                218.531, 202.969, 218.531, 218.531, 194.969, 218.531, 218.531, 202.969,
            ][..],
            &[
                16.156, 73.156, 16.156, 16.156, -13.844, 16.156, 16.156, 73.156,
            ][..],
            vec![
                cb(0.89, 0.0, 0.57, 1.0),
                cb(0.167, 0.0, 0.833, 0.833),
                Easing::Hold,
                cb(0.89, 0.0, 0.57, 1.0),
                cb(0.167, 0.0, 0.833, 0.833),
                cb(0.167, 0.167, 0.833, 1.0),
                cb(0.89, 0.0, 0.57, 1.0),
            ],
        ),
        (
            &[0.0, 58.0, 75.0, 215.0, 292.0][..],
            &[55.406, 121.406, 55.406, 55.406, 121.406][..],
            &[-92.719, -57.719, -92.719, -92.719, -57.719][..],
            vec![
                cb(0.89, 0.0, 0.57, 1.0),
                cb(0.167, 0.213, 0.833, 0.833),
                cb(0.167, 0.167, 0.833, 1.0),
                cb(0.89, 0.0, 0.57, 1.0),
            ],
        ),
        (
            &[0.0, 57.0, 78.0, 132.0, 159.039, 169.0, 215.0, 291.0][..],
            &[
                -98.531, -56.531, -98.531, -98.531, -56.531, -98.531, -98.531, -56.531,
            ][..],
            &[
                28.719, -21.281, 28.719, 28.719, -21.281, 28.719, 28.719, -21.281,
            ][..],
            vec![
                cb(0.89, 0.0, 0.57, 1.0),
                cb(0.167, 0.233, 0.833, 0.833),
                cb(0.167, 0.167, 0.833, 1.0),
                cb(0.89, 0.0, 0.57, 1.0),
                cb(0.167, 0.233, 0.833, 0.833),
                cb(0.167, 0.167, 0.833, 1.0),
                cb(0.89, 0.0, 0.57, 1.0),
            ],
        ),
        (
            &[0.0, 61.0, 81.0, 215.0, 295.0][..],
            &[-30.594, 15.906, -30.594, -30.594, 15.906][..],
            &[212.719, 233.219, 212.719, 212.719, 233.219][..],
            vec![
                cb(0.89, 0.0, 0.57, 1.0),
                cb(0.167, 0.222, 0.833, 0.833),
                cb(0.167, 0.167, 0.833, 1.0),
                cb(0.89, 0.0, 0.57, 1.0),
            ],
        ),
        (
            &[0.0, 60.0, 75.0, 215.0, 294.0][..],
            &[165.344, 195.344, 165.344, 165.344, 195.344][..],
            &[204.969, 163.969, 204.969, 204.969, 163.969][..],
            vec![
                cb(0.605, 0.0, 0.86, 1.0),
                cb(0.472, 0.0, 0.833, 0.833),
                cb(0.167, 0.167, 0.833, 1.0),
                cb(0.605, 0.0, 0.86, 1.0),
            ],
        ),
    ]
}
