//! Web binary entry: the one trunk builds (`apps/urnadbapp/web/index.html`,
//! `data-bin`). It has its own name because trunk picks the wasm artifact by
//! target name, and the lib's cdylib (the android `.so`) and the `urnadbapp`
//! bin share the name "urnadbapp". On every other target this binary is empty.

#[cfg(target_arch = "wasm32")]
fn main() {
    urnadbapp::run_web();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
