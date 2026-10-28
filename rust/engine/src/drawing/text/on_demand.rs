//! The Noto faces a target fetches when its text first needs them (issue
//! #137). A desktop, android and ios embed them (`super::fonts`); the web
//! build leaves them out of the wasm, which they would weigh about 5 MB in
//! gzip, and fetches each script's files the first time a text holds it.
//!
//! The bytes are the same subsets the desktop embeds, served beside the
//! build, and each file must hash to the sha256 written here before it is
//! taken; a test holds this table to the files the desktop embeds. A pack
//! that arrives bumps a generation. Each font system (the measurer's and
//! the renderer's) compares it with the one it last saw and, when it moved,
//! loads every arrived face again in the order of [`PACKS`], the order the
//! desktop loads them in, whatever the order the network delivered them in
//! (emoji last). Then it drops what it measured or shaped without them.
//!
//! The loop sleeps between frames, so whoever runs it registers a waker
//! ([`on_faces_arrived`]) to draw again when a face arrives.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

use cosmic_text::fontdb::{Database, ID, Source};
use sha2::{Digest, Sha256};

/// One script's files, in load order, with the sha256 each must hash to.
#[cfg_attr(
    not(any(target_arch = "wasm32", test)),
    allow(dead_code, reason = "only the web build and the tests deliver packs")
)]
pub(super) struct Pack {
    pub(super) name: &'static str,
    pub(super) files: &'static [(&'static str, &'static str)],
}

/// Every pack, in the order their faces load: the order of the embedded
/// `NOTO` faces, then emoji.
pub(super) const PACKS: [Pack; 7] = [
    Pack {
        name: "arabic",
        files: &[
            (
                "NotoSansArabic-Regular.ttf",
                "4d5ebe3af0a0066cf1677dad433b7fbbb88ba5157bb6236259a23ffba11c6506",
            ),
            (
                "NotoSansArabic-Bold.ttf",
                "eb71407443397b5d188808fce081b4c3b2e817805e1a53500116edcd2f90dfad",
            ),
        ],
    },
    Pack {
        name: "hebrew",
        files: &[
            (
                "NotoSansHebrew-Regular.ttf",
                "52ad6bf2315456d7a69d3f2ade8562bbab12ef248c13e5b82650ae497ee7a00b",
            ),
            (
                "NotoSansHebrew-Bold.ttf",
                "6e93414b946e699944f141578ae93e51675fc9d2059816bb8c585f4107a02080",
            ),
        ],
    },
    Pack {
        name: "devanagari",
        files: &[
            (
                "NotoSansDevanagari-Regular.ttf",
                "4c8cd82080e22dd03d82d95b561beadc77828c07c813a08ee893471e6b0f2719",
            ),
            (
                "NotoSansDevanagari-Bold.ttf",
                "ca6afad47f1d725e6940d03a051dcd4b2892c29073ceb588b9147c9c05c4f8a6",
            ),
        ],
    },
    Pack {
        name: "sc",
        files: &[
            (
                "NotoSansSC-Regular.otf",
                "1ac388542a633b12d462c6dfb64fcd28190470b3dcb0b9a3f554216857009646",
            ),
            (
                "NotoSansSC-Bold.otf",
                "54c453a713e43b7175bd220eb0a23d83c5c84e89979a1e06103b3c09e405cbe2",
            ),
        ],
    },
    Pack {
        name: "jp",
        files: &[
            (
                "NotoSansJP-Regular.otf",
                "c969154a46eae59b2708e6bed7a827e52ce3d9e457b30b900100e2221acdb68a",
            ),
            (
                "NotoSansJP-Bold.otf",
                "1ce6d6b9fae3c13da3319f82b5a4d56fc87eadecdae863e15e476498fd5cd196",
            ),
        ],
    },
    Pack {
        name: "kr",
        files: &[
            (
                "NotoSansKR-Regular.otf",
                "fbd8cef4ea33aba89dbe21e5399ecbfad90afcda24523244f89e81f2513a72bf",
            ),
            (
                "NotoSansKR-Bold.otf",
                "5384076cb74a1cd16cd5997066f6e0b859eaf1df9d2b8dd4ef67fa874b90377a",
            ),
        ],
    },
    Pack {
        name: "emoji",
        files: &[(
            "NotoColorEmoji-CBDT.ttf",
            "73436441dd57d3480ab1a5bec03feef2e501501655b6542a5a983153ddf78c36",
        )],
    },
];

const ARABIC: usize = 0;
const HEBREW: usize = 1;
const DEVANAGARI: usize = 2;
const SC: usize = 3;
const JP: usize = 4;
const KR: usize = 5;
const EMOJI: usize = 6;

/// Whether this target fetches the packs: only the web build leaves them
/// out of the binary. Tests turn it on for their thread (`fetching`).
fn fetches() -> bool {
    cfg!(target_arch = "wasm32") || FETCHING.with(Cell::get)
}

/// The packs `text` needs, as a bit per index of [`PACKS`]. Han asks for
/// SC, which answers for it on the desktop too (SC loads before JP); kana
/// asks for JP, so a Japanese text brings both, as the desktop holds both.
pub(super) fn packs_of(text: &str) -> u8 {
    if text.is_ascii() {
        return 0;
    }
    let mut mask = 0u8;
    for c in text.chars() {
        let pack = match c as u32 {
            0x0600..=0x06FF
            | 0x0750..=0x077F
            | 0x08A0..=0x08FF
            | 0xFB50..=0xFDFF
            | 0xFE70..=0xFEFF => ARABIC,
            0x0590..=0x05FF | 0xFB1D..=0xFB4F => HEBREW,
            0x0900..=0x097F | 0xA8E0..=0xA8FF => DEVANAGARI,
            0x3040..=0x30FF | 0x31F0..=0x31FF | 0xFF66..=0xFF9F => JP,
            0x1100..=0x11FF | 0x3130..=0x318F | 0xAC00..=0xD7AF => KR,
            0x2E80..=0x2FDF
            | 0x3000..=0x303F
            | 0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF
            | 0x20000..=0x3134F => SC,
            0x2600..=0x27BF | 0x1F000..=0x1FAFF => EMOJI,
            _ => continue,
        };
        mask |= 1 << pack;
    }
    mask
}

#[derive(Default)]
struct State {
    /// Packs asked for, a bit per index of [`PACKS`]; never asked twice.
    asked: u8,
    /// The files of each pack that arrived and hashed right.
    arrived: [Option<Vec<Arc<Vec<u8>>>>; 7],
    /// Moves with every arrival; font systems compare it with theirs.
    generation: u64,
    wakers: Vec<Rc<dyn Fn()>>,
    /// Where the files are served, relative to the page by default.
    base: Option<String>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
    static FETCHING: Cell<bool> = const { Cell::new(false) };
}

/// Asks for the packs `text` needs that were never asked for. Nothing on a
/// target that embeds them.
pub(super) fn want(text: &str) {
    if !fetches() {
        return;
    }
    let mask = packs_of(text);
    let new = STATE.with(|s| {
        let mut s = s.borrow_mut();
        let new = mask & !s.asked;
        s.asked |= new;
        new
    });
    for (i, _) in PACKS
        .iter()
        .enumerate()
        .filter(|(i, _)| new & (1 << i) != 0)
    {
        fetch(i);
    }
}

/// The generation of the arrived faces: a font system that saw another
/// loads them again ([`load`]).
pub(super) fn generation() -> u64 {
    STATE.with(|s| s.borrow().generation)
}

/// Takes the files of pack `pack` when every one hashes to its sha256,
/// then wakes the loop. A file that does not leaves the pack out.
#[cfg_attr(
    not(any(target_arch = "wasm32", test)),
    allow(dead_code, reason = "only the web build and the tests deliver packs")
)]
pub(super) fn arrive(pack: usize, files: Vec<Vec<u8>>) -> Result<(), String> {
    let spec = PACKS.get(pack).ok_or_else(|| format!("no pack {pack}"))?;
    if files.len() != spec.files.len() {
        return Err(format!(
            "{}: {} files, {} expected",
            spec.name,
            files.len(),
            spec.files.len()
        ));
    }
    for (bytes, (name, sha)) in files.iter().zip(spec.files) {
        let got = hex(&Sha256::digest(bytes));
        if got != *sha {
            return Err(format!("{name}: sha256 {got}, {sha} expected"));
        }
    }
    let wakers = STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.arrived[pack] = Some(files.into_iter().map(Arc::new).collect());
        s.generation += 1;
        s.wakers.clone()
    });
    for wake in wakers {
        wake();
    }
    Ok(())
}

/// Removes the faces `loaded` from `db` and loads every arrived face again,
/// in the order of [`PACKS`]; returns their ids, which replace `loaded`.
pub(super) fn load(db: &mut Database, loaded: &[ID]) -> Vec<ID> {
    for id in loaded {
        db.remove_face(*id);
    }
    let files: Vec<Arc<Vec<u8>>> = STATE.with(|s| {
        s.borrow()
            .arrived
            .iter()
            .flatten()
            .flat_map(|files| files.iter().cloned())
            .collect()
    });
    let mut ids = Vec::new();
    for bytes in files {
        ids.extend(db.load_font_source(Source::Binary(bytes)));
    }
    ids
}

/// Calls `wake` on this thread every time a pack arrives, so a loop that
/// sleeps between frames draws the text with its faces. A web shell
/// registers it once, beside its event loop.
pub fn on_faces_arrived(wake: impl Fn() + 'static) {
    STATE.with(|s| s.borrow_mut().wakers.push(Rc::new(wake)));
}

/// Where the web build serves the Noto files; `fonts/` beside the page by
/// default. Set it before the first text is shaped.
pub fn set_faces_url(base: impl Into<String>) {
    STATE.with(|s| s.borrow_mut().base = Some(base.into()));
}

#[cfg_attr(
    not(any(target_arch = "wasm32", test)),
    allow(dead_code, reason = "only the web build and the tests deliver packs")
)]
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(target_arch = "wasm32")]
fn fetch(pack: usize) {
    let base = STATE
        .with(|s| s.borrow().base.clone())
        .unwrap_or_else(|| "fonts/".into());
    wasm_bindgen_futures::spawn_local(async move {
        let spec = &PACKS[pack];
        let mut files = Vec::with_capacity(spec.files.len());
        for (name, _) in spec.files {
            match fetch_bytes(&format!("{base}{name}")).await {
                Ok(bytes) => files.push(bytes),
                Err(e) => {
                    log::warn!("fonts: {} left out, {e}", spec.name);
                    return;
                }
            }
        }
        if let Err(e) = arrive(pack, files) {
            log::warn!("fonts: {} left out, {e}", spec.name);
        }
    });
}

#[cfg(target_arch = "wasm32")]
async fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    let window = web_sys::window().ok_or("no window")?;
    let response = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|e| format!("{url}: {e:?}"))?;
    let response: web_sys::Response = response
        .dyn_into()
        .map_err(|_| format!("{url}: not a response"))?;
    if !response.ok() {
        return Err(format!("{url}: HTTP {}", response.status()));
    }
    let buffer = response
        .array_buffer()
        .map_err(|e| format!("{url}: {e:?}"))?;
    let buffer = JsFuture::from(buffer)
        .await
        .map_err(|e| format!("{url}: {e:?}"))?;
    Ok(js_sys::Uint8Array::new(&buffer).to_vec())
}

/// Off the web nothing is fetched; a test delivers the packs itself
/// through [`arrive`].
#[cfg(not(target_arch = "wasm32"))]
fn fetch(_pack: usize) {}

/// Turns fetching on for this thread's tests, from a clean state.
#[cfg(test)]
pub(super) fn fetching() {
    FETCHING.with(|f| f.set(true));
    STATE.with(|s| *s.borrow_mut() = State::default());
}

/// The packs asked for on this thread (tests).
#[cfg(test)]
pub(super) fn asked() -> u8 {
    STATE.with(|s| s.borrow().asked)
}
