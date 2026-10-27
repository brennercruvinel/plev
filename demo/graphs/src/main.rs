//! Desktop window of the graphs demo: `cargo run -p graphs [lesmis|compound]`.
//! The web build is the lib (`graphs::start`).

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use engine::runtime::window::{App, AppEvent};

    env_logger::init();
    let name = std::env::args().nth(1).unwrap_or_default();
    let view = match graphs::Page::from_name(&name).and_then(graphs::GraphsView::new) {
        Ok(view) => view,
        Err(e) => {
            eprintln!("graphs: {e}");
            std::process::exit(1);
        }
    };
    let event_loop = winit::event_loop::EventLoop::<AppEvent>::with_user_event()
        .build()
        .expect("event loop");
    let mut app = App::new().with_view(Box::new(view));
    if let Err(e) = event_loop.run_app(&mut app) {
        eprintln!("graphs: {e}");
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
