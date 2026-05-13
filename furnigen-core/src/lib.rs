//! FurniGen core: wardrobe spec, validation, and geometry (canonical units: **mm**).
//!
//! WASM and the web UI depend on this crate; keep it free of browser or Three.js APIs.

mod bom;
mod export_2d;
mod export_mesh;
mod inner_volume;
mod interior_shelves;
mod panels;
mod preview_mesh;
mod shelf_layout;
mod shelf_mesh;
mod spec;

pub use bom::{bom_to_csv, build_bom, BomDocument, BomPartRow};
pub use export_2d::{build_panels_dxf, build_panels_svg};
pub use export_mesh::{preview_mesh_to_gltf, preview_mesh_to_obj, MeshExportError};
pub use inner_volume::{straight_run_inner_volume_mm, ClearanceSpec, StraightRunInnerVolume};
pub use interior_shelves::{
    equal_spacing_shelf_bottoms_mm, golden_ratio_ladder_shelf_bottoms_mm,
    max_shelf_count_for_min_segment_mm, max_shelves_min_segment_shelf_bottoms_mm,
    seeded_random_min_gap_shelf_bottoms_mm, two_tier_rhythm_shelf_bottoms_mm,
    validate_equal_bays_in_width_mm, validate_explicit_shelf_bottoms_mm,
    weighted_random_band_shelf_bottoms_mm, zones_equal_fill_shelf_bottoms_mm, SplitMix64,
    DEFAULT_SHELF_THICKNESS_MM, MAX_SHELF_BOARDS,
};
pub use panels::{panel_blanks_for_spec, PanelBlank};
pub use preview_mesh::{build_preview_mesh, PreviewMesh};
pub use shelf_layout::{layout_interior_mm, ShelfBoardMm, UprightDividerMm};
pub use shelf_mesh::{
    append_shelf_bottom_face_mm, append_shelf_bottom_face_rect_mm, append_shelf_top_face_mm,
    append_shelf_top_face_rect_mm, append_upright_x_slab_mm,
};
pub use spec::{
    inner_volume_for_spec_parts, inner_volume_mm, parse_wardrobe_spec_json, validate_wardrobe_spec,
    InteriorSpec, LayoutSpec, SpecError, WardrobeSpec, WARDROBE_SPEC_VERSION,
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
