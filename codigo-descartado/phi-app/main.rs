#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
fn main() {
    env_logger::init();
    let event_loop = winit::event_loop::EventLoop::<phi::window::AppEvent>::with_user_event()
        .build()
        .unwrap();
    phi::run_event_loop(event_loop);
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
fn main() {
    // Entry points: wasm_main() for WASM, android_main() for Android — both in lib.rs
}
