// `hot_reload` cfg: the `hot-reload` feature is on by default, but the
// watchers and the read-from-disk shader path only make sense for a desktop
// debug build run from the source tree. Everything else (release, wasm,
// android, ios) gets the embedded shaders with zero watcher code.
fn main() {
    println!("cargo:rustc-check-cfg=cfg(hot_reload)");
    println!("cargo:rerun-if-changed=build.rs");
    let feature = std::env::var_os("CARGO_FEATURE_HOT_RELOAD").is_some();
    let debug = std::env::var("PROFILE").as_deref() == Ok("debug");
    let desktop = matches!(
        std::env::var("CARGO_CFG_TARGET_OS").as_deref(),
        Ok("macos" | "linux" | "windows")
    );
    if feature && debug && desktop {
        println!("cargo:rustc-cfg=hot_reload");
    }
}
