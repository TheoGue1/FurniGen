//! Wardrobe JSON contract (canonical **mm**). v1: straight runs only; `layout` is tagged for future variants.

use serde::{Deserialize, Serialize};
use serde_json::Map;
use thiserror::Error;

/// Supported top-level contract revision (bump when breaking JSON shape).
pub const WARDROBE_SPEC_VERSION: u32 = 1;

/// Root document exchanged with WASM and the web UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WardrobeSpec {
    pub version: u32,
    pub layout: LayoutSpec,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub extensions: Map<String, serde_json::Value>,
}

/// Geometry layout; new `type` variants may appear in later milestones (L/U, corners).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LayoutSpec {
    StraightRun {
        width_mm: f64,
        height_mm: f64,
        depth_mm: f64,
    },
}

#[derive(Debug, Error, PartialEq)]
pub enum SpecError {
    #[error("JSON parse error: {0}")]
    Json(String),
    #[error("unsupported spec version {0} (supported: {WARDROBE_SPEC_VERSION})")]
    UnsupportedVersion(u32),
    #[error("{0}")]
    Validation(String),
}

impl From<serde_json::Error> for SpecError {
    fn from(e: serde_json::Error) -> Self {
        SpecError::Json(e.to_string())
    }
}

fn validate_positive_finite(label: &str, v: f64) -> Result<(), SpecError> {
    if !v.is_finite() {
        return Err(SpecError::Validation(format!(
            "{label} must be finite, got {v}"
        )));
    }
    if v <= 0.0 {
        return Err(SpecError::Validation(format!(
            "{label} must be positive, got {v}"
        )));
    }
    Ok(())
}

/// Validates shape rules after serde deserialization.
pub fn validate_wardrobe_spec(spec: &WardrobeSpec) -> Result<(), SpecError> {
    if spec.version != WARDROBE_SPEC_VERSION {
        return Err(SpecError::UnsupportedVersion(spec.version));
    }
    match &spec.layout {
        LayoutSpec::StraightRun {
            width_mm,
            height_mm,
            depth_mm,
        } => {
            validate_positive_finite("width_mm", *width_mm)?;
            validate_positive_finite("height_mm", *height_mm)?;
            validate_positive_finite("depth_mm", *depth_mm)?;
        }
    }
    Ok(())
}

/// Parses JSON then validates. All numeric fields are **millimeters**.
pub fn parse_wardrobe_spec_json(json: &str) -> Result<WardrobeSpec, SpecError> {
    let spec: WardrobeSpec = serde_json::from_str(json)?;
    validate_wardrobe_spec(&spec)?;
    Ok(spec)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = include_str!("../../spec-fixtures/wardrobe-spec-v1-minimal.json");
    const INVALID_DEPTH: &str =
        include_str!("../../spec-fixtures/wardrobe-spec-v1-invalid-depth.json");
    const WITH_EXT: &str =
        include_str!("../../spec-fixtures/wardrobe-spec-v1-with-extensions.json");

    #[test]
    fn golden_minimal_parse_and_validate() {
        let spec = parse_wardrobe_spec_json(MINIMAL.trim()).unwrap();
        assert_eq!(spec.version, 1);
        assert!(spec.extensions.is_empty());
        assert_eq!(
            spec.layout,
            LayoutSpec::StraightRun {
                width_mm: 2400.0,
                height_mm: 2200.0,
                depth_mm: 600.0,
            }
        );
    }

    #[test]
    fn golden_invalid_depth_fails_validation() {
        let err = parse_wardrobe_spec_json(INVALID_DEPTH.trim()).unwrap_err();
        assert!(matches!(err, SpecError::Validation(_)));
    }

    #[test]
    fn golden_with_extensions_round_trips() {
        let spec = parse_wardrobe_spec_json(WITH_EXT.trim()).unwrap();
        assert_eq!(
            spec.extensions.get("reserved"),
            Some(&serde_json::json!(true))
        );
        let again = parse_wardrobe_spec_json(&serde_json::to_string(&spec).unwrap()).unwrap();
        assert_eq!(spec, again);
    }

    #[test]
    fn serde_round_trip_preserves_layout() {
        let spec = parse_wardrobe_spec_json(MINIMAL.trim()).unwrap();
        let json = serde_json::to_string(&spec).unwrap();
        let back: WardrobeSpec = serde_json::from_str(&json).unwrap();
        validate_wardrobe_spec(&back).unwrap();
        assert_eq!(spec, back);
    }
}
