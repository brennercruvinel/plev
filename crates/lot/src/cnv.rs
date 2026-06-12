//! Conversion: a parsed lottie animation -> a self-contained .monster
//! file. This is the bridge that retires the json at the door: `lot`
//! reads it ONCE, samples the composition through [`crate::rnd`],
//! dedups the tessellated geometry into the monster asset table, and the
//! monster encoder discovers the deltas. Playback afterwards is
//! `monster::MonsterPlayer` over our format; no lottie code runs.
//!
//! the dedup thesis (swf display list at shape granularity): payloads
//! are compared as exact quantized bytes, so a static shape is one
//! asset referenced by every frame, the node never changes, and the
//! delta encoder emits zero bytes for it. a moving shape re-tessellates
//! per sample (morph = cpu re-tessellation in v0) and becomes a replace
//! op pointing at a new asset; the asset table is the cost of motion,
//! the timeline stays tiny.

use std::collections::HashMap;

use crate::gem::Mat;
use crate::mdl::Animation;
use crate::rnd::Player;
use monster::asset_path::pack_chunks;
use monster::{
    Asset, AssetKind, Desc, DiscoverConfig, DiscoverError, Node, NodeKind, Props, WriteError,
    discover, encode,
};

/// What the conversion measured; the honest numbers for the table.
#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub width: f64,
    pub height: f64,
    pub frames: usize,
    pub fps: f64,
    pub duration_s: f32,
    /// Distinct geometry payloads in the asset table.
    pub assets: usize,
    /// Sum of asset payload bytes (the dominant cost of motion).
    pub asset_bytes: usize,
    pub keyframes: usize,
    pub places: usize,
    pub replaces: usize,
    pub removes: usize,
    pub monster_bytes: usize,
}

#[derive(Debug)]
pub enum CnvError {
    Json(serde_json::Error),
    /// Nothing visible in any sampled frame: an empty .monster would lie.
    NothingToConvert,
    /// More distinct payloads than u16 asset ids; resample lower.
    TooManyAssets {
        frames_done: usize,
    },
    /// More simultaneous shapes than u16 depth slots.
    TooManyNodes {
        at_frame: usize,
    },
    Discover(DiscoverError),
    Write(WriteError),
}

impl std::fmt::Display for CnvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CnvError::Json(e) => write!(f, "json parse: {e}"),
            CnvError::NothingToConvert => write!(f, "no drawable content in any sampled frame"),
            CnvError::TooManyAssets { frames_done } => write!(
                f,
                "asset table exceeds 65535 distinct payloads after {frames_done} frames; \
                 lower the sample rate"
            ),
            CnvError::TooManyNodes { at_frame } => {
                write!(f, "frame {at_frame} holds more than 65535 shapes")
            }
            CnvError::Discover(e) => write!(f, "delta discovery: {e}"),
            CnvError::Write(e) => write!(f, "monster encode: {e}"),
        }
    }
}

impl std::error::Error for CnvError {}

impl From<serde_json::Error> for CnvError {
    fn from(e: serde_json::Error) -> Self {
        CnvError::Json(e)
    }
}

impl From<DiscoverError> for CnvError {
    fn from(e: DiscoverError) -> Self {
        CnvError::Discover(e)
    }
}

impl From<WriteError> for CnvError {
    fn from(e: WriteError) -> Self {
        CnvError::Write(e)
    }
}

/// Convert lottie json text into .monster bytes plus the measured stats.
/// `name` labels the description track (the format's text channel,
/// spec decision 7); the first keyframe's entry carries the stage size
/// machine-readably as `stage WxH`, which is how a player learns the
/// composition bounds without any lottie knowledge.
pub fn convert(json: &str, name: &str) -> Result<(Vec<u8>, Stats), CnvError> {
    let anim = Animation::from_json(json)?;
    let fr = anim.fr.max(1.0);
    let frame_count = ((anim.op - anim.ip).ceil() as usize).max(1);
    let (width, height) = (anim.w, anim.h);
    let ip = anim.ip;
    let mut player = Player::new(anim);

    let mut intern: HashMap<Vec<u8>, u16> = HashMap::new();
    let mut assets: Vec<Asset> = Vec::new();
    let mut frames: Vec<(f32, Vec<Node>)> = Vec::with_capacity(frame_count);
    for i in 0..frame_count {
        let t = i as f32 / fr as f32;
        let mut scene: Vec<Node> = Vec::new();
        for path in player.render(ip + i as f64, Mat::IDENTITY) {
            for payload in pack_chunks(&path) {
                let next_id = assets.len();
                let id = *intern.entry(payload).or_insert_with_key(|key| {
                    assets.push(Asset {
                        kind: AssetKind::Path,
                        data: key.clone(),
                    });
                    next_id as u16
                });
                if assets.len() > usize::from(u16::MAX) {
                    return Err(CnvError::TooManyAssets { frames_done: i });
                }
                let slot = scene.len();
                if slot > usize::from(u16::MAX) {
                    return Err(CnvError::TooManyNodes { at_frame: i });
                }
                scene.push(Node {
                    id: slot as u16,
                    depth: slot as u16,
                    kind: NodeKind::Path { path: id },
                    props: Props::new(),
                });
            }
        }
        frames.push((t, scene));
    }
    if frames.iter().all(|(_, scene)| scene.is_empty()) {
        return Err(CnvError::NothingToConvert);
    }

    let timeline = discover(&frames, &DiscoverConfig::default())?;
    let descs = vec![Desc {
        keyframe: 0,
        text: format!(
            "stage {}x{} | {name} | converted by lot",
            width as u32, height as u32
        ),
    }];
    let bytes = encode(&timeline, &assets, &descs)?;

    let stats = Stats {
        width,
        height,
        frames: frame_count,
        fps: fr,
        duration_s: timeline.duration_s,
        assets: assets.len(),
        asset_bytes: assets.iter().map(|a| a.data.len()).sum(),
        keyframes: timeline.keyframes.len(),
        places: timeline.places.len(),
        replaces: timeline.replaces.len(),
        removes: timeline.removes.len(),
        monster_bytes: bytes.len(),
    };
    Ok((bytes, stats))
}
