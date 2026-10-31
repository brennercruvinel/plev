//! the scene as data that travels (task-79,
//! docs/decisoes/gravacao/0065-scene-travels-as-data.md): a window's frame written layer by
//! layer, so what did not change does not travel, and read back into a
//! compositor of another process that draws the same pixels.
//!
//! it is not the monster path of the visual track: the monster IR has no
//! node for a clip, a backdrop blur or a shadow and keeps what it carries
//! on the twips and rgba8 grids, so the frame it gives back is not the
//! frame drawn. this codec writes every node as it is, every `f32` as its
//! bits.
//!
//! the bytes, every integer little-endian, an `f32` as its bits, a string
//! as its byte length (`u32`) then its UTF-8, an `Option` as `0` or `1`
//! then the value:
//!
//! ```text
//! head   := width f32, height f32, scale f32, clear 4 f32 (linear)
//! body   := key u32 (the app's LayerId), props, count u32, node*
//! patch  := props, head u32, tail u32, count u32, node*
//!           (the receiver's nodes: its first `head`, the nodes written,
//!           its last `tail`)
//! props  := z i32, visible u8, opacity f32, clip opt (4 u32),
//!           count u32, effect*  (1 blur: sigma; 2 shadow: sigma, color)
//! node   := 1 rect x y w h color | 2 rounded rect x y w h color radius
//!           border_width border_color | 3 gradient rect x y w h color
//!           color2 angle radius border_width border_color | 4 text key
//!           x y color | 5 path asset_key u64 | 6 image x y w h radius
//!           asset_key u64 | 7 push clip x y w h | 8 pop clip | 9
//!           backdrop blur x y w h radius sigma | 10 shadow x y w h
//!           radius blur_radius offset(2) color inset u8
//! key    := text str, font_size u32, line_height u32, max_width opt u32,
//!           weight u16, letter_spacing u32, family opt str
//! asset  := 1 image: id u32, atlas_x u32, atlas_y u32, width u32,
//!           height u32, pixels u8 (0 none, 1 then width * height * 4 of
//!           RGBA8, 2 a stream's: never, and its id written 0)
//!         | 2 path: engine hash u64, count u32, (position 2 f32, color
//!           4 f32)*, count u32, index u32*
//! ```
//!
//! a layer's hash is FNV-1a 64 of its body; a fixed image's asset key is
//! its handle (one per content), a stream image's its place and size (the
//! same for all its frames), a path's its engine hash and sizes.
//!
//! a stream image (`engine::drawing::gpu::image_is_stream`: a camera's frames)
//! never travels with its pixels, whoever watches: the capture ADR keeps
//! a camera's frames from every agent, and a watcher is an agent. the
//! receiver draws a box in its place that says so ([`LIVE_LABEL`]). a
//! text travels as its key, the string and the typography: the receiver
//! shapes and rasterizes it with the same embedded fonts.
//!
//! a secret never travels either (task-79 phase 2): the app names the
//! places its secrets are bound to ([`Mask`]), and the sender leaves what
//! a secret draws there out of the layer before anything is written,
//! with a box that says so ([`SECRET_LABEL`]) in its stead
//! ([`Sender::frame_masked`]).
//!
//! what the receiver holds follows two rules the sender keeps too: a
//! layer is held at the hash of its last body; an asset is held while
//! each frame that comes references it, and let go by the first that
//! does not.

mod asset;
mod bytes;
mod mask;
mod mirror;
mod node;
mod send;

pub use mask::{Mask, SECRET_LABEL};
pub use mirror::{Brought, ImageCounts, Incoming, LIVE_LABEL, Mirror};
pub use node::{Drawn, Head, Props};
pub use send::{Frame, LayerOut, Sender};

/// a scene that does not read back as it claims.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum SceneError {
    #[error("the scene ends inside {0}")]
    Truncated(&'static str),
    #[error("{0} is not what the sender writes")]
    Invalid(&'static str),
    #[error("{0} has bytes past its last field")]
    Trailing(&'static str),
    #[error("unknown tag {0}")]
    Tag(u8),
    /// a frame names a layer at a hash, or patches one, that the
    /// receiver does not hold: it has to start over from a whole frame.
    #[error("layer {0} is not held at the hash the frame names")]
    NotHeld(u32),
    /// a body, or what a patch makes, does not hash to what the frame
    /// says.
    #[error("layer {0} does not hash to what the frame says")]
    Hash(u32),
}

#[cfg(test)]
mod tests;
