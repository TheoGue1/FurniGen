//! Wardrobe JSON contract (canonical **mm**). v1: straight runs only; `layout` is tagged for future variants.

use serde::{Deserialize, Serialize};
use serde_json::Map;
use thiserror::Error;

use crate::inner_volume::ClearanceSpec;

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
    /// Dense lower air gaps until a shelf top reaches `transition_y_mm`, then wider gaps; optional top reserve.
    TwoTierRhythmShelves {
        transition_y_mm: f64,
        gap_lower_mm: f64,
        gap_upper_mm: f64,
        #[serde(
            default = "default_f64_zero",
            skip_serializing_if = "is_default_f64_zero"
        )]
        top_reserve_mm: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        shelf_thickness_mm: Option<f64>,
    },
    /// Equal air gaps in a vertical band; shelf count defaults to the **maximum** that satisfies
    /// `min_vertical_segment_mm` on every segment, or an explicit `shelf_count` ≤ that maximum.
    MaxShelvesMinSegmentShelves {
        min_vertical_segment_mm: f64,
        #[serde(
            default = "default_f64_zero",
            skip_serializing_if = "is_default_f64_zero"
        )]
        bottom_reserve_mm: f64,
        #[serde(
            default = "default_f64_zero",
            skip_serializing_if = "is_default_f64_zero"
        )]
        top_reserve_mm: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        shelf_count: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        shelf_thickness_mm: Option<f64>,
    },
    /// Deterministic random shelf bottoms with `min_gap_mm` between boards and floor/ceiling reserves.
    SeededRandomMinGapShelves {
        seed: u64,
        shelf_count: u32,
        min_gap_mm: f64,
        #[serde(
            default = "default_f64_zero",
            skip_serializing_if = "is_default_f64_zero"
        )]
        bottom_reserve_mm: f64,
        #[serde(
            default = "default_f64_zero",
            skip_serializing_if = "is_default_f64_zero"
        )]
        top_reserve_mm: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        shelf_thickness_mm: Option<f64>,
    },
    /// Like [`InteriorSpec::SeededRandomMinGapShelves`], but slack slots use Gamma weights biased by vertical third.
    WeightedRandomBandShelves {
        seed: u64,
        shelf_count: u32,
        min_gap_mm: f64,
        #[serde(
            default = "default_f64_zero",
            skip_serializing_if = "is_default_f64_zero"
        )]
        bottom_reserve_mm: f64,
        #[serde(
            default = "default_f64_zero",
            skip_serializing_if = "is_default_f64_zero"
        )]
        top_reserve_mm: f64,
        #[serde(
            default = "default_one_f64",
            skip_serializing_if = "is_default_one_f64"
        )]
        band_weight_lower: f64,
        #[serde(
            default = "default_one_f64",
            skip_serializing_if = "is_default_one_f64"
        )]
        band_weight_middle: f64,
        #[serde(
            default = "default_one_f64",
            skip_serializing_if = "is_default_one_f64"
        )]
        band_weight_upper: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        shelf_thickness_mm: Option<f64>,
    },
    /// Equal clear-width bays separated by upright dividers; **same** equal-vertical spacing stack in each bay.
    EqualVerticalBaysEqualSpacingShelves {
        bay_count: u32,
        shelf_count: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        upright_thickness_mm: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        shelf_thickness_mm: Option<f64>,
    },
    /// Upright bays with **globally aligned** shelf rows from explicit bottom Y values (same heights in every bay).
    GridUprightsExplicitRowsShelves {
        bay_count: u32,
        shelf_bottom_y_mm: Vec<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        upright_thickness_mm: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        shelf_thickness_mm: Option<f64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min_gap_mm: Option<f64>,
    },
}

/// Root document exchanged with WASM and the web UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WardrobeSpec {
    pub version: u32,
    pub layout: LayoutSpec,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interior: Option<InteriorSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clearance: Option<ClearanceSpec>,
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

fn default_f64_zero() -> f64 {
    0.0
}

/// Inner straight-run volume (mm) from outer layout dimensions and optional clearance.
pub fn inner_volume_for_spec_parts(
    layout: &LayoutSpec,
    clearance: Option<&ClearanceSpec>,
) -> crate::inner_volume::StraightRunInnerVolume {
    match layout {
        LayoutSpec::StraightRun {
            width_mm,
            height_mm,
            depth_mm,
        } => crate::inner_volume::straight_run_inner_volume_mm(
            *width_mm, *height_mm, *depth_mm, clearance,
        ),
    }
}

/// Inner volume with no clearance shrink (legacy helper for tests and simple callers).
pub fn inner_volume_mm(layout: &LayoutSpec) -> crate::inner_volume::StraightRunInnerVolume {
    inner_volume_for_spec_parts(layout, None)
}

fn is_default_f64_zero(v: &f64) -> bool {
    *v == 0.0
}

fn default_one_f64() -> f64 {
    1.0
}

fn is_default_one_f64(v: &f64) -> bool {
    (*v - 1.0).abs() < f64::EPSILON
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
            validate_clearance(spec.clearance.as_ref())?;
            let inner = inner_volume_for_spec_parts(&spec.layout, spec.clearance.as_ref());
            validate_inner_volume_positive(&inner)?;
            validate_interior_for_inner(spec.interior.as_ref(), spec, &inner)?;
        }
    }
    Ok(())
}

fn validate_clearance(c: Option<&ClearanceSpec>) -> Result<(), SpecError> {
    let Some(c) = c else {
        return Ok(());
    };
    for (label, v) in [
        ("carcass_panel_thickness_mm", c.carcass_panel_thickness_mm),
        ("side_inset_mm", c.side_inset_mm),
        ("front_setback_mm", c.front_setback_mm),
        ("shelf_nosing_mm", c.shelf_nosing_mm),
    ] {
        if let Some(x) = v {
            if !x.is_finite() {
                return Err(SpecError::Validation(format!(
                    "{label} must be finite when provided"
                )));
            }
            if x < 0.0 {
                return Err(SpecError::Validation(format!(
                    "{label} must be non-negative when provided"
                )));
            }
        }
    }
    Ok(())
}

fn validate_inner_volume_positive(
    inner: &crate::inner_volume::StraightRunInnerVolume,
) -> Result<(), SpecError> {
    const EPS: f64 = 1e-6;
    if inner.width_mm <= EPS || inner.height_mm <= EPS || inner.depth_mm <= EPS {
        return Err(SpecError::Validation(format!(
            "clearance / panel thickness leaves non-positive inner volume (inner {:.3}×{:.3}×{:.3} mm); reduce carcass_panel_thickness_mm or enlarge outer dimensions",
            inner.width_mm, inner.height_mm, inner.depth_mm
        )));
    }
    Ok(())
}

fn validate_interior_for_inner(
    interior: Option<&InteriorSpec>,
    spec: &WardrobeSpec,
    inner: &crate::inner_volume::StraightRunInnerVolume,
) -> Result<(), SpecError> {
    let inner_height_mm = inner.height_mm;
    let inner_width_mm = inner.width_mm;
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
        InteriorSpec::TwoTierRhythmShelves {
            transition_y_mm,
            gap_lower_mm,
            gap_upper_mm,
            top_reserve_mm,
            shelf_thickness_mm,
        } => {
            let t =
                shelf_thickness_mm.unwrap_or(crate::interior_shelves::DEFAULT_SHELF_THICKNESS_MM);
            validate_positive_finite("shelf_thickness_mm", t)?;
            if !top_reserve_mm.is_finite() || *top_reserve_mm < 0.0 {
                return Err(SpecError::Validation(
                    "top_reserve_mm must be finite and non-negative".to_owned(),
                ));
            }
            crate::interior_shelves::two_tier_rhythm_shelf_bottoms_mm(
                inner_height_mm,
                *top_reserve_mm,
                *transition_y_mm,
                *gap_lower_mm,
                *gap_upper_mm,
                t,
            )
            .map_err(SpecError::Validation)?;
            Ok(())
        }
        InteriorSpec::MaxShelvesMinSegmentShelves {
            min_vertical_segment_mm,
            bottom_reserve_mm,
            top_reserve_mm,
            shelf_count,
            shelf_thickness_mm,
        } => {
            let t =
                shelf_thickness_mm.unwrap_or(crate::interior_shelves::DEFAULT_SHELF_THICKNESS_MM);
            validate_positive_finite("shelf_thickness_mm", t)?;
            if !bottom_reserve_mm.is_finite() || *bottom_reserve_mm < 0.0 {
                return Err(SpecError::Validation(
                    "bottom_reserve_mm must be finite and non-negative".to_owned(),
                ));
            }
            if !top_reserve_mm.is_finite() || *top_reserve_mm < 0.0 {
                return Err(SpecError::Validation(
                    "top_reserve_mm must be finite and non-negative".to_owned(),
                ));
            }
            if !min_vertical_segment_mm.is_finite() || *min_vertical_segment_mm <= 0.0 {
                return Err(SpecError::Validation(
                    "min_vertical_segment_mm must be finite and positive".to_owned(),
                ));
            }
            crate::interior_shelves::max_shelves_min_segment_shelf_bottoms_mm(
                inner_height_mm,
                *bottom_reserve_mm,
                *top_reserve_mm,
                *min_vertical_segment_mm,
                t,
                *shelf_count,
            )
            .map_err(SpecError::Validation)?;
            Ok(())
        }
        InteriorSpec::SeededRandomMinGapShelves {
            seed,
            shelf_count,
            min_gap_mm,
            bottom_reserve_mm,
            top_reserve_mm,
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
            crate::interior_shelves::seeded_random_min_gap_shelf_bottoms_mm(
                inner_height_mm,
                *bottom_reserve_mm,
                *top_reserve_mm,
                *shelf_count,
                t,
                *min_gap_mm,
                *seed,
            )
            .map_err(SpecError::Validation)?;
            Ok(())
        }
        InteriorSpec::WeightedRandomBandShelves {
            seed,
            shelf_count,
            min_gap_mm,
            bottom_reserve_mm,
            top_reserve_mm,
            band_weight_lower,
            band_weight_middle,
            band_weight_upper,
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
            crate::interior_shelves::weighted_random_band_shelf_bottoms_mm(
                inner_height_mm,
                *bottom_reserve_mm,
                *top_reserve_mm,
                *shelf_count,
                t,
                *min_gap_mm,
                *seed,
                *band_weight_lower,
                *band_weight_middle,
                *band_weight_upper,
            )
            .map_err(SpecError::Validation)?;
            Ok(())
        }
        InteriorSpec::EqualVerticalBaysEqualSpacingShelves {
            bay_count,
            shelf_count,
            upright_thickness_mm,
            shelf_thickness_mm,
        } => {
            if *bay_count < 1 {
                return Err(SpecError::Validation(
                    "bay_count must be at least 1".to_owned(),
                ));
            }
            if *shelf_count < 1 {
                return Err(SpecError::Validation(
                    "shelf_count must be at least 1".to_owned(),
                ));
            }
            let t =
                shelf_thickness_mm.unwrap_or(crate::interior_shelves::DEFAULT_SHELF_THICKNESS_MM);
            validate_positive_finite("shelf_thickness_mm", t)?;
            let tu = upright_thickness_mm.unwrap_or(18.0);
            validate_positive_finite("upright_thickness_mm", tu)?;
            let side = spec
                .clearance
                .as_ref()
                .and_then(|c| c.side_inset_mm)
                .unwrap_or(0.0);
            let usable_w = inner_width_mm - 2.0 * side;
            crate::interior_shelves::validate_equal_bays_in_width_mm(usable_w, *bay_count, tu)
                .map_err(SpecError::Validation)?;
            crate::interior_shelves::equal_spacing_shelf_bottoms_mm(
                inner_height_mm,
                *shelf_count,
                t,
            )
            .map_err(SpecError::Validation)?;
            Ok(())
        }
        InteriorSpec::GridUprightsExplicitRowsShelves {
            bay_count,
            shelf_bottom_y_mm,
            upright_thickness_mm,
            shelf_thickness_mm,
            min_gap_mm,
        } => {
            if *bay_count < 1 {
                return Err(SpecError::Validation(
                    "bay_count must be at least 1".to_owned(),
                ));
            }
            let t =
                shelf_thickness_mm.unwrap_or(crate::interior_shelves::DEFAULT_SHELF_THICKNESS_MM);
            validate_positive_finite("shelf_thickness_mm", t)?;
            let tu = upright_thickness_mm.unwrap_or(18.0);
            validate_positive_finite("upright_thickness_mm", tu)?;
            let side = spec
                .clearance
                .as_ref()
                .and_then(|c| c.side_inset_mm)
                .unwrap_or(0.0);
            let usable_w = inner_width_mm - 2.0 * side;
            crate::interior_shelves::validate_equal_bays_in_width_mm(usable_w, *bay_count, tu)
                .map_err(SpecError::Validation)?;
            crate::interior_shelves::validate_explicit_shelf_bottoms_mm(
                inner_height_mm,
                shelf_bottom_y_mm,
                t,
                *min_gap_mm,
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
    const TWO_TIER_RHYTHM: &str =
        include_str!("../../spec-fixtures/wardrobe-spec-v1-two-tier-rhythm-shelves.json");
    const MAX_SHELVES_MIN_SEGMENT: &str =
        include_str!("../../spec-fixtures/wardrobe-spec-v1-max-shelves-min-segment-shelves.json");
    const SEEDED_RANDOM_MIN_GAP: &str =
        include_str!("../../spec-fixtures/wardrobe-spec-v1-seeded-random-min-gap-shelves.json");
    const WEIGHTED_RANDOM_BAND: &str =
        include_str!("../../spec-fixtures/wardrobe-spec-v1-weighted-random-band-shelves.json");
    const EQUAL_VERTICAL_BAYS: &str =
        include_str!("../../spec-fixtures/wardrobe-spec-v1-equal-vertical-bays-shelves.json");
    const GRID_UPRIGHTS_EXPLICIT: &str =
        include_str!("../../spec-fixtures/wardrobe-spec-v1-grid-uprights-explicit-rows.json");
    const CLEARANCE_THICKNESS: &str =
        include_str!("../../spec-fixtures/wardrobe-spec-v1-clearance-thickness.json");

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
    fn golden_two_tier_rhythm_fixture_parse_and_validate() {
        let spec = parse_wardrobe_spec_json(TWO_TIER_RHYTHM.trim()).unwrap();
        assert_eq!(
            spec.interior,
            Some(InteriorSpec::TwoTierRhythmShelves {
                transition_y_mm: 900.0,
                gap_lower_mm: 80.0,
                gap_upper_mm: 200.0,
                top_reserve_mm: 0.0,
                shelf_thickness_mm: None,
            })
        );
        validate_wardrobe_spec(&spec).unwrap();
    }

    #[test]
    fn interior_two_tier_rejects_gap_upper_below_lower() {
        let json = r#"{"version":1,"layout":{"type":"straight_run","width_mm":2400.0,"height_mm":2200.0,"depth_mm":600.0},"interior":{"type":"two_tier_rhythm_shelves","transition_y_mm":900.0,"gap_lower_mm":120.0,"gap_upper_mm":80.0}}"#;
        let err = parse_wardrobe_spec_json(json).unwrap_err();
        assert!(matches!(err, SpecError::Validation(_)));
    }

    #[test]
    fn golden_max_shelves_min_segment_fixture_parse_and_validate() {
        let spec = parse_wardrobe_spec_json(MAX_SHELVES_MIN_SEGMENT.trim()).unwrap();
        assert_eq!(
            spec.interior,
            Some(InteriorSpec::MaxShelvesMinSegmentShelves {
                min_vertical_segment_mm: 100.0,
                bottom_reserve_mm: 0.0,
                top_reserve_mm: 0.0,
                shelf_count: None,
                shelf_thickness_mm: None,
            })
        );
        validate_wardrobe_spec(&spec).unwrap();
    }

    #[test]
    fn interior_max_shelves_rejects_shelf_count_above_feasible() {
        let json = r#"{"version":1,"layout":{"type":"straight_run","width_mm":2400.0,"height_mm":2200.0,"depth_mm":600.0},"interior":{"type":"max_shelves_min_segment_shelves","min_vertical_segment_mm":100.0,"shelf_count":99}}"#;
        let err = parse_wardrobe_spec_json(json).unwrap_err();
        assert!(matches!(err, SpecError::Validation(_)));
    }

    #[test]
    fn serde_round_trip_preserves_layout() {
        let spec = parse_wardrobe_spec_json(MINIMAL.trim()).unwrap();
        let json = serde_json::to_string(&spec).unwrap();
        let back: WardrobeSpec = serde_json::from_str(&json).unwrap();
        validate_wardrobe_spec(&back).unwrap();
        assert_eq!(spec, back);
    }

    #[test]
    fn golden_seeded_random_min_gap_fixture_parse_and_validate() {
        let spec = parse_wardrobe_spec_json(SEEDED_RANDOM_MIN_GAP.trim()).unwrap();
        assert!(matches!(
            spec.interior,
            Some(InteriorSpec::SeededRandomMinGapShelves { .. })
        ));
        validate_wardrobe_spec(&spec).unwrap();
    }

    #[test]
    fn golden_weighted_random_band_fixture_parse_and_validate() {
        let spec = parse_wardrobe_spec_json(WEIGHTED_RANDOM_BAND.trim()).unwrap();
        assert!(matches!(
            spec.interior,
            Some(InteriorSpec::WeightedRandomBandShelves { .. })
        ));
        validate_wardrobe_spec(&spec).unwrap();
    }

    #[test]
    fn golden_equal_vertical_bays_fixture_parse_and_validate() {
        let spec = parse_wardrobe_spec_json(EQUAL_VERTICAL_BAYS.trim()).unwrap();
        assert!(matches!(
            spec.interior,
            Some(InteriorSpec::EqualVerticalBaysEqualSpacingShelves { .. })
        ));
        validate_wardrobe_spec(&spec).unwrap();
    }

    #[test]
    fn golden_grid_uprights_explicit_rows_fixture_parse_and_validate() {
        let spec = parse_wardrobe_spec_json(GRID_UPRIGHTS_EXPLICIT.trim()).unwrap();
        assert!(matches!(
            spec.interior,
            Some(InteriorSpec::GridUprightsExplicitRowsShelves { .. })
        ));
        validate_wardrobe_spec(&spec).unwrap();
    }

    #[test]
    fn golden_clearance_thickness_fixture_parse_and_validate() {
        let spec = parse_wardrobe_spec_json(CLEARANCE_THICKNESS.trim()).unwrap();
        assert!(spec.clearance.is_some());
        validate_wardrobe_spec(&spec).unwrap();
        let inner = inner_volume_for_spec_parts(&spec.layout, spec.clearance.as_ref());
        assert!((inner.width_mm - (2400.0 - 36.0)).abs() < 1e-6);
    }
}
