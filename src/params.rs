//! Parameter metadata for pattern detectors
//!
//! This module provides metadata about detector parameters, enabling:
//! - Grid search optimization
//! - Parameter documentation
//! - Automatic configuration UI generation
//!
//! # Example
//!
//! ```rust
//! use yacpd::params::{ParamMeta, ParamType, ParameterizedDetector};
//! use yacpd::prelude::*;
//!
//! // Get parameter metadata for a detector
//! let params = EngulfingDetector::param_meta();
//! for param in params {
//!     println!("{}: {:?} (default: {})", param.name, param.param_type, param.default);
//! }
//! ```

use std::collections::HashMap;

use crate::{PatternError, Period, Ratio, Result};

// ============================================================
// PARAMETER TYPES
// ============================================================

/// Type of parameter value
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamType {
  /// Ratio value (0.0..=1.0 typically, but can exceed 1.0 for some params like shadow_ratio)
  Ratio,
  /// Period value (positive integer)
  Period,
}

/// Metadata for a single detector parameter
#[derive(Debug, Clone)]
pub struct ParamMeta {
  /// Parameter name (e.g., "min_engulf_ratio")
  pub name: &'static str,
  /// Parameter type (Ratio or Period)
  pub param_type: ParamType,
  /// Default value
  pub default: f64,
  /// Range for optimization: (min, max, step)
  pub range: (f64, f64, f64),
  /// Human-readable description
  pub description: &'static str,
}

impl ParamMeta {
  /// Create a new ParamMeta for a Ratio parameter
  pub const fn ratio(
    name: &'static str,
    default: f64,
    range: (f64, f64, f64),
    description: &'static str,
  ) -> Self {
    Self { name, param_type: ParamType::Ratio, default, range, description }
  }

  /// Create a new ParamMeta for a Period parameter
  pub const fn period(
    name: &'static str,
    default: f64,
    range: (f64, f64, f64),
    description: &'static str,
  ) -> Self {
    Self { name, param_type: ParamType::Period, default, range, description }
  }

  /// Generate all values for grid search
  pub fn generate_grid(&self) -> Vec<f64> {
    let (min, max, step) = self.range;
    let mut values = Vec::new();
    let mut v = min;
    while v <= max + f64::EPSILON {
      values.push(v);
      v += step;
    }
    values
  }

  /// Validate a value for this parameter
  pub fn validate(&self, value: f64) -> Result<()> {
    let (min, max, _) = self.range;
    if value < min || value > max {
      return Err(PatternError::OutOfRange { field: self.name, value, min, max });
    }
    match self.param_type {
      ParamType::Ratio => {
        // For Ratio, value might exceed 1.0 for some parameters (e.g., shadow_ratio: 2.0)
        // We don't strictly enforce 0-1 here, let the Ratio::new handle it
        Ok(())
      },
      ParamType::Period => {
        if value < 1.0 || value.fract() != 0.0 {
          return Err(PatternError::InvalidValue("Period must be a positive integer"));
        }
        Ok(())
      },
    }
  }
}

// ============================================================
// PARAMETERIZED DETECTOR TRAIT
// ============================================================

/// Trait for detectors that support parameterization
///
/// Implementing this trait enables:
/// - Discovery of available parameters
/// - Creation of detectors with custom parameter values
/// - Grid search optimization
pub trait ParameterizedDetector: Sized {
  /// Returns metadata for all configurable parameters
  fn param_meta() -> &'static [ParamMeta];

  /// Creates a detector with parameters from a HashMap
  ///
  /// Missing parameters use their default values.
  fn with_params(params: &HashMap<&str, f64>) -> Result<Self>;

  /// Returns the pattern ID string
  fn pattern_id_str() -> &'static str;
}

// ============================================================
// PARAMETER VALUE HELPERS
// ============================================================

/// Helper to get a Ratio from params with default fallback
pub fn get_ratio(params: &HashMap<&str, f64>, key: &str, default: f64) -> Result<Ratio> {
  let value = params.get(key).copied().unwrap_or(default);
  Ratio::new(value)
}

/// Helper to get a Period from params with default fallback
pub fn get_period(params: &HashMap<&str, f64>, key: &str, default: usize) -> Result<Period> {
  let value = params.get(key).copied().unwrap_or(default as f64);
  Period::new(value as usize)
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_param_meta_ratio() {
    let meta = ParamMeta::ratio("test_ratio", 0.5, (0.3, 0.7, 0.1), "Test ratio parameter");

    assert_eq!(meta.name, "test_ratio");
    assert_eq!(meta.param_type, ParamType::Ratio);
    assert_eq!(meta.default, 0.5);
  }

  #[test]
  fn test_param_meta_period() {
    let meta = ParamMeta::period("test_period", 14.0, (10.0, 20.0, 2.0), "Test period parameter");

    assert_eq!(meta.name, "test_period");
    assert_eq!(meta.param_type, ParamType::Period);
    assert_eq!(meta.default, 14.0);
  }

  #[test]
  fn test_generate_grid() {
    let meta = ParamMeta::ratio("test", 0.5, (0.3, 0.7, 0.2), "Test");

    let grid = meta.generate_grid();
    assert_eq!(grid.len(), 3);
    assert!((grid[0] - 0.3).abs() < f64::EPSILON);
    assert!((grid[1] - 0.5).abs() < f64::EPSILON);
    assert!((grid[2] - 0.7).abs() < f64::EPSILON);
  }

  #[test]
  fn test_validate_ratio() {
    let meta = ParamMeta::ratio("test", 0.5, (0.3, 0.7, 0.1), "Test");

    assert!(meta.validate(0.5).is_ok());
    assert!(meta.validate(0.3).is_ok());
    assert!(meta.validate(0.7).is_ok());
    assert!(meta.validate(0.2).is_err());
    assert!(meta.validate(0.8).is_err());
  }

  #[test]
  fn test_validate_period() {
    let meta = ParamMeta::period("test", 14.0, (10.0, 20.0, 2.0), "Test");

    assert!(meta.validate(14.0).is_ok());
    assert!(meta.validate(10.0).is_ok());
    assert!(meta.validate(20.0).is_ok());
    assert!(meta.validate(8.0).is_err());
    assert!(meta.validate(22.0).is_err());
  }

  #[test]
  fn test_get_ratio_helper() {
    let mut params = HashMap::new();
    params.insert("key1", 0.8);

    assert!((get_ratio(&params, "key1", 0.5).unwrap().get() - 0.8).abs() < f64::EPSILON);
    assert!((get_ratio(&params, "key2", 0.5).unwrap().get() - 0.5).abs() < f64::EPSILON);
  }

  #[test]
  fn test_get_period_helper() {
    let mut params = HashMap::new();
    params.insert("key1", 20.0);

    assert_eq!(get_period(&params, "key1", 14).unwrap().get(), 20);
    assert_eq!(get_period(&params, "key2", 14).unwrap().get(), 14);
  }
}
