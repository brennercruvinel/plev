// Spellbook desktop shell.
//
// Starts the embedded HTTP server on a free loopback port, then points the Tauri webview at it.
// Serving the existing frontend over http:// (rather than Tauri's asset protocol) is deliberate:
// webapp/static/js/api.js calls relative /api/* paths with fetch(), so it runs here unmodified.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod db;
mod paths;
mod wizard;
mod writes;

use axum::{response::Html, routing::get, Router};
use std::net::{Ipv4Addr, SocketAddr};
use tower_http::services::ServeDir;

async fn serve(listener: tokio::net::TcpListener) {
    let static_dir = paths::static_dir();
    let index = static_dir.join("index.html");

    // index.html references its CSS/JS as /assets/..., which the Python server maps onto the
    // static dir via app.mount("/assets", ...). Serving the directory at the router root instead
    // makes every /assets/* request 404 — the window then renders raw unstyled HTML with no JS.
    let app = Router::new()
        .merge(api::router())
        .merge(writes::router())
        .route(
            "/",
            get(move || {
                let index = index.clone();
                async move { Html(std::fs::read_to_string(index).unwrap_or_default()) }
            }),
        )
        .nest_service("/assets", ServeDir::new(&static_dir));
    let _ = axum::serve(listener, app).await;
}

fn main() {
    if let Err(e) = db::init_app_db() {
        eprintln!("failed to initialise app.db: {e}");
    }

    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");

    // Bind to port 0 and read back the assigned port: fixed ports collide with the Python dev
    // server (and with a second copy of this app), and the failure mode there is a blank window.
    let listener = rt
        .block_on(async { tokio::net::TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0))).await })
        .expect("bind loopback");
    let port = listener.local_addr().expect("local addr").port();

    std::thread::spawn(move || {
        rt.block_on(serve(listener));
    });

    let url = format!("http://127.0.0.1:{port}");
    println!("Spellbook serving on {url}");
    println!("  static: {}", paths::static_dir().display());
    println!("  cards : {}", paths::cards_db().display());
    println!("  app db: {}", paths::app_db().display());

    tauri::Builder::default()
        .setup(move |app| {
            use tauri::WebviewWindowBuilder;
            let parsed = url.parse().expect("valid url");
            WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::External(parsed))
                .title("Spellbook")
                .inner_size(1400.0, 900.0)
                .min_inner_size(900.0, 600.0)
                .resizable(true)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
