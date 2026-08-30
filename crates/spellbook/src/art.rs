//! Card artwork: cache-relative path in, `ImageHandle` out.
//!
//! The web build asked the browser for `/img/<path>` and let the HTTP cache
//! sort it out. There is no browser and no server, so this is the piece that
//! replaces both: a screen asks for the art it is about to draw, and gets back
//! either a handle it can put in a `SceneNode::Image` or nothing, in which case
//! it draws the placeholder and the art appears on a later frame.
//!
//! Three rules follow from how the engine and the loop work:
//!
//! - The UI thread never reads or decodes a file. A grid can want a hundred
//!   images at once, and a hundred disk reads plus JPEG decodes inside one
//!   frame is a visible stall. Both happen on the worker thread; what arrives
//!   here is already RGBA.
//! - Art is downscaled to the size it will be drawn at, on the worker thread.
//!   The engine packs every image into one atlas that stops growing at
//!   MAX_IMAGE_ATLAS_SIZE (8192), and Scryfall art_crop is 626x457 - about 220
//!   of them fill it, which a collection of any size would blow through. A
//!   tile drawn 300px wide has no use for the other 326 columns.
//! - A miss is not an error. Art that is not in the cache yet is normal - the
//!   updater fills 38k files over a long download - so a miss is recorded and
//!   never re-requested in a loop, which is what a naive "ask every frame"
//!   cache would do sixty times a second.

use std::collections::HashMap;

use engine::gpu::ImageHandle;

/// Decoded, already-downscaled artwork as it comes back from the worker.
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// What the cache knows about one image.
enum State {
    /// Asked for, not answered yet.
    Pending,
    Ready(ImageHandle),
    /// Not on disk, or the bytes would not decode. Asked for once, never again
    /// this session.
    Missing,
}

#[derive(Default)]
pub struct ArtCache {
    entries: HashMap<String, State>,
    /// Paths to ask the worker thread for, drained by the shell each frame.
    wanted: Vec<String>,
}

impl ArtCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// The handle for `rel`, requesting it if this is the first time it has
    /// been asked for. Returns `None` while it is in flight or missing - the
    /// caller draws its placeholder and gets the art on a later frame.
    pub fn get(&mut self, rel: &str) -> Option<ImageHandle> {
        match self.entries.get(rel) {
            Some(State::Ready(h)) => Some(*h),
            Some(State::Pending | State::Missing) => None,
            None => {
                self.entries.insert(rel.to_string(), State::Pending);
                self.wanted.push(rel.to_string());
                None
            }
        }
    }

    /// Paths the worker thread should load, handed over once.
    pub fn take_requests(&mut self) -> Vec<String> {
        std::mem::take(&mut self.wanted)
    }

    /// Decoded pixels back from the worker. `None` means the file was not
    /// there or would not decode. Returns `true` when the frame needs
    /// redrawing, which is only when art actually became available.
    pub fn resolve(&mut self, rel: &str, art: Option<Rgba>) -> bool {
        let state = match art {
            Some(a) => match engine::gpu::load_image_rgba(a.width, a.height, a.pixels) {
                Ok(handle) => State::Ready(handle),
                Err(e) => {
                    // Atlas full, or a size the atlas cannot take. Recorded as
                    // a miss so the tile draws its placeholder rather than the
                    // frame failing.
                    log::warn!("card art {rel} did not fit the atlas: {e}");
                    State::Missing
                }
            },
            None => State::Missing,
        };
        let ready = matches!(state, State::Ready(_));
        self.entries.insert(rel.to_string(), state);
        ready
    }

    /// Forget a miss so it is retried - after a data update has filled the
    /// cache, the art that was missing all session is now there.
    pub fn retry_missing(&mut self) {
        self.entries.retain(|_, s| !matches!(s, State::Missing));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The first ask queues one request; asking again while it is in flight
    /// must not queue another. Screens call `get` inside layout, so this runs
    /// on every frame for every visible card.
    #[test]
    fn an_image_is_requested_once_however_often_it_is_asked_for() {
        let mut cache = ArtCache::new();
        for _ in 0..60 {
            assert!(cache.get("art_crop/front/a/4/x.jpg").is_none());
        }
        assert_eq!(cache.take_requests(), vec!["art_crop/front/a/4/x.jpg"]);
        assert!(cache.take_requests().is_empty());
    }

    /// A miss is remembered. Without this the cache would re-ask for every
    /// uncached card sixty times a second, which on a fresh install is most
    /// of the collection.
    #[test]
    fn a_missing_image_is_not_asked_for_again() {
        let mut cache = ArtCache::new();
        cache.get("art_crop/front/a/4/x.jpg");
        cache.take_requests();
        assert!(!cache.resolve("art_crop/front/a/4/x.jpg", None));

        assert!(cache.get("art_crop/front/a/4/x.jpg").is_none());
        assert!(
            cache.take_requests().is_empty(),
            "a known miss must not be re-requested"
        );
    }

    /// Until a data update fills the cache. Then the misses are worth another
    /// try, and only the misses - anything already decoded stays.
    #[test]
    fn retrying_clears_only_the_misses() {
        let mut cache = ArtCache::new();
        cache.get("a.jpg");
        cache.get("b.jpg");
        cache.take_requests();
        cache.resolve("a.jpg", None);

        cache.retry_missing();
        assert!(cache.get("a.jpg").is_none());
        assert_eq!(cache.take_requests(), vec!["a.jpg"], "b was still pending");
    }
}
