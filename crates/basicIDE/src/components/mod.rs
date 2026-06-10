// Componentes ainda não conectados às views mantêm um allow(dead_code)
// local até o port completar — assim o resto do crate fica sujeito ao
// lint normalmente.
#[allow(dead_code)]
pub mod avatar;
#[allow(dead_code)]
pub mod badge;
pub mod button;
#[allow(dead_code)]
pub mod checkbox;
pub mod context_menu;
pub mod hoff;
pub mod modal;
#[allow(dead_code)]
pub mod panel_header;
#[allow(dead_code)]
pub mod separator;
#[allow(dead_code)]
pub mod tabs;
