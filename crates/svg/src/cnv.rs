//! Conversion: an svg document -> a self-contained single-frame .monster
//! file. The svg is read ONCE here: usvg normalizes it, [`crate::tess`]
//! tessellates the paths, the geometry is deduped into the monster asset
//! table, and the encoder writes one keyframe. Playback afterwards is
//! `monster::MonsterPlayer` over our format; no svg code runs.
//!
//! an svg is a still image, so the timeline is a single keyframe: every
//! tessellated path is one node placed on the stage. dedup by exact
//! quantized bytes still applies, so two identical shapes in the document
//! share one asset (the swf display-list lesson at shape granularity).

use std::collections::HashMap;

use crate::tess;
use monster::asset_path::pack_chunks;
use monster::{
    Asset, AssetKind, Desc, DiscoverConfig, DiscoverError, Node, NodeKind, Props, WriteError,
    discover, encode,
};

/// What the conversion measured; the honest numbers for the table.
#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub width: f32,
    pub height: f32,
    /// Tessellated paths the svg produced (before the u16 chunk split).
    pub paths: usize,
    /// Distinct geometry payloads in the asset table.
    pub assets: usize,
    /// Sum of asset payload bytes (the whole cost of a still image).
    pub asset_bytes: usize,
    pub nodes: usize,
    pub monster_bytes: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum SvgError {
    #[error("svg parse: {0}")]
    Parse(#[from] usvg::Error),
    /// Nothing visible after tessellation: an empty .monster would lie.
    #[error("no drawable content in the svg")]
    NothingToConvert,
    /// More distinct payloads than u16 asset ids can name.
    #[error("asset table exceeds 65535 distinct payloads; the svg is too detailed for v0")]
    TooManyAssets,
    /// More shapes than u16 depth slots.
    #[error("svg holds more than 65535 shapes")]
    TooManyNodes,
    #[error("delta discovery: {0}")]
    Discover(#[from] DiscoverError),
    #[error("monster encode: {0}")]
    Write(#[from] WriteError),
}

/// Convert svg text into .monster bytes plus the measured stats. `name`
/// labels the description track; the single keyframe's entry carries the
/// stage size machine-readably as `stage WxH`, which is how a player
/// learns the composition bounds without any svg knowledge.
pub fn convert(svg: &str, name: &str) -> Result<(Vec<u8>, Stats), SvgError> {
    let opt = usvg::Options::default();
    let tree = parse(svg, &opt)?;
    let size = tree.size();
    let (width, height) = (size.width(), size.height());

    let paths = tess::tessellate(&tree);
    let mut intern: HashMap<Vec<u8>, u16> = HashMap::new();
    let mut assets: Vec<Asset> = Vec::new();
    let mut scene: Vec<Node> = Vec::new();
    for path in &paths {
        for payload in pack_chunks(path) {
            let next_id = assets.len();
            let id = *intern.entry(payload).or_insert_with_key(|key| {
                assets.push(Asset {
                    kind: AssetKind::Path,
                    data: key.clone(),
                });
                next_id as u16
            });
            if assets.len() > usize::from(u16::MAX) {
                return Err(SvgError::TooManyAssets);
            }
            let slot = scene.len();
            if slot > usize::from(u16::MAX) {
                return Err(SvgError::TooManyNodes);
            }
            scene.push(Node {
                id: slot as u16,
                depth: slot as u16,
                kind: NodeKind::Path { path: id },
                props: Props::new(),
            });
        }
    }
    if scene.is_empty() {
        return Err(SvgError::NothingToConvert);
    }
    let nodes = scene.len();

    // one still frame at t=0; discover collapses it to a single keyframe.
    let frames = vec![(0.0f32, scene)];
    let timeline = discover(&frames, &DiscoverConfig::default())?;
    let descs = vec![Desc {
        keyframe: 0,
        text: format!(
            "stage {}x{} | {name} | converted by svg",
            width as u32, height as u32
        ),
    }];
    let bytes = encode(&timeline, &assets, &descs)?;

    let stats = Stats {
        width,
        height,
        paths: paths.len(),
        assets: assets.len(),
        asset_bytes: assets.iter().map(|a| a.data.len()).sum(),
        nodes,
        monster_bytes: bytes.len(),
    };
    Ok((bytes, stats))
}

/// Parse, tolerating one common corpus quirk: exporters (LottieFiles among
/// them) emit `xlink:href` without declaring the xlink namespace on the
/// root, which usvg's strict xml layer rejects. When that is the only
/// problem, inject the standard declaration once and retry; any other
/// failure is returned as-is.
fn parse(svg: &str, opt: &usvg::Options) -> Result<usvg::Tree, usvg::Error> {
    match usvg::Tree::from_str(svg, opt) {
        Ok(tree) => Ok(tree),
        Err(err) if svg.contains("xlink:") && !svg.contains("xmlns:xlink") => {
            let patched = svg.replacen(
                "<svg",
                "<svg xmlns:xlink=\"http://www.w3.org/1999/xlink\"",
                1,
            );
            usvg::Tree::from_str(&patched, opt).map_err(|_| err)
        }
        Err(err) => Err(err),
    }
}
