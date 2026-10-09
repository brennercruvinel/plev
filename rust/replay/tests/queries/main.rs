//! the inspector's model against sessions recorded the way an app records
//! them: a toy git client driven through a real dispatcher whose stages
//! reach the recorder, read back by the inspector; its text, its binary,
//! and the proof that reading runs nothing.

mod apps;
mod cli;
mod control;
mod export;
mod readonly;
mod text;
mod thesis;
mod timeline;
mod why;
