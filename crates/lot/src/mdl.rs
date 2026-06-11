//! Serde data model for the lottie subset this crate plays.
//!
//! Properties keep their raw `serde_json::Value` payload (`Prop.k`);
//! evaluation lives in `kfr`. Unknown fields are ignored by serde, so
//! unsupported lottie features degrade gracefully at parse time.

use serde::Deserialize;
use serde_json::Value;

fn one() -> f64 {
    1.0
}

fn big() -> f64 {
    1e9
}

#[derive(Deserialize)]
pub struct Animation {
    pub fr: f64,
    #[serde(default)]
    pub ip: f64,
    #[serde(default = "big")]
    pub op: f64,
    pub w: f64,
    pub h: f64,
    #[serde(default)]
    pub layers: Vec<Layer>,
    #[serde(default)]
    pub assets: Vec<Asset>,
}

impl Animation {
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    pub fn asset(&self, id: &str) -> Option<&Asset> {
        self.assets.iter().find(|a| a.id == id)
    }
}

#[derive(Deserialize)]
pub struct Asset {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub layers: Vec<Layer>,
}

#[derive(Deserialize)]
pub struct Layer {
    pub ty: i64,
    #[serde(default)]
    pub ind: i64,
    #[serde(default)]
    pub parent: Option<i64>,
    #[serde(default, rename = "refId")]
    pub ref_id: Option<String>,
    #[serde(default)]
    pub ks: Transform,
    #[serde(default)]
    pub ip: f64,
    #[serde(default = "big")]
    pub op: f64,
    #[serde(default)]
    pub st: f64,
    #[serde(default = "one")]
    pub sr: f64,
    #[serde(default)]
    pub shapes: Vec<Shape>,
    #[serde(default)]
    pub hd: bool,
    #[serde(default)]
    pub nm: String,
}

/// Layer or group transform. All members optional; defaults applied at
/// evaluation time (anchor/position 0, scale 100, rotation 0, opacity 100).
#[derive(Deserialize, Default)]
pub struct Transform {
    pub a: Option<Prop>,
    pub p: Option<Prop>,
    pub s: Option<Prop>,
    pub r: Option<Prop>,
    pub o: Option<Prop>,
    pub sk: Option<Prop>,
}

/// An animatable property: `a` flags keyframes, `k` is the raw payload
/// (scalar, vector, bezier path object, or keyframe array).
pub struct Prop {
    pub a: i64,
    pub k: Value,
}

impl<'de> Deserialize<'de> for Prop {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = Value::deserialize(d)?;
        Ok(match &v {
            // Canonical {a, k} form.
            Value::Object(m) if m.contains_key("k") => Prop {
                a: m.get("a").and_then(Value::as_i64).unwrap_or(0),
                k: m.get("k").cloned().unwrap_or(Value::Null),
            },
            // Bare value (e.g. fill rule `r: 1`): treat as static.
            _ => Prop { a: 0, k: v },
        })
    }
}

/// One item in a shape layer's `shapes` (or a group's `it`) list. A single
/// struct covers every supported `ty`; fields not used by a given type
/// simply stay `None`.
#[derive(Deserialize)]
pub struct Shape {
    #[serde(default)]
    pub ty: String,
    #[serde(default)]
    pub it: Vec<Shape>,
    /// sh: bezier path data.
    #[serde(default)]
    pub ks: Option<Prop>,
    /// fl/st: color.
    #[serde(default)]
    pub c: Option<Prop>,
    /// fl/st/tr/gf/gs: opacity (0-100).
    #[serde(default)]
    pub o: Option<Prop>,
    /// st/gs: stroke width.
    #[serde(default)]
    pub w: Option<Prop>,
    /// el/rc/tr: position.
    #[serde(default)]
    pub p: Option<Prop>,
    /// el/rc size, tr scale.
    #[serde(default)]
    pub s: Option<Prop>,
    /// tr: anchor.
    #[serde(default)]
    pub a: Option<Prop>,
    /// tr rotation / rc roundness / fl fill-rule.
    #[serde(default)]
    pub r: Option<Prop>,
    /// tr: skew.
    #[serde(default)]
    pub sk: Option<Prop>,
    /// gf/gs: gradient stops.
    #[serde(default)]
    pub g: Option<Grad>,
    #[serde(default)]
    pub hd: bool,
    #[serde(default)]
    pub nm: String,
}

#[derive(Deserialize)]
pub struct Grad {
    #[serde(default)]
    pub p: i64,
    #[serde(default)]
    pub k: Option<Prop>,
}
