//! Nominal rectangular panel blanks for the current layout (mm). Aligns with preview mesh faces.

use serde::Serialize;

use crate::interior_shelves::{
    equal_spacing_shelf_bottoms_mm, golden_ratio_ladder_shelf_bottoms_mm,
    max_shelves_min_segment_shelf_bottoms_mm, two_tier_rhythm_shelf_bottoms_mm,
    validate_explicit_shelf_bottoms_mm, zones_equal_fill_shelf_bottoms_mm,
    DEFAULT_SHELF_THICKNESS_MM,
};
use crate::{InteriorSpec, LayoutSpec, WardrobeSpec};

/// One rectangular stock panel: two in-plane dimensions before edge banding / thickness offsets.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PanelBlank {
    pub id: String,
    pub label: String,
    /// In-plane width of the blank (mm).
    pub width_mm: f64,
    /// In-plane height of the blank (mm).
    pub height_mm: f64,
}

/// Lists carcass panels for export (BOM, 2D outlines). v1: straight run, open front (no door).
pub fn panel_blanks_for_spec(spec: &WardrobeSpec) -> Vec<PanelBlank> {
    match &spec.layout {
        LayoutSpec::StraightRun {
            width_mm: w,
            height_mm: h,
            depth_mm: d,
        } => {
            let mut panels = straight_run_open_front_panels(*w, *h, *d);
            if let Some(interior) = &spec.interior {
                let bottoms = match interior {
                    InteriorSpec::EqualSpacingShelves {
                        shelf_count,
                        shelf_thickness_mm,
                    } => {
                        let t = shelf_thickness_mm.unwrap_or(DEFAULT_SHELF_THICKNESS_MM);
                        equal_spacing_shelf_bottoms_mm(*h, *shelf_count, t).ok()
                    }
                    InteriorSpec::ExplicitShelfHeights {
                        shelf_bottom_y_mm,
                        shelf_thickness_mm,
                        min_gap_mm,
                    } => {
                        let t = shelf_thickness_mm.unwrap_or(DEFAULT_SHELF_THICKNESS_MM);
                        validate_explicit_shelf_bottoms_mm(*h, shelf_bottom_y_mm, t, *min_gap_mm)
                            .ok()
                    }
                    InteriorSpec::ZonesEqualFillShelves {
                        bottom_zone_mm,
                        top_reserve_mm,
                        shelf_count,
                        shelf_thickness_mm,
                    } => {
                        let t = shelf_thickness_mm.unwrap_or(DEFAULT_SHELF_THICKNESS_MM);
                        zones_equal_fill_shelf_bottoms_mm(
                            *h,
                            *bottom_zone_mm,
                            *top_reserve_mm,
                            *shelf_count,
                            t,
                        )
                        .ok()
                    }
                    InteriorSpec::GoldenRatioLadderShelves {
                        rungs,
                        shelf_thickness_mm,
                    } => {
                        let t = shelf_thickness_mm.unwrap_or(DEFAULT_SHELF_THICKNESS_MM);
                        golden_ratio_ladder_shelf_bottoms_mm(*h, *rungs, t).ok()
                    }
                    InteriorSpec::TwoTierRhythmShelves {
                        transition_y_mm,
                        gap_lower_mm,
                        gap_upper_mm,
                        top_reserve_mm,
                        shelf_thickness_mm,
                    } => {
                        let t = shelf_thickness_mm.unwrap_or(DEFAULT_SHELF_THICKNESS_MM);
                        two_tier_rhythm_shelf_bottoms_mm(
                            *h,
                            *top_reserve_mm,
                            *transition_y_mm,
                            *gap_lower_mm,
                            *gap_upper_mm,
                            t,
                        )
                        .ok()
                    }
                    InteriorSpec::MaxShelvesMinSegmentShelves {
                        min_vertical_segment_mm,
                        bottom_reserve_mm,
                        top_reserve_mm,
                        shelf_count,
                        shelf_thickness_mm,
                    } => {
                        let t = shelf_thickness_mm.unwrap_or(DEFAULT_SHELF_THICKNESS_MM);
                        max_shelves_min_segment_shelf_bottoms_mm(
                            *h,
                            *bottom_reserve_mm,
                            *top_reserve_mm,
                            *min_vertical_segment_mm,
                            t,
                            *shelf_count,
                        )
                        .ok()
                    }
                    InteriorSpec::Stub => None,
                };
                if let Some(bottoms) = bottoms {
                    for i in 0..bottoms.len() {
                        let n = i + 1;
                        panels.push(PanelBlank {
                            id: format!("shelf_{n:02}"),
                            label: format!("Shelf {n}"),
                            width_mm: *w,
                            height_mm: *d,
                        });
                    }
                }
            }
            panels
        }
    }
}

fn straight_run_open_front_panels(w: f64, h: f64, d: f64) -> Vec<PanelBlank> {
    vec![
        PanelBlank {
            id: "back".to_owned(),
            label: "Back".to_owned(),
            width_mm: w,
            height_mm: h,
        },
        PanelBlank {
            id: "left_side".to_owned(),
            label: "Left side".to_owned(),
            width_mm: d,
            height_mm: h,
        },
        PanelBlank {
            id: "right_side".to_owned(),
            label: "Right side".to_owned(),
            width_mm: d,
            height_mm: h,
        },
        PanelBlank {
            id: "bottom".to_owned(),
            label: "Bottom".to_owned(),
            width_mm: w,
            height_mm: d,
        },
        PanelBlank {
            id: "top".to_owned(),
            label: "Top".to_owned(),
            width_mm: w,
            height_mm: d,
        },
    ]
}
