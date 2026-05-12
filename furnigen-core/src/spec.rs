//! Wardrobe JSON contract (canonical **mm**). v1: straight runs only; `layout` is tagged for future variants.

use serde::{Deserialize, Serialize};
use serde_json::Map;
use thiserror::Error;

/// Supported top-level contract revision (bump when breaking JSON shape).
pub const WARDROBE_SPEC_VERSION: u32 = 1;

/// Interior fittings (shelves, uprights, …).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InteriorSpec {
    /// Placeholder; no extra geometry.
    Stub,
    /// `shelf_count` horizontal boards with equal air gaps to inner floor, between boards, and to inner top.
    EqualSpacingShelves {
        shelf_count: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        shelf_thickness_mm: Option<f64>,
    },
    /// Fixed shelf board bottoms (mm from inner floor), strictly ascending; optional `min_gap_mm` between boards.
    ExplicitShelfHeights {
        shelf_bottom_y_mm: Vec<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        shelf_thickness_mm: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min_gap_mm: Option<f64>,
    },
    /// Equal vertical gaps for `shelf_count` boards, but only inside the middle band between reserved bottom/top zones (mm).
    ZonesEqualFillShelves {
        bottom_zone_mm: f64,
        top_reserve_mm: f64,
        shelf_count: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        shelf_thickness_mm: Option<f64>,
    },
    /// `rungs` horizontal shelf boards; vertical air gaps (floor–shelves–ceiling) scale like φ^0…φ^n.
    GoldenRatioLadderShelves {
        rungs: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        shelf_thickness_mm: Option<f64>,
    },
}

/// Root document exchanged with WASM and the web UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WardrobeSpec {
    pub version: u32,
    pub layout: LayoutSpec,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interior: Option<InteriorSpec>,
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
            validate_interior_for_height(spec.interior.as_ref(), *height_mm)?;
        }
    }
    Ok(())
}

fn validate_interior_for_height(
    interior: Option<&InteriorSpec>,
    inner_height_mm: f64,
) -> Result<(), SpecError> {
    let Some(interior) = interior else {
        return Ok(());
    };
    match interior {
        InteriorSpec::Stub => Ok(()),
        InteriorSpec::EqualSpacingShelves {
            shelf_count,
            shelf_thickness_mm,
        } => {
            if *shelf_count < 1 {
                return Err(SpecError::Validation(
                    "shelf_count must be at least 1".to_owned(),
                ));
            }
            let t =
                shelf_thickness_mm.unwrap_or(crate::interior_shelves::DEFAULT_SHELF_THICKNESS_MM);
            validate_positive_finite("shelf_thickness_mm", t)?;
            crate::interior_shelves::equal_spacing_shelf_bottoms_mm(
                inner_height_mm,
                *shelf_count,
                t,
            )
            .map_err(SpecError::Validation)?;
            Ok(())
        }
        InteriorSpec::ExplicitShelfHeights {
            shelf_bottom_y_mm,
            shelf_thickness_mm,
            min_gap_mm,
        } => {
            let t =
                shelf_thickness_mm.unwrap_or(crate::interior_shelves::DEFAULT_SHELF_THICKNESS_MM);
            validate_positive_finite("shelf_thickness_mm", t)?;
            crate::interior_shelves::validate_explicit_shelf_bottoms_mm(
                inner_height_mm,
                shelf_bottom_y_mm,
                t,
                *min_gap_mm,
            )
            .map_err(SpecError::Validation)?;
            Ok(())
        }
        InteriorSpec::ZonesEqualFillShelves {
            bottom_zone_mm,
            top_reserve_mm,
            shelf_count,
            shelf_thickness_mm,
        } => {
            if *shelf_count < 1 {
                return Err(SpecError::Validation(
                    "shelf_count must be at least 1".to_owned(),
                ));
            }
            let t =
                shelf_thickness_mm.unwrap_or(crate::interior_shelves::DEFAULT_SHELF_THICKNESS_MM);
            validate_positive_finite("shelf_thickness_mm", t)?;
            if !bottom_zone_mm.is_finite() || *bottom_zone_mm < 0.0 {
                return Err(SpecError::Validation(
                    "bottom_zone_mm must be finite and non-negative".to_owned(),
                ));
            }
            if !top_reserve_mm.is_finite() || *top_reserve_mm < 0.0 {
                return Err(SpecError::Validation(
                    "top_reserve_mm must be finite and non-negative".to_owned(),
                ));
            }
            if *bottom_zone_mm + *top_reserve_mm >= inner_height_mm {
                return Err(SpecError::Validation(format!(
                    "bottom_zone_mm ({bottom_zone_mm}) + top_reserve_mm ({top_reserve_mm}) must be strictly less than inner height ({inner_height_mm} mm)"
                )));
            }
            crate::interior_shelves::zones_equal_fill_shelf_bottoms_mm(
                inner_height_mm,
                *bottom_zone_mm,
                *top_reserve_mm,
                *shelf_count,
                t,
            )
            .map_err(SpecError::Validation)?;
            Ok(())
        }
        InteriorSpec::GoldenRatioLadderShelves {
            rungs,
            shelf_thickness_mm,
        } => {
            if *rungs < 1 {
                return Err(SpecError::Validation("rungs must be at least 1".to_owned()));
            }
            let t =
                shelf_thickness_mm.unwrap_or(crate::interior_shelves::DEFAULT_SHELF_THICKNESS_MM);
            validate_positive_finite("shelf_thickness_mm", t)?;
            crate::interior_shelves::golden_ratio_ladder_shelf_bottoms_mm(
                inner_height_mm,
                *rungs,
                t,
            )
            .map_err(SpecError::Validation)?;
            Ok(())
        }
    }
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
    const EQUAL_SHELVES: &str =
        include_str!("../../spec-fixtures/wardrobe-spec-v1-equal-spacing-shelves.json");
    const EXPLICIT_SHELVES: &str =
        include_str!("../../spec-fixtures/wardrobe-spec-v1-explicit-shelf-heights.json");
    const ZONES_EQUAL_FILL: &str =
        include_str!("../../spec-fixtures/wardrobe-spec-v1-zones-equal-fill-shelves.json");
    const GOLDEN_RATIO_LADDER: &str =
        include_str!("../../spec-fixtures/wardrobe-spec-v1-golden-ratio-ladder-shelves.json");

    #[test]
    fn golden_minimal_parse_and_validate() {
        let spec = parse_wardrobe_spec_json(MINIMAL.trim()).unwrap();
        assert_eq!(spec.version, 1);
        assert!(spec.extensions.is_empty());
        assert_eq!(spec.interior, None);
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
    fn interior_stub_round_trips_json() {
        let json = r#"{"version":1,"layout":{"type":"straight_run","width_mm":1.0,"height_mm":2.0,"depth_mm":3.0},"interior":{"type":"stub"}}"#;
        let spec: WardrobeSpec = serde_json::from_str(json).unwrap();
        assert_eq!(spec.interior, Some(InteriorSpec::Stub));
        validate_wardrobe_spec(&spec).unwrap();
        let again: WardrobeSpec =
            serde_json::from_str(&serde_json::to_string(&spec).unwrap()).unwrap();
        assert_eq!(spec, again);
    }

    #[test]
    fn interior_equal_spacing_shelves_round_trips_and_validates() {
        let json = r#"{"version":1,"layout":{"type":"straight_run","width_mm":2400.0,"height_mm":2200.0,"depth_mm":600.0},"interior":{"type":"equal_spacing_shelves","shelf_count":4}}"#;
        let spec: WardrobeSpec = serde_json::from_str(json).unwrap();
        assert_eq!(
            spec.interior,
            Some(InteriorSpec::EqualSpacingShelves {
                shelf_count: 4,
                shelf_thickness_mm: None,
            })
        );
        validate_wardrobe_spec(&spec).unwrap();
    }

    #[test]
    fn interior_equal_spacing_rejects_impossible_fit() {
        let json = r#"{"version":1,"layout":{"type":"straight_run","width_mm":1000.0,"height_mm":100.0,"depth_mm":400.0},"interior":{"type":"equal_spacing_shelves","shelf_count":20,"shelf_thickness_mm":18.0}}"#;
        let err = parse_wardrobe_spec_json(json).unwrap_err();
        assert!(matches!(err, SpecError::Validation(_)));
    }

    #[test]
    fn golden_invalid_depth_fails_validation() {
        let err = parse_wardrobe_spec_json(INVALID_DEPTH.trim()).unwrap_err();
        assert!(matches!(err, SpecError::Validation(_)));
    }

    #[test]
    fn golden_equal_spacing_shelves_fixture_parse_and_validate() {
        let spec = parse_wardrobe_spec_json(EQUAL_SHELVES.trim()).unwrap();
        assert_eq!(
            spec.interior,
            Some(InteriorSpec::EqualSpacingShelves {
                shelf_count: 4,
                shelf_thickness_mm: None,
            })
        );
        validate_wardrobe_spec(&spec).unwrap();
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
    fn interior_explicit_shelf_heights_round_trips_and_validates() {
        let json = r#"{"version":1,"layout":{"type":"straight_run","width_mm":2400.0,"height_mm":2200.0,"depth_mm":600.0},"interior":{"type":"explicit_shelf_heights","shelf_bottom_y_mm":[400.0,1000.0,1600.0]}}"#;
        let spec: WardrobeSpec = serde_json::from_str(json).unwrap();
        assert_eq!(
            spec.interior,
            Some(InteriorSpec::ExplicitShelfHeights {
                shelf_bottom_y_mm: vec![400.0, 1000.0, 1600.0],
                shelf_thickness_mm: None,
                min_gap_mm: None,
            })
        );
        validate_wardrobe_spec(&spec).unwrap();
    }

    #[test]
    fn interior_explicit_rejects_overlap() {
        let json = r#"{"version":1,"layout":{"type":"straight_run","width_mm":2400.0,"height_mm":2200.0,"depth_mm":600.0},"interior":{"type":"explicit_shelf_heights","shelf_bottom_y_mm":[400.0,410.0],"shelf_thickness_mm":18.0}}"#;
        let err = parse_wardrobe_spec_json(json).unwrap_err();
        assert!(matches!(err, SpecError::Validation(_)));
    }

    #[test]
    fn golden_explicit_shelf_heights_fixture_parse_and_validate() {
        let spec = parse_wardrobe_spec_json(EXPLICIT_SHELVES.trim()).unwrap();
        assert_eq!(
            spec.interior,
            Some(InteriorSpec::ExplicitShelfHeights {
                shelf_bottom_y_mm: vec![400.0, 1000.0, 1600.0],
                shelf_thickness_mm: None,
                min_gap_mm: None,
            })
        );
        validate_wardrobe_spec(&spec).unwrap();
    }

    #[test]
    fn interior_zones_equal_fill_round_trips_and_validates() {
        let json = r#"{"version":1,"layout":{"type":"straight_run","width_mm":2400.0,"height_mm":2200.0,"depth_mm":600.0},"interior":{"type":"zones_equal_fill_shelves","bottom_zone_mm":500.0,"top_reserve_mm":300.0,"shelf_count":3}}"#;
        let spec: WardrobeSpec = serde_json::from_str(json).unwrap();
        assert_eq!(
            spec.interior,
            Some(InteriorSpec::ZonesEqualFillShelves {
                bottom_zone_mm: 500.0,
                top_reserve_mm: 300.0,
                shelf_count: 3,
                shelf_thickness_mm: None,
            })
        );
        validate_wardrobe_spec(&spec).unwrap();
    }

    #[test]
    fn interior_zones_equal_fill_rejects_zones_consuming_height() {
        let json = r#"{"version":1,"layout":{"type":"straight_run","width_mm":1000.0,"height_mm":1000.0,"depth_mm":400.0},"interior":{"type":"zones_equal_fill_shelves","bottom_zone_mm":600.0,"top_reserve_mm":500.0,"shelf_count":1}}"#;
        let err = parse_wardrobe_spec_json(json).unwrap_err();
        assert!(matches!(err, SpecError::Validation(_)));
    }

    #[test]
    fn golden_zones_equal_fill_fixture_parse_and_validate() {
        let spec = parse_wardrobe_spec_json(ZONES_EQUAL_FILL.trim()).unwrap();
        assert_eq!(
            spec.interior,
            Some(InteriorSpec::ZonesEqualFillShelves {
                bottom_zone_mm: 500.0,
                top_reserve_mm: 300.0,
                shelf_count: 3,
                shelf_thickness_mm: None,
            })
        );
        validate_wardrobe_spec(&spec).unwrap();
    }

    #[test]
    fn interior_golden_ratio_ladder_round_trips_and_validates() {
        let json = r#"{"version":1,"layout":{"type":"straight_run","width_mm":2400.0,"height_mm":2200.0,"depth_mm":600.0},"interior":{"type":"golden_ratio_ladder_shelves","rungs":3}}"#;
        let spec: WardrobeSpec = serde_json::from_str(json).unwrap();
        assert_eq!(
            spec.interior,
            Some(InteriorSpec::GoldenRatioLadderShelves {
                rungs: 3,
                shelf_thickness_mm: None,
            })
        );
        validate_wardrobe_spec(&spec).unwrap();
    }

    #[test]
    fn interior_golden_ratio_ladder_rejects_zero_rungs() {
        let json = r#"{"version":1,"layout":{"type":"straight_run","width_mm":2400.0,"height_mm":2200.0,"depth_mm":600.0},"interior":{"type":"golden_ratio_ladder_shelves","rungs":0}}"#;
        let err = parse_wardrobe_spec_json(json).unwrap_err();
        assert!(matches!(err, SpecError::Validation(_)));
    }

    #[test]
    fn golden_ratio_ladder_fixture_parse_and_validate() {
        let spec = parse_wardrobe_spec_json(GOLDEN_RATIO_LADDER.trim()).unwrap();
        assert_eq!(
            spec.interior,
            Some(InteriorSpec::GoldenRatioLadderShelves {
                rungs: 3,
                shelf_thickness_mm: None,
            })
        );
        validate_wardrobe_spec(&spec).unwrap();
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
