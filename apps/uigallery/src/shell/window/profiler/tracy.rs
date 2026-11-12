//! Tracy's part of the profiler. Tracy reads the gallery in its own
//! window, the Tracy GUI, over TCP: the client the gallery started before
//! its first frame (`entry.rs`, `tracy` feature) collects only while the
//! GUI is connected. The part says whether it is, and opens the GUI
//! (`PLEV_TRACY`, else `tracy-profiler` on the PATH; v0.14.0, the
//! protocol of the client's `tracy-client-sys` 0.29). Where the client
//! cannot be, it says why: a build without the feature, or the web, where
//! a page opens no TCP socket.

use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use engine::widgets::prelude::{Button, ButtonSize, ButtonVariant, Rect, WidgetEvent};

use super::{caption_style, text};

/// The client is in this build.
const CLIENT: bool = cfg!(all(feature = "tracy", not(target_arch = "wasm32")));

pub(super) struct Tracy {
    open: Button,
    /// What the last press of `open` did.
    said: Option<String>,
}

impl Default for Tracy {
    fn default() -> Self {
        Self {
            open: Button::new("Open Tracy")
                .size(ButtonSize::Sm)
                .variant(ButtonVariant::Outline),
            said: None,
        }
    }
}

impl Tracy {
    fn status() -> &'static str {
        if cfg!(target_arch = "wasm32") {
            "Tracy reads a native app over TCP, which a web page cannot open."
        } else if !CLIENT {
            "Off: build the gallery with --features tracy."
        } else if connected() {
            "The Tracy GUI is reading the gallery: the timeline is in its window."
        } else {
            "The client is waiting: open the Tracy GUI (v0.14.0) on this machine."
        }
    }

    fn button(&self, body: Rect, theme: &Theme) -> Rect {
        let (w, h) = self.open.preferred_size(theme);
        let line = caption_style(theme).line_height;
        Rect::new(body.x, body.y + line + theme.spacing.sm, w, h)
    }

    /// `true` when it changed what it shows.
    pub(super) fn event(&mut self, ev: &WidgetEvent, body: Rect, theme: &Theme) -> bool {
        if !CLIENT {
            return false;
        }
        let r = self.open.handle_event(ev, self.button(body, theme));
        if r.clicked {
            self.said = Some(open_gui());
        }
        r.changed || r.clicked
    }

    pub(super) fn draw(&self, c: &mut Compositor, layer: LayerId, body: Rect, theme: &Theme) {
        let style = caption_style(theme);
        let mid = theme.colors.text_mid.0;
        text(
            c,
            layer,
            Self::status(),
            body.x,
            body.y,
            body.w,
            mid,
            &style,
        );
        if !CLIENT {
            return;
        }
        let at = self.button(body, theme);
        self.open.render_to_layer(c, layer, at, theme);
        if let Some(said) = &self.said {
            let y = at.bottom() + theme.spacing.sm;
            text(c, layer, said, body.x, y, body.w, mid, &style);
        }
    }
}

#[cfg(all(feature = "tracy", not(target_arch = "wasm32")))]
fn connected() -> bool {
    tracy_client::Client::is_connected()
}

#[cfg(not(all(feature = "tracy", not(target_arch = "wasm32"))))]
fn connected() -> bool {
    false
}

/// The Tracy GUI started on this machine's client, left running on its
/// own; what came of it, said.
#[cfg(all(feature = "tracy", not(target_arch = "wasm32")))]
fn open_gui() -> String {
    let program = std::env::var_os("PLEV_TRACY").unwrap_or_else(|| "tracy-profiler".into());
    match std::process::Command::new(&program)
        .args(["-a", "127.0.0.1"])
        .spawn()
    {
        Ok(_) => "Opened: it connects to the gallery on 127.0.0.1.".to_owned(),
        Err(e) => format!(
            "{} did not open ({e}): set PLEV_TRACY to Tracy v0.14.0's tracy-profiler.",
            program.to_string_lossy()
        ),
    }
}

#[cfg(not(all(feature = "tracy", not(target_arch = "wasm32"))))]
fn open_gui() -> String {
    String::new()
}
