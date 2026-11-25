//! Web binary entry: the one trunk builds (`apps/uigallery/web/index.html`, `data-bin`).
//! It has its own name because trunk picks the wasm artifact by target
//! name. When the lib was also a cdylib (the android `.so`), it and the
//! `uigallery` bin shared the name "uigallery": built for wasm32, both left a
//! `.wasm` and trunk refused to choose. The lib is a plain rlib since
//! task-106; the entry keeps its name, and trunk was not tried on the
//! `uigallery` bin since. On every other target this binary is empty.

#[cfg(target_arch = "wasm32")]
fn main() {
    uigallery::run_web();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
