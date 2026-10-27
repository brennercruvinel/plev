//! Desktop window of the corpus demo: `cargo run -p corpus [lesmis|compound]`.
//! The web build is the lib (`corpus::start`).

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use engine::runtime::window::{App, AppEvent};

    env_logger::init();
    let page = corpus::Page::from_name(&std::env::args().nth(1).unwrap_or_default());
    let view = match corpus::CorpusView::new(page) {
        Ok(view) => view,
        Err(e) => {
            eprintln!("corpus: {e}");
            std::process::exit(1);
        }
    };
    let event_loop = winit::event_loop::EventLoop::<AppEvent>::with_user_event()
        .build()
        .expect("event loop");
    let mut app = App::new().with_view(Box::new(view));
    if let Err(e) = event_loop.run_app(&mut app) {
        eprintln!("corpus: {e}");
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
