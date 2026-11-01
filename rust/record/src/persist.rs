//! a `Session` to and from a session file (see `format` for the layout).
//! the writer copies every chunk's monster bytes as they are; the reader
//! is strict: the header, every section, every event line, every chunk
//! (through `motion::decode`) and every asset reference is checked, and
//! anything the writer would not have written is a typed `FileError`.
//!
//! every window is written as version 2: with section B when it has
//! bindings, without it when not, and each text with its wrap rule and
//! OpenType features (see `format` and `payload`). a file of version 0
//! reads with no bindings (see `bound`), and one of version 0 or 1 with
//! every text at the engine's defaults.
//!
//! one form the writer no longer writes still reads: a snapshot event
//! from before task-83 names each node its step added by id only
//! (`{"node_added": "<id>"}`). that node is the one the event's own
//! snapshot holds under the id, so the reader puts it in place and loses
//! nothing (see `read_event`).

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use accord::Millis;
use engine::drawing::gpu::image::MAX_IMAGE_ATLAS_SIZE;
use motion::{LoweredAsset, NodeKind};
use serde_json::Value as Json;

use crate::bound::{self, Step};
use crate::error::FileError;
use crate::event::{Event, Kind};
use crate::format::{self, Cur, len32, put_u32, put_u64};
use crate::payload::{Read, put_asset, read_asset};
use crate::session::Session;
use crate::visuals::reader::{Asset, Screen, Shown};

/// what a write put in the file.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Persisted {
    /// the file's size in bytes, and the part each section takes.
    pub bytes: usize,
    pub events_bytes: usize,
    pub index_bytes: usize,
    pub chunks_bytes: usize,
    pub assets_bytes: usize,
    /// section B, where each bound control showed; 0 when the window has
    /// no bindings and the file leaves the section out.
    pub bindings_bytes: usize,
    /// events written.
    pub events: usize,
    /// events left out because their json does not read back (a `Num`
    /// that is not finite has no json form), with the changes that
    /// depended on a snapshot left out that way.
    pub dropped: usize,
    pub chunks: usize,
    pub assets: usize,
    /// images written with their size only (see `payload`).
    pub images_without_pixels: usize,
}

/// the S section: one json line per event. an event whose line does not
/// read back (a `Num` that is not finite has no json form) is left out.
/// when it is a snapshot or a change, so are the changes after it up to
/// the next snapshot, which would apply to a state that is not the one
/// recorded: the state in that gap reads as the last one before it.
fn events_section(events: &[Event], out: &mut Persisted) -> Result<Vec<u8>, FileError> {
    let mut buf = Vec::new();
    let mut orphaned = false;
    for event in events {
        let keyed = matches!(event.kind, Kind::Snapshot { .. });
        let chained = keyed || matches!(event.kind, Kind::Changes(_));
        if orphaned && !keyed && chained {
            out.dropped += 1;
            continue;
        }
        let line = serde_json::to_string(event).map_err(|e| FileError::Serialize(e.to_string()))?;
        if serde_json::from_str::<Event>(&line).is_err() {
            out.dropped += 1;
            orphaned |= chained;
            continue;
        }
        if keyed {
            orphaned = false;
        }
        buf.extend_from_slice(line.as_bytes());
        buf.push(b'\n');
        out.events += 1;
    }
    Ok(buf)
}

/// the I and C sections.
fn chunk_sections(screen: &Screen) -> Result<(Vec<u8>, Vec<u8>), FileError> {
    let mut index = Vec::new();
    let mut chunks = Vec::new();
    put_u32(&mut index, len32(screen.chunks.len(), "chunk count")?);
    for c in &screen.chunks {
        put_u64(&mut index, c.start);
        put_u32(&mut index, len32(c.times.len(), "frame count")?);
        for t in &c.times {
            put_u32(&mut index, *t);
        }
        put_u32(&mut index, len32(c.refs.len(), "asset refs")?);
        for r in &c.refs {
            put_u32(&mut index, *r);
        }
        put_u64(&mut index, chunks.len() as u64);
        put_u64(&mut index, c.bytes.len() as u64);
        chunks.extend_from_slice(&c.bytes);
    }
    Ok((index, chunks))
}

impl Session {
    /// the session as a file, and what went in it.
    pub fn to_bytes(&self) -> Result<(Vec<u8>, Persisted), FileError> {
        let mut out = Persisted::default();
        let events = events_section(self.events(), &mut out)?;
        let screen = self.screen();
        let (index, chunks) = chunk_sections(screen)?;
        let mut assets = Vec::new();
        put_u32(&mut assets, len32(screen.assets.len(), "asset count")?);
        for asset in &screen.assets {
            put_asset(&mut assets, asset)?;
            let bare = match asset {
                Asset::Live(LoweredAsset::Image(_)) => true,
                Asset::Image { pixels, .. } => pixels.is_none(),
                Asset::Live(_) => false,
            };
            out.images_without_pixels += usize::from(bare);
        }
        let steps: Vec<&[Step]> = screen.chunks.iter().map(|c| c.bound.as_slice()).collect();
        let bindings = bound::section(&steps)?;
        let bytes = format::assemble(
            self.start(),
            self.build(),
            [&events, &index, &chunks, &assets],
            bindings.as_deref(),
        );
        out.bytes = bytes.len();
        out.bindings_bytes = bindings.as_ref().map_or(0, Vec::len);
        out.events_bytes = events.len();
        out.index_bytes = index.len();
        out.chunks_bytes = chunks.len();
        out.assets_bytes = assets.len();
        out.chunks = screen.chunks.len();
        out.assets = screen.assets.len();
        Ok((bytes, out))
    }

    /// writes the session to `path`, whole or not at all: the bytes go to
    /// `<path>.part` first and are renamed over `path`.
    pub fn write(&self, path: impl AsRef<Path>) -> Result<Persisted, FileError> {
        let path = path.as_ref();
        let (bytes, out) = self.to_bytes()?;
        let mut part = OsString::from(path.as_os_str());
        part.push(".part");
        let part = PathBuf::from(part);
        let written = fs::write(&part, &bytes).and_then(|()| fs::rename(&part, path));
        if let Err(e) = written {
            let _ = fs::remove_file(&part);
            return Err(e.into());
        }
        Ok(out)
    }

    /// reads a session file.
    pub fn open(path: impl AsRef<Path>) -> Result<Session, FileError> {
        Session::from_bytes(&fs::read(path)?)
    }

    /// reads a session from its bytes, strictly. an image that carries
    /// its pixels is loaded into this process's image store
    /// (`engine::drawing::gpu::load_image_rgba`) so its node draws; one without is
    /// kept with its size and draws nothing.
    pub fn from_bytes(bytes: &[u8]) -> Result<Session, FileError> {
        let parsed = format::parse(bytes)?;
        let [events, index, chunks, assets] = parsed.sections;
        let styled = parsed.version >= format::VERSION_TEXT_STYLE;
        let assets = read_assets(assets, styled)?;
        let mut chunks = read_chunks(index, chunks, &assets)?;
        if let Some(payload) = parsed.bindings {
            let starts: Vec<Millis> = chunks.iter().map(|c| c.start).collect();
            let steps = bound::read_section(payload, &starts)?;
            for (chunk, bound) in chunks.iter_mut().zip(steps) {
                chunk.bound = bound;
            }
        }
        let events = read_events(events)?;
        let session = Session::from_file(events, Screen { chunks, assets }, parsed.build);
        if session.start() != parsed.start {
            return Err(FileError::StartMismatch {
                header: parsed.start,
                content: session.start(),
            });
        }
        Ok(session)
    }
}

fn read_events(payload: &[u8]) -> Result<Vec<Event>, FileError> {
    let mut out: Vec<Event> = Vec::new();
    let Some(body) = payload.strip_suffix(b"\n") else {
        return if payload.is_empty() {
            Ok(out)
        } else {
            Err(FileError::Truncated("the last event line"))
        };
    };
    for (at, line) in body.split(|b| *b == b'\n').enumerate() {
        let line_no = at + 1;
        let event = read_event(line, line_no)?;
        if out.last().is_some_and(|last| event.t < last.t) {
            return Err(FileError::EventOrder { line: line_no });
        }
        out.push(event);
    }
    Ok(out)
}

/// one line of the S section. a line that does not read as an `Event` gets
/// one more try, for the form before task-83: in a snapshot event, each
/// `node_added` that is a bare id becomes the node the event's snapshot
/// holds under that id, exactly as it was recorded (the node a step adds
/// is in the snapshot that step recorded). an id the snapshot does not
/// hold is a typed error; no node is ever made up. any other line keeps
/// the error it had.
fn read_event(line: &[u8], line_no: usize) -> Result<Event, FileError> {
    let bad = |message: String| FileError::Event {
        line: line_no,
        message,
    };
    let refused = match serde_json::from_slice::<Event>(line) {
        Ok(event) => return Ok(event),
        Err(e) => e,
    };
    let Ok(mut json) = serde_json::from_slice::<Json>(line) else {
        return Err(bad(refused.to_string()));
    };
    match nodes_added_by_id(&mut json) {
        Ok(0) => Err(bad(refused.to_string())),
        Ok(_) => serde_json::from_value(json).map_err(|e| bad(e.to_string())),
        Err(id) => Err(bad(format!(
            "node_added names {id}, which the snapshot of its event does not hold"
        ))),
    }
}

/// puts in place, in a snapshot event's `changes`, the node its snapshot
/// holds for each `node_added` written as a bare id; how many it put.
/// `Err` with the first id the snapshot does not hold.
fn nodes_added_by_id(json: &mut Json) -> Result<usize, String> {
    let Some(event) = json.pointer_mut("/kind/snapshot") else {
        return Ok(0);
    };
    let Some(mut changes) = event.get_mut("changes").map(Json::take) else {
        return Ok(0);
    };
    let nodes = event
        .pointer("/snapshot/nodes")
        .and_then(Json::as_array)
        .map_or(&[][..], Vec::as_slice);
    let mut put = 0;
    for change in changes.as_array_mut().into_iter().flatten() {
        let Some(Json::String(id)) = change.get("node_added") else {
            continue;
        };
        let node = nodes
            .iter()
            .find(|n| n.get("id").and_then(Json::as_str) == Some(id.as_str()))
            .ok_or_else(|| id.clone())?;
        change["node_added"] = node.clone();
        put += 1;
    }
    event["changes"] = changes;
    Ok(put)
}

fn read_assets(payload: &[u8], styled: bool) -> Result<Vec<Asset>, FileError> {
    let mut c = Cur::new(payload);
    let count = c.count(5, "asset count")?;
    let mut out = Vec::with_capacity(count);
    for at in 0..count {
        out.push(match read_asset(&mut c, at, styled)? {
            Read::Ready(asset) => Asset::Live(asset),
            Read::Image(image) => {
                // the store checks its pixel count in u32 and its atlas
                // side limit itself; an image past either is kept with
                // its size and draws nothing.
                let fits = image.width.max(image.height) <= MAX_IMAGE_ATLAS_SIZE;
                let handle = image.pixels.as_ref().filter(|_| fits).and_then(|p| {
                    engine::drawing::gpu::load_image_rgba(image.width, image.height, p.to_vec())
                        .ok()
                });
                Asset::Image {
                    width: image.width,
                    height: image.height,
                    pixels: image.pixels,
                    handle,
                }
            }
        });
    }
    c.done("the asset table")?;
    Ok(out)
}

fn read_chunks(index: &[u8], body: &[u8], assets: &[Asset]) -> Result<Vec<Shown>, FileError> {
    let mut c = Cur::new(index);
    let count = c.count(32, "chunk count")?;
    let mut out: Vec<Shown> = Vec::with_capacity(count);
    let mut offset = 0u64;
    for chunk in 0..count {
        let bad = |reason| FileError::Index { chunk, reason };
        let start: Millis = c.u64("chunk start")?;
        let frames = c.count(4, "frame count")?;
        let mut times = Vec::with_capacity(frames);
        for _ in 0..frames {
            let t = c.u32("frame time")?;
            if times.last().is_some_and(|&last| t <= last) {
                return Err(bad("frame times do not increase"));
            }
            times.push(t);
        }
        match (times.first(), times.last()) {
            (Some(0), Some(&last)) if start.checked_add(Millis::from(last)).is_some() => {}
            (Some(0), Some(_)) => return Err(bad("the last frame time overflows")),
            _ => return Err(bad("a chunk opens with its first frame at 0")),
        }
        if out.last().is_some_and(|before| start <= before.start) {
            return Err(bad("chunk starts do not increase"));
        }
        let n = c.count(4, "asset refs")?;
        let mut refs = Vec::with_capacity(n);
        for _ in 0..n {
            let r = c.u32("asset ref")?;
            if usize::try_from(r).map_or(true, |r| r >= assets.len()) {
                return Err(bad("an asset ref is past the asset table"));
            }
            refs.push(r);
        }
        let at = c.u64("chunk offset")?;
        let len = c.u64("chunk length")?;
        if at != offset {
            return Err(bad("the chunks do not tile section C"));
        }
        let end = at
            .checked_add(len)
            .filter(|&end| end <= body.len() as u64)
            .ok_or(bad("the chunk runs past section C"))?;
        let bytes = body[at as usize..end as usize].to_vec();
        check_chunk(chunk, &bytes, &refs, assets)?;
        offset = end;
        out.push(Shown {
            start,
            times,
            bytes,
            refs,
            bound: Vec::new(),
        });
    }
    c.done("the chunk index")?;
    if offset != body.len() as u64 {
        return Err(FileError::Index {
            chunk: count,
            reason: "section C has bytes past the last chunk",
        });
    }
    Ok(out)
}

/// a chunk must decode as a monster file, and every node that references
/// an asset must reference one of its refs, of its kind.
fn check_chunk(
    chunk: usize,
    bytes: &[u8],
    refs: &[u32],
    assets: &[Asset],
) -> Result<(), FileError> {
    let doc = motion::decode(bytes).map_err(|e| FileError::Chunk {
        chunk,
        reason: e.to_string(),
    })?;
    let t = &doc.timeline;
    let nodes = t
        .keyframes
        .iter()
        .flat_map(|k| k.snapshot.iter())
        .chain(t.places.iter().map(|p| &p.node))
        .chain(t.replaces.iter().map(|r| &r.node));
    for node in nodes {
        let (id, fits): (u16, fn(&Asset) -> bool) = match node.kind {
            NodeKind::Text { style } => (style, |a| {
                matches!(a, Asset::Live(LoweredAsset::TextStyle(_)))
            }),
            NodeKind::Image { image } => (image, |a| {
                matches!(a, Asset::Live(LoweredAsset::Image(_)) | Asset::Image { .. })
            }),
            NodeKind::Path { path } => (path, |a| matches!(a, Asset::Live(LoweredAsset::Path(_)))),
            NodeKind::Rect | NodeKind::RoundedRect | NodeKind::GradientRect => continue,
        };
        let asset = refs
            .get(usize::from(id))
            .and_then(|&r| usize::try_from(r).ok())
            .and_then(|r| assets.get(r));
        if !asset.is_some_and(fits) {
            return Err(FileError::Chunk {
                chunk,
                reason: format!("node {} names asset {id}, not one of its kind", node.id),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use engine::drawing::compositor::SceneNode;
    use engine::drawing::gpu::ImageHandle;
    use motion::LoweredAsset;

    use crate::format::{self, Cur};
    use crate::recorder::Recorder;
    use crate::session::Session;
    use crate::visuals::reader::{Asset, Screen};

    fn rect(x: f32) -> SceneNode {
        SceneNode::Rect {
            x,
            y: 0.0,
            w: 4.0,
            h: 4.0,
            color: [1.0, 0.0, 0.0, 1.0],
        }
    }

    #[test]
    fn every_chunk_in_the_file_is_the_monster_bytes_as_they_were() {
        let rec = Recorder::new(crate::Limits {
            chunk_every: 100,
            ..crate::Limits::default()
        });
        for t in (0..1_000u64).step_by(16) {
            assert!(rec.frame_scene(t, &[rect(t as f32 / 10.0)]).is_ok());
        }
        let memory = rec.snapshot_window();
        let Ok((bytes, _)) = memory.to_bytes() else {
            panic!("the window does not write");
        };
        let Ok(parsed) = format::parse(&bytes) else {
            panic!("the header does not read");
        };
        let [_, index, body, _] = parsed.sections;
        let mut c = Cur::new(index);
        let Ok(count) = c.u32("count") else {
            panic!("no chunk count");
        };
        assert!(count > 5);
        assert_eq!(count as usize, memory.screen().chunks.len());
        for (n, chunk) in memory.screen().chunks.iter().enumerate() {
            let mut read = || -> Result<(u64, u64), crate::FileError> {
                c.u64("start")?;
                let frames = c.u32("frames")? as usize;
                c.take(frames * 4, "times")?;
                let refs = c.u32("refs")? as usize;
                c.take(refs * 4, "refs")?;
                Ok((c.u64("offset")?, c.u64("len")?))
            };
            let Ok((at, len)) = read() else {
                panic!("chunk {n} does not index");
            };
            let inside = &body[at as usize..(at + len) as usize];
            assert_eq!(inside, chunk.bytes.as_slice(), "chunk {n} verbatim");
            assert!(motion::decode(inside).is_ok(), "chunk {n} decodes");
        }
    }

    #[test]
    fn an_image_with_its_pixels_is_loaded_and_drawn() {
        let rec = Recorder::default();
        let handle = ImageHandle {
            id: 1,
            atlas_x: 1,
            atlas_y: 1,
            width: 2,
            height: 1,
        };
        let image = SceneNode::Image {
            x: 0.0,
            y: 0.0,
            w: 2.0,
            h: 1.0,
            image: handle,
            corner_radius: 0.0,
        };
        assert!(rec.frame_scene(0, &[image]).is_ok());
        let window = rec.snapshot_window();
        let mut screen: Screen = window.screen().clone();
        let pixels: Arc<[u8]> = Arc::from(vec![255u8, 0, 0, 255, 0, 255, 0, 255]);
        for asset in &mut screen.assets {
            if let Asset::Live(LoweredAsset::Image(h)) = asset {
                *asset = Asset::Image {
                    width: h.width,
                    height: h.height,
                    pixels: Some(pixels.clone()),
                    handle: None,
                };
            }
        }
        let session = Session::from_parts(window.events().to_vec(), screen);
        let Ok((bytes, written)) = session.to_bytes() else {
            panic!("the window does not write");
        };
        assert_eq!(written.images_without_pixels, 0);
        let Ok(file) = Session::from_bytes(&bytes) else {
            panic!("the file does not read");
        };
        let shown = file.scene_at(0).ok().flatten().unwrap_or_default();
        let [SceneNode::Image { image, .. }] = shown.as_slice() else {
            panic!("the image draws: {shown:?}");
        };
        assert_eq!((image.width, image.height), (2, 1));
        let kept = file.screen().assets.iter().any(
            |a| matches!(a, Asset::Image { pixels: Some(p), handle: Some(_), .. } if *p == pixels),
        );
        assert!(kept, "the pixels stay for the next write");

        // a pixel run shorter than the size says is a typed error.
        let mut short = bytes.clone();
        short.truncate(short.len() - 1);
        assert!(Session::from_bytes(&short).is_err());
    }

    #[test]
    fn an_image_past_the_atlas_limit_keeps_its_size_and_is_not_loaded() {
        let side = super::MAX_IMAGE_ATLAS_SIZE + 1;
        let pixels: Arc<[u8]> = Arc::from(vec![0u8; side as usize * 4]);
        let screen = Screen {
            chunks: Vec::new(),
            assets: vec![Asset::Image {
                width: side,
                height: 1,
                pixels: Some(pixels),
                handle: None,
            }],
        };
        let session = Session::from_parts(Vec::new(), screen);
        let Ok((bytes, _)) = session.to_bytes() else {
            panic!("the window does not write");
        };
        let Ok(file) = Session::from_bytes(&bytes) else {
            panic!("the file does not read");
        };
        assert!(matches!(
            file.screen().assets.as_slice(),
            [Asset::Image { width, handle: None, pixels: Some(_), .. }] if *width == side
        ));
    }
}
