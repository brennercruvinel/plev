//! The window's library half: the session recording (`record`). The
//! binary's halves of the window (keys and keymap, pacing, the renderer,
//! the shell's side of the recording) sit beside it and are declared by
//! `main.rs`, which owns `App`.

pub mod record;
