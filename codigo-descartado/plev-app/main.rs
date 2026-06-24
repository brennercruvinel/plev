//! plev-app: launcher da galeria `showcase`.
//!
//! `plev::run_event_loop` sobe apenas o `window::App` embutido do engine, que
//! hoje e' um shell sem conteudo (o showcase virou crate proprio, e o render
//! do engine resolve um compositor vazio). Por isso a janela abria vazia.
//! Para haver conteudo visivel, este app delega ao `showcase::run()`.
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
fn main() {
    showcase::run();
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
fn main() {}
