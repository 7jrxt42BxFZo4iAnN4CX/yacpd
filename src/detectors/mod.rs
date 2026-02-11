//! Candlestick pattern detectors
//!
//! This module contains TA-Lib compatible and extended candlestick pattern detectors.
//!
//! # Pattern Categories
//!
//! - **Single-bar (17)**: Doji variants, Hammer family, Marubozu, etc.
//! - **Two-bar (17)**: Engulfing, Harami, Piercing, etc.
//! - **Three-bar (22)**: Morning/Evening Star, Three Soldiers/Crows, etc.
//! - **Multi-bar (8)**: Breakaway, Hikkake, Rising/Falling Three Methods, etc.
//! - **Extended (30+)**: Price Lines, Windows, Meeting Lines, Basic candles, etc.

pub mod helpers;

/// Generate `with_defaults()` -> `Self::default()` for multiple detector types.
macro_rules! impl_with_defaults {
  ($($detector:ty),* $(,)?) => {
    $(impl $detector {
      pub fn with_defaults() -> Self { Self::default() }
    })*
  };
}

pub mod extended;
pub mod multi_bar;
pub mod single_bar;
pub mod three_bar;
pub mod two_bar;

// Re-export all detectors for convenience
pub use extended::*;
pub use helpers::*;
pub use multi_bar::*;
pub use single_bar::*;
pub use three_bar::*;
pub use two_bar::*;
