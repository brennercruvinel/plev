//! One card's widget on the Comments page, and what the page asks of any
//! of them: its size, its events, its draw, its projection, the part a
//! reader names, the line it types in.

use accord::{SemId, SnapshotBuilder, Value};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::message::comment::CommentThread;
use engine::widgets::message::compose::ComposeLine;
use engine::widgets::message::marker::Marker;
use engine::widgets::message::new_comment::NewComment;
use engine::widgets::message::quick_comment::QuickComment;
use engine::widgets::prelude::{EventResult, Rect, Wake, WidgetEvent};

pub(super) enum Sample {
    Marker(Marker),
    Thread(Box<CommentThread>),
    New(Box<NewComment>),
    Quick(Box<QuickComment>),
}

impl Sample {
    /// Its ids under the card `card`.
    pub fn identify(&mut self, card: &SemId) {
        let id = |name: &str| card.child(name).ok();
        let placeholder = Sample::Marker(Marker::pin(Vec::<String>::new()));
        *self = match std::mem::replace(self, placeholder) {
            Sample::Marker(m) => match id("marker") {
                Some(i) => Sample::Marker(m.sem(i)),
                None => Sample::Marker(m),
            },
            Sample::Thread(t) => match id("thread") {
                Some(i) => Sample::Thread(Box::new((*t).sem(i))),
                None => Sample::Thread(t),
            },
            Sample::New(n) => match id("new") {
                Some(i) => Sample::New(Box::new((*n).sem(i))),
                None => Sample::New(n),
            },
            Sample::Quick(q) => match id("quick") {
                Some(i) => Sample::Quick(Box::new((*q).sem(i))),
                None => Sample::Quick(q),
            },
        };
    }

    pub fn size(&self, avail: f32, theme: &Theme) -> (f32, f32) {
        match self {
            Sample::Marker(m) => m.size(theme),
            Sample::Thread(t) => t.size(avail, theme),
            Sample::New(n) => n.size(avail, theme),
            Sample::Quick(q) => q.size(avail, theme),
        }
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, at: Rect, theme: &Theme) -> EventResult {
        match self {
            Sample::Marker(m) => m.handle_event(event, at, theme),
            Sample::Thread(t) => t.handle_event(event, at, theme),
            Sample::New(n) => n.handle_event(event, at, theme),
            Sample::Quick(q) => q.handle_event(event, at, theme),
        }
    }

    pub fn render(&self, c: &mut Compositor, at: Rect, theme: &Theme) {
        match self {
            Sample::Marker(m) => m.render(c, at, theme),
            Sample::Thread(t) => t.render(c, at, theme),
            Sample::New(n) => n.render(c, at, theme),
            Sample::Quick(q) => q.render(c, at, theme),
        }
    }

    pub fn wake(&mut self, dt: f32) -> Wake {
        match self {
            Sample::Marker(_) => Wake::idle(),
            Sample::Thread(t) => t.wake(dt),
            Sample::New(n) => n.wake(dt),
            Sample::Quick(q) => q.wake(dt),
        }
    }

    pub fn line(&self) -> Option<&ComposeLine> {
        match self {
            Sample::Marker(_) => None,
            Sample::Thread(t) => Some(t.reply()),
            Sample::New(n) => Some(n.line()),
            Sample::Quick(q) => Some(q.line()),
        }
    }

    pub fn line_mut(&mut self) -> Option<&mut ComposeLine> {
        match self {
            Sample::Marker(_) => None,
            Sample::Thread(t) => Some(t.reply_mut()),
            Sample::New(n) => Some(n.line_mut()),
            Sample::Quick(q) => Some(q.line_mut()),
        }
    }

    /// Where its line is, in a box at `at`.
    pub fn line_rect(&self, at: Rect, theme: &Theme) -> Option<Rect> {
        match self {
            Sample::Marker(_) => None,
            Sample::Thread(t) => t.is_open().then(|| t.layout(at, theme).line),
            Sample::New(n) => Some(n.layout(at, theme).line),
            Sample::Quick(q) => Some(q.layout(at, theme).line),
        }
    }

    pub fn ime_area(&self, at: Rect, theme: &Theme) -> Option<Rect> {
        match self {
            Sample::Marker(_) => None,
            Sample::Thread(t) => Some(t.ime_area(at, theme)),
            Sample::New(n) => Some(n.ime_area(at, theme)),
            Sample::Quick(q) => Some(q.ime_area(at, theme)),
        }
    }

    /// Lets its focus go. `true` when it had it.
    pub fn blur(&mut self) -> bool {
        match self {
            Sample::Marker(m) => {
                let had = m.is_focused();
                m.set_focused(false);
                had
            }
            Sample::Thread(t) => {
                let had = t.focused().is_some();
                t.focus(None);
                had
            }
            Sample::New(n) => {
                let had = n.focused().is_some();
                n.focus(None);
                had
            }
            Sample::Quick(q) => {
                let had = q.focused().is_some();
                q.focus(None);
                had
            }
        }
    }

    /// The focus to `id`, when it is one of its parts.
    pub fn focus(&mut self, id: &SemId) {
        match self {
            Sample::Marker(m) => {
                if m.sem_id() == Some(id) {
                    m.set_focused(true);
                }
            }
            Sample::Thread(t) => {
                if let Some(part) = t.part_of(id) {
                    t.focus(Some(part));
                }
            }
            Sample::New(n) => {
                if let Some(part) = n.part_of(id) {
                    n.focus(Some(part));
                }
            }
            Sample::Quick(q) => {
                if let Some(part) = q.part_of(id) {
                    q.focus(Some(part));
                }
            }
        }
    }

    pub fn focused_id(&self) -> Option<SemId> {
        match self {
            Sample::Marker(m) => m.sem_id().filter(|_| m.is_focused()).cloned(),
            Sample::Thread(t) => t.focused().and_then(|p| t.part_id(p)),
            Sample::New(n) => n.focused().and_then(|p| n.part_id(p)),
            Sample::Quick(q) => q.focused().and_then(|p| q.part_id(p)),
        }
    }

    pub fn project(&self, b: &mut SnapshotBuilder, card: &SemId) {
        match self {
            Sample::Marker(m) => {
                if let Some(id) = m.sem_id() {
                    m.project(b, id, Some(card));
                }
            }
            Sample::Thread(t) => {
                if let Some(id) = t.sem_id() {
                    t.project(b, id, "Comment thread", Some(card));
                }
            }
            Sample::New(n) => {
                if let Some(id) = n.sem_id() {
                    n.project(b, id, "New comment", Some(card));
                }
            }
            Sample::Quick(q) => {
                if let Some(id) = q.sem_id() {
                    q.project(b, id, Some(card));
                }
            }
        }
    }

    /// Runs `action` on `target` when it is one of its parts: `None`
    /// when it is not.
    pub fn act(
        &mut self,
        target: &SemId,
        action: &str,
        param: Option<&Value>,
    ) -> Option<Result<(), String>> {
        match self {
            Sample::Marker(m) => (m.sem_id() == Some(target)).then(|| {
                if action == "activate" && m.is_pin() {
                    m.open();
                    Ok(())
                } else {
                    Err(format!("a marker does not {action}"))
                }
            }),
            Sample::Thread(t) => t.part_of(target).map(|p| t.act(p, action, param)),
            Sample::New(n) => n.part_of(target).map(|p| n.act(p, action, param)),
            Sample::Quick(q) => q.part_of(target).map(|p| q.act(p, action, param)),
        }
    }
}
