//! Web binary entry: the one trunk builds (`apps/uigallery/web/index.html`, `data-bin`).
//! It has its own name because trunk picks the wasm artifact by target
//! name, and the lib's cdylib (the android `.so`) and the `uigallery` bin
//! share the name "uigallery": built for wasm32, both leave a `.wasm` and
//! trunk refuses to choose. On every other target this binary is empty.

#[cfg(target_arch = "wasm32")]
fn main() {
    uigallery::run_web();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
