//! `InspectorView`: the inspector's screen as plain state. it owns the
//! session it opened and nothing else: no app, nothing that dispatches, no
//! recorder, no socket. the handlers (`input.rs`) move the instant, the
//! pointer and the pick; `render.rs` draws it into a compositor, with or
//! without a window.

use std::path::Path;

use engine::widgets::content::{CodeBlock, CodeLine, EmptyState};
use engine::widgets::feedback::Tooltip;
use engine::widgets::nav::{NavLink, PanelHeader, Sidebar};
use engine::widgets::shell::{AppShell, ShellLayout};
use engine::widgets::{Rect, Theme};
use engine::drawing::compositor::{LayerId, SceneNode};
use engine::styling::theme::Breakpoint;
use replay::{Inspector, Line, prose};
use record::{Bound, FileError, ReadError, Session};
use accord::{Millis, SemId};

use crate::fit::{self, Fit};
use crate::panel::Panel;
use crate::strip::Strip;

/// where each part of the screen goes.
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    pub shell: ShellLayout,
    /// the rect the frame fits into (the frame itself is centered in it
    /// at its own aspect: `InspectorView::fit`).
    pub frame: Rect,
    pub strip: Rect,
    pub panel: Rect,
}

/// the frame on screen at the instant, read once per frame time.
#[derive(Clone, Debug, Default)]
pub(crate) struct Shown {
    pub(crate) frame: Option<Millis>,
    pub(crate) scene: Vec<SceneNode>,
    pub(crate) bound: Vec<Bound>,
    pub(crate) extent: Option<(f32, f32)>,
    /// the chunk did not read back (`rec::ReadError`), said as it is.
    pub(crate) error: Option<String>,
}

/// what a press started on, until it is let go.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Press {
    Scrub,
    Track,
    /// the frame, over this control (or over none).
    Frame(Option<SemId>),
}

/// why a session did not open.
#[derive(Debug)]
pub enum OpenError {
    /// the file does not read as a session (`rec::FileError`, as it is:
    /// an animation, a bad magic, a truncated file).
    File(FileError),
    /// the file reads, but its record does not replay.
    Read(ReadError),
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenError::File(e) => write!(f, "{e}"),
            OpenError::Read(e) => write!(f, "the session does not replay: {e}"),
        }
    }
}

impl std::error::Error for OpenError {}

/// the keys the view answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    /// to the instant of the event before.
    Previous,
    /// to the instant of the event after.
    Next,
    First,
    Last,
    /// drop the pick (and close the drawer).
    Close,
}

/// the inspector's screen.
#[derive(Debug)]
pub struct InspectorView {
    pub(crate) session: Session,
    pub(crate) theme: Theme,
    pub(crate) shell: AppShell,
    pub(crate) strip: Strip,
    pub(crate) requests: Vec<Line>,
    pub(crate) start: Millis,
    pub(crate) end: Millis,
    /// the instant, on the recorder's clock.
    pub(crate) t: Millis,
    pub(crate) shown: Shown,
    pub(crate) pointer: Option<(f32, f32)>,
    pub(crate) hover: Option<SemId>,
    pub(crate) tip: Tooltip,
    pub(crate) press: Option<Press>,
    pub(crate) panel: Option<Result<Panel, String>>,
    /// the panel's rows as the model wrote them; the block holds them
    /// wrapped to its width.
    pub(crate) lines: Vec<CodeLine>,
    pub(crate) block: CodeBlock,
    pub(crate) empty: EmptyState,
    pub(crate) overlay: Option<LayerId>,
}

impl InspectorView {
    /// the session at `path`, on a window of `width` by `height` logical
    /// pixels.
    pub fn open(path: &Path, width: f32, height: f32) -> Result<InspectorView, OpenError> {
        let session = Session::open(path).map_err(OpenError::File)?;
        let name = path.file_name().map_or_else(
            || path.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        );
        InspectorView::new(session, &name, width, height).map_err(OpenError::Read)
    }

    /// a session already read, named `name` in the header.
    pub fn new(
        session: Session,
        name: &str,
        width: f32,
        height: f32,
    ) -> Result<InspectorView, ReadError> {
        let inspector = Inspector::new(&session);
        let summary = inspector.summary()?;
        let strip = Strip::new(&inspector.timeline());
        let (start, end) = (inspector.start(), inspector.end());
        let links = summary
            .requests
            .iter()
            .map(|l| NavLink::new(format!("{} {} {}", l.id, l.action, l.target)).hint(l.ending))
            .collect();
        let mut sidebar = Sidebar::new(links).brand("inspector", "reads a session, never runs it");
        sidebar.footer = vec![format!("build {} {}", summary.version, summary.git)];
        // no request is chosen yet: none of them lit.
        for link in &mut sidebar.links {
            link.active = false;
        }
        let header = PanelHeader::new(name);
        let mut view = InspectorView {
            theme: Theme::default(),
            shell: AppShell::new(sidebar, header, width, height),
            strip,
            requests: summary.requests,
            start,
            end,
            t: start,
            shown: Shown::default(),
            pointer: None,
            hover: None,
            tip: Tooltip::new("").delay(0.0),
            press: None,
            panel: None,
            lines: Vec::new(),
            block: CodeBlock::lines(Vec::new()),
            empty: EmptyState::new(
                "no control picked",
                "click a control on the frame to see what it was, its state, who observed what, the request, the verdict and the result",
            ),
            overlay: None,
            session,
        };
        view.refresh();
        Ok(view)
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn size(&self) -> (f32, f32) {
        self.shell.size()
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.shell.resize(width, height);
        self.wrap_rows();
        self.refresh_hover();
    }

    /// the instant, on the recorder's clock.
    pub fn t(&self) -> Millis {
        self.t
    }

    /// the instant, from the start of the session.
    pub fn at(&self) -> Millis {
        self.t.saturating_sub(self.start)
    }

    /// the session's first and last instants, on the recorder's clock.
    pub fn span(&self) -> (Millis, Millis) {
        (self.start, self.end)
    }

    /// the requests in the list, in the order they were asked.
    pub fn requests(&self) -> &[Line] {
        &self.requests
    }

    pub fn strip(&self) -> &Strip {
        &self.strip
    }

    /// the control under the pointer, in the frame on screen.
    pub fn hovered(&self) -> Option<&SemId> {
        self.hover.as_ref()
    }

    /// the control picked, whose answers the panel shows.
    pub fn picked(&self) -> Option<&SemId> {
        match &self.panel {
            Some(Ok(p)) => Some(p.id()),
            _ => None,
        }
    }

    pub fn panel(&self) -> Option<&Panel> {
        self.panel.as_ref().and_then(|p| p.as_ref().ok())
    }

    /// what the panel reads, row by row, as the model wrote it.
    pub fn rows(&self) -> &[CodeLine] {
        &self.lines
    }

    /// the same rows as the panel shows them, wrapped to its width.
    pub fn shown_rows(&self) -> &[CodeLine] {
        &self.block.rows
    }

    /// the time of the frame on screen at the instant; `None` before the
    /// first frame.
    pub fn frame_time(&self) -> Option<Millis> {
        self.shown.frame
    }

    /// where each control shows in the frame on screen, in stacking order.
    pub fn bound(&self) -> &[Bound] {
        &self.shown.bound
    }

    pub fn layout(&self) -> Layout {
        let theme = &self.theme;
        let shell = self.shell.layout(theme);
        let c = shell.content;
        let gap = theme.spacing.lg;
        if shell.breakpoint == Breakpoint::Expanded {
            let panel_w = theme.size.readable_max_w.min((c.w - gap) / 2.0).max(0.0);
            let left_w = (c.w - panel_w - gap).max(0.0);
            let strip_h = self.strip.height(left_w, theme);
            let frame = Rect::new(c.x, c.y, left_w, (c.h - strip_h - gap).max(0.0));
            let strip = Rect::new(c.x, frame.bottom() + gap, left_w, strip_h);
            let panel = Rect::new(c.x + left_w + gap, c.y, panel_w, c.h);
            return Layout {
                shell,
                frame,
                strip,
                panel,
            };
        }
        // one column: the frame at its aspect, taking at most half of what
        // the strip leaves; the panel gets the rest.
        let strip_h = self.strip.height(c.w, theme);
        let left = (c.h - strip_h - gap * 2.0).max(0.0);
        let (ew, eh) = self.shown.extent.unwrap_or((c.w, left / 2.0));
        let aspect = if ew > 0.0 { eh / ew } else { 0.0 };
        let frame_h = (c.w * aspect).min(left / 2.0).max(0.0);
        let frame = Rect::new(c.x, c.y, c.w, frame_h);
        let strip = Rect::new(c.x, frame.bottom() + gap, c.w, strip_h);
        let top = strip.bottom() + gap;
        let panel = Rect::new(c.x, top, c.w, (c.y + c.h - top).max(0.0));
        Layout {
            shell,
            frame,
            strip,
            panel,
        }
    }

    /// how the frame on screen fits its rect; `None` without a frame.
    pub fn fit(&self) -> Option<Fit> {
        let extent = self.shown.extent?;
        Some(Fit::into(self.layout().frame, extent))
    }

    /// where `id` shows in the view now, over the frame on screen.
    pub fn control_rect(&self, id: &SemId) -> Option<Rect> {
        let fit = self.fit()?;
        let b = self.shown.bound.iter().find(|b| b.id == *id)?;
        Some(fit.rect(b.rect))
    }

    /// moves the instant to `t` (clamped to the session), and everything
    /// that reads it: the frame, the header, the panel, the hover.
    pub fn seek(&mut self, t: Millis) -> bool {
        let t = t.clamp(self.start, self.end);
        let moved = t != self.t;
        self.t = t;
        self.strip.scrub.set_value(self.at() as f32);
        self.refresh();
        moved
    }

    /// picks `id` (or drops the pick): the panel shows its answers at the
    /// instant.
    pub fn pick(&mut self, id: Option<SemId>) -> bool {
        let unchanged = match (&self.panel, &id) {
            (None, None) => true,
            (Some(Ok(p)), Some(id)) => p.id() == id,
            _ => false,
        };
        if unchanged {
            return false;
        }
        self.panel = id.map(|id| self.answer(&id));
        self.block.scroll = 0.0;
        self.fill_rows();
        true
    }

    /// the request at `index` of the list: the instant of its first stage,
    /// its target picked.
    pub fn go_to_request(&mut self, index: usize) -> bool {
        let Some(line) = self.requests.get(index) else {
            return false;
        };
        let (t, target) = (line.t, line.target.clone());
        self.shell.sidebar.set_active(index);
        self.shell.sidebar.drawer_open = false;
        self.seek(t);
        self.pick(Some(target));
        true
    }

    fn answer(&self, id: &SemId) -> Result<Panel, String> {
        Panel::of(&Inspector::new(&self.session), id, self.t).map_err(|e| e.to_string())
    }

    fn fill_rows(&mut self) {
        self.lines = match &self.panel {
            Some(Ok(panel)) => panel.rows(),
            Some(Err(e)) => vec![CodeLine::new(
                format!("the session does not read at this instant: {e}"),
                engine::widgets::content::LineKind::Removed,
            )],
            None => Vec::new(),
        };
        self.wrap_rows();
    }

    /// the model's rows wrapped to the width the block gives its text: the
    /// block's width less its inset, which is where its rows start under
    /// its top edge (the same on every side).
    fn wrap_rows(&mut self) {
        let panel = self.layout().panel;
        let inset = self.block.rows_rect(panel, &self.theme).y - panel.y;
        let width = (panel.w - inset * 2.0).max(0.0);
        let style = CodeBlock::text_style(&self.theme);
        self.block.rows = crate::panel::wrap(&self.lines, width, &style);
    }

    /// what reads the instant, read again.
    pub(crate) fn refresh(&mut self) {
        let frame = self.session.frame_at(self.t);
        if frame != self.shown.frame {
            self.shown = self.read_frame(frame);
        }
        let frame_text = match self.shown.frame {
            Some(f) => format!("frame {}", text::stamp(f.saturating_sub(self.start))),
            None => "no frame yet".to_owned(),
        };
        self.shell.header.blurb = Some(format!(
            "{} of {}, {frame_text}",
            text::stamp(self.at()),
            text::stamp(self.end.saturating_sub(self.start)),
        ));
        if let Some(id) = self.picked().cloned() {
            self.panel = Some(self.answer(&id));
            self.fill_rows();
        }
        self.refresh_hover();
    }

    fn read_frame(&self, frame: Option<Millis>) -> Shown {
        let bound = self.session.bindings_at(self.t).unwrap_or_default();
        match self.session.scene_at(self.t) {
            Ok(scene) => {
                let scene = scene.unwrap_or_default();
                Shown {
                    frame,
                    extent: fit::extent(&scene, &bound),
                    scene,
                    bound,
                    error: None,
                }
            }
            Err(e) => Shown {
                frame,
                extent: fit::extent(&[], &bound),
                scene: Vec::new(),
                bound,
                error: Some(e.to_string()),
            },
        }
    }

    /// the control under the pointer, read again (the instant, the layout
    /// or the pointer moved).
    pub(crate) fn refresh_hover(&mut self) -> bool {
        let over = self.pointer.and_then(|(x, y)| {
            let fit = self.fit()?;
            let (fx, fy) = fit.to_frame(x, y)?;
            self.session.hit_at(self.t, fx, fy)
        });
        if over == self.hover {
            if let Some(r) = over.as_ref().and_then(|id| self.control_rect(id)) {
                self.tip.set_hover(true, r);
            }
            return false;
        }
        self.tip.set_hover(false, Rect::default());
        if let Some(id) = &over {
            self.tip.text = id.to_string();
            let anchor = self.control_rect(id).unwrap_or_default();
            self.tip.set_hover(true, anchor);
            self.tip.tick(0.0);
        }
        self.hover = over;
        true
    }
}
