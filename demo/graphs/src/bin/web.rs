//! Web binary entry: the one trunk builds (`demo/graphs/index.html`,
//! `data-bin`). It has its own name because trunk picks the wasm artifact
//! by target name and the desktop bin is called `graphs` too. On every other
//! target this binary is empty.

#[cfg(target_arch = "wasm32")]
fn main() {
    graphs::run_web();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
