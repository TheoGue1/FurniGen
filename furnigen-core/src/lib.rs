//! FurniGen core: wardrobe spec, validation, and geometry (canonical units: **mm**).
//!
//! WASM and the web UI depend on this crate; keep it free of browser or Three.js APIs.

mod spec;

pub use spec::{
    parse_wardrobe_spec_json, validate_wardrobe_spec, LayoutSpec, SpecError, WardrobeSpec,
    WARDROBE_SPEC_VERSION,
};

use thiserror::Error;

/// Recoverable validation errors for numeric fields and future spec rules.
#[derive(Debug, Error, PartialEq)]
pub enum ValidationError {
    #[error("depth_mm must be a finite positive value, got {0}")]
    InvalidDepth(f64),
}

/// Validates a straight-run carcass depth in millimeters (positive, finite).
pub fn validate_depth_mm(depth_mm: f64) -> Result<(), ValidationError> {
    if !depth_mm.is_finite() || depth_mm <= 0.0 {
        return Err(ValidationError::InvalidDepth(depth_mm));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_positive_depth() {
        assert_eq!(
            validate_depth_mm(0.0),
            Err(ValidationError::InvalidDepth(0.0))
        );
        assert_eq!(
            validate_depth_mm(-10.0),
            Err(ValidationError::InvalidDepth(-10.0))
        );
    }

    #[test]
    fn rejects_nan_and_infinite_depth() {
        assert!(matches!(
            validate_depth_mm(f64::NAN),
            Err(ValidationError::InvalidDepth(_))
        ));
        assert!(matches!(
            validate_depth_mm(f64::INFINITY),
            Err(ValidationError::InvalidDepth(_))
        ));
    }

    #[test]
    fn accepts_typical_depth() {
        assert!(validate_depth_mm(600.0).is_ok());
    }
}
