//! The model implementations: candle's llama and the burn port.

#[cfg(feature = "burn")]
pub mod burn;
#[cfg(feature = "candle")]
pub mod candle;
