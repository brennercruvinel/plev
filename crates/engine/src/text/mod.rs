mod atlas;
pub mod backend;
mod cache;
mod fonts;
pub mod measure;
#[cfg(not(target_arch = "wasm32"))]
pub mod probe;
mod system;
mod vertex;

pub use backend::{CosmicTextBackend, StyleRun, TextBackend, TextStyle};
pub use measure::{LineMetrics, ShapedText, TextMeasurer};
pub use system::TextSystem;
pub use vertex::TextVertex;

#[cfg(test)]
mod tests_measure;
#[cfg(test)]
mod tests_raster;
