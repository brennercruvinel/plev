//! Files dragged over the window and dropped on it, as one event of the
//! engine. The platform sends three (`HoveredFile`, `DroppedFile` and
//! `HoveredFileCancelled` in winit 0.30), one per file for the first two;
//! [`FileDrop::from_window_event`] is the one translation every shell
//! uses (the engine's own `App`, the urnaui, the showcase), so no shell
//! reads a winit drop event itself. The event then reaches a view the way
//! a pointer event does: through `InputState::handle_file_drop`, hit
//! tested at its position when it has one, or through the view's own
//! `handle_file_drop` in an app that routes events itself.
//!
//! The position is in logical pixels, as the pointer's, and optional:
//! winit 0.30 gives none with a drop (AppKit knows the `draggingLocation`,
//! winit drops it, and no `CursorMoved` arrives while a file is dragged),
//! so it is `None` from the window today and `Some` in tests and in the
//! winit that exposes it.

use std::path::PathBuf;

use winit::event::WindowEvent;

/// One step of a file drag over the window.
#[derive(Clone, Debug, PartialEq)]
pub enum FileDrop {
    /// Files entered the window and are still held over it.
    Hovered {
        paths: Vec<PathBuf>,
        position: Option<(f32, f32)>,
    },
    /// Files were released on the window.
    Dropped {
        paths: Vec<PathBuf>,
        position: Option<(f32, f32)>,
    },
    /// The held files left the window without a drop.
    Cancelled,
}

impl FileDrop {
    /// The engine's event for a platform event; `None` for every other
    /// event. One path per platform event: a drop of several files is
    /// several events, joined by [`join`](Self::join) where a shell
    /// buffers them (the engine's `App`), one per file where it does not.
    pub fn from_window_event(event: &WindowEvent) -> Option<FileDrop> {
        match event {
            WindowEvent::HoveredFile(path) => Some(FileDrop::Hovered {
                paths: vec![path.clone()],
                position: None,
            }),
            WindowEvent::DroppedFile(path) => Some(FileDrop::Dropped {
                paths: vec![path.clone()],
                position: None,
            }),
            WindowEvent::HoveredFileCancelled => Some(FileDrop::Cancelled),
            _ => None,
        }
    }

    /// The files of the step; none for a cancel.
    pub fn paths(&self) -> &[PathBuf] {
        match self {
            FileDrop::Hovered { paths, .. } | FileDrop::Dropped { paths, .. } => paths,
            FileDrop::Cancelled => &[],
        }
    }

    /// Where over the window, in logical pixels, when the platform said.
    pub fn position(&self) -> Option<(f32, f32)> {
        match self {
            FileDrop::Hovered { position, .. } | FileDrop::Dropped { position, .. } => *position,
            FileDrop::Cancelled => None,
        }
    }

    /// True for the step that releases files on the window.
    pub fn is_drop(&self) -> bool {
        matches!(self, FileDrop::Dropped { .. })
    }

    /// Folds `next` into this step when both are the same phase of one
    /// drag (the platform sends one event per file): the paths append,
    /// the position stays the one known first, or becomes the one `next`
    /// knows. Gives `next` back when the phases differ, or either is a
    /// cancel.
    pub fn join(&mut self, next: FileDrop) -> Result<(), FileDrop> {
        match (self, next) {
            (
                FileDrop::Hovered { paths, position },
                FileDrop::Hovered {
                    paths: more,
                    position: at,
                },
            )
            | (
                FileDrop::Dropped { paths, position },
                FileDrop::Dropped {
                    paths: more,
                    position: at,
                },
            ) => {
                paths.extend(more);
                if position.is_none() {
                    *position = at;
                }
                Ok(())
            }
            (_, next) => Err(next),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    #[test]
    fn the_three_platform_events_translate_and_nothing_else_does() {
        assert_eq!(
            FileDrop::from_window_event(&WindowEvent::HoveredFile(p("/a.urna"))),
            Some(FileDrop::Hovered {
                paths: vec![p("/a.urna")],
                position: None,
            })
        );
        assert_eq!(
            FileDrop::from_window_event(&WindowEvent::DroppedFile(p("/a.urna"))),
            Some(FileDrop::Dropped {
                paths: vec![p("/a.urna")],
                position: None,
            })
        );
        assert_eq!(
            FileDrop::from_window_event(&WindowEvent::HoveredFileCancelled),
            Some(FileDrop::Cancelled)
        );
        assert_eq!(
            FileDrop::from_window_event(&WindowEvent::CloseRequested),
            None
        );
        assert_eq!(
            FileDrop::from_window_event(&WindowEvent::Focused(true)),
            None
        );
    }

    #[test]
    fn paths_and_position_read_every_phase() {
        let dropped = FileDrop::Dropped {
            paths: vec![p("/a"), p("/b")],
            position: Some((10.0, 20.0)),
        };
        assert_eq!(dropped.paths(), [p("/a"), p("/b")]);
        assert_eq!(dropped.position(), Some((10.0, 20.0)));
        assert!(dropped.is_drop());
        let hovered = FileDrop::Hovered {
            paths: vec![p("/a")],
            position: None,
        };
        assert!(!hovered.is_drop());
        assert_eq!(hovered.position(), None);
        assert!(FileDrop::Cancelled.paths().is_empty());
        assert_eq!(FileDrop::Cancelled.position(), None);
        assert!(!FileDrop::Cancelled.is_drop());
    }

    #[test]
    fn one_event_per_file_joins_into_one_drop() {
        let mut drop = FileDrop::from_window_event(&WindowEvent::DroppedFile(p("/a"))).unwrap();
        let next = FileDrop::from_window_event(&WindowEvent::DroppedFile(p("/b"))).unwrap();
        assert_eq!(drop.join(next), Ok(()));
        assert_eq!(drop.paths(), [p("/a"), p("/b")]);
        assert_eq!(drop.position(), None);

        // the first position known stays; a later one fills a None
        let mut hover = FileDrop::Hovered {
            paths: vec![p("/a")],
            position: None,
        };
        assert_eq!(
            hover.join(FileDrop::Hovered {
                paths: vec![p("/b")],
                position: Some((3.0, 4.0)),
            }),
            Ok(())
        );
        assert_eq!(hover.position(), Some((3.0, 4.0)));
        assert_eq!(
            hover.join(FileDrop::Hovered {
                paths: vec![p("/c")],
                position: Some((9.0, 9.0)),
            }),
            Ok(())
        );
        assert_eq!(hover.position(), Some((3.0, 4.0)));
        assert_eq!(hover.paths().len(), 3);
    }

    #[test]
    fn different_phases_and_cancels_do_not_join() {
        let mut hover = FileDrop::Hovered {
            paths: vec![p("/a")],
            position: None,
        };
        let dropped = FileDrop::Dropped {
            paths: vec![p("/a")],
            position: None,
        };
        assert_eq!(hover.join(dropped.clone()), Err(dropped.clone()));
        assert_eq!(hover.paths(), [p("/a")], "untouched");
        assert_eq!(hover.join(FileDrop::Cancelled), Err(FileDrop::Cancelled));
        let mut cancelled = FileDrop::Cancelled;
        assert_eq!(cancelled.join(dropped.clone()), Err(dropped));
        assert_eq!(
            cancelled.join(FileDrop::Cancelled),
            Err(FileDrop::Cancelled)
        );
    }
}
