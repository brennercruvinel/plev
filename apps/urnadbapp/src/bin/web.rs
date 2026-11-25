//! Web binary entry: the one trunk builds (`apps/urnadbapp/web/index.html`,
//! `data-bin`). It has its own name because trunk picks the wasm artifact by
//! target name. When the lib was also a cdylib (the android `.so`), it and the
//! `urnadbapp` bin shared the name "urnadbapp". The lib is a plain rlib since
//! task-106; the entry keeps its name. On every other target this binary is
//! empty.

#[cfg(target_arch = "wasm32")]
fn main() {
    urnadbapp::run_web();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
