//! Resolved shelf boards (mm) from a validated [`crate::WardrobeSpec`].

use crate::interior_shelves::{
    equal_spacing_shelf_bottoms_mm, golden_ratio_ladder_shelf_bottoms_mm,
    max_shelves_min_segment_shelf_bottoms_mm, seeded_random_min_gap_shelf_bottoms_mm,
    two_tier_rhythm_shelf_bottoms_mm, validate_explicit_shelf_bottoms_mm,
    weighted_random_band_shelf_bottoms_mm, zones_equal_fill_shelf_bottoms_mm,
    DEFAULT_SHELF_THICKNESS_MM,
};
use crate::spec::inner_volume_for_spec_parts;
use crate::{InteriorSpec, WardrobeSpec};

/// One horizontal shelf board in inner coordinates (+X width, +Y up, +Z toward front).
#[derive(Debug, Clone, PartialEq)]
pub struct ShelfBoardMm {
    pub y_bottom_mm: f64,
    pub x0_mm: f64,
    pub x1_mm: f64,
    pub z0_mm: f64,
    pub z1_mm: f64,
}

/// Vertical upright divider between bays: thin slab in X, full inner height, full shelf depth span.
#[derive(Debug, Clone, PartialEq)]
pub struct UprightDividerMm {
    pub x0_mm: f64,
    pub x1_mm: f64,
    pub z0_mm: f64,
    pub z1_mm: f64,
}

fn shelf_span_xz_for_inner(
    spec: &WardrobeSpec,
    inner_w: f64,
    inner_d: f64,
) -> (f64, f64, f64, f64) {
    let side = spec
        .clearance
        .as_ref()
        .and_then(|c| c.side_inset_mm)
        .unwrap_or(0.0);
    let setback = spec
        .clearance
        .as_ref()
        .and_then(|c| c.front_setback_mm)
        .unwrap_or(0.0);
    let nosing = spec
        .clearance
        .as_ref()
        .and_then(|c| c.shelf_nosing_mm)
        .unwrap_or(0.0);
    let x0 = side.max(0.0);
    let x1 = (inner_w - side).max(x0 + 1e-6);
    let z0 = 0.0_f64;
    let z1 = (inner_d - setback + nosing).max(z0 + 1e-6);
    (x0, x1, z0, z1)
}

fn upright_dividers_mm(
    x0_span: f64,
    x1_span: f64,
    bay_count: u32,
    upright_thickness_mm: f64,
) -> Option<Vec<UprightDividerMm>> {
    if bay_count < 2 {
        return Some(Vec::new());
    }
    let m = bay_count as f64;
    let usable = (x1_span - x0_span).max(0.0);
    let tu = upright_thickness_mm;
    let bay_clear = (usable - (m - 1.0) * tu) / m;
    if bay_clear <= 1e-6 {
        return None;
    }
    let mut out = Vec::with_capacity(bay_count as usize - 1);
    for i in 0..bay_count - 1 {
        let ux0 = x0_span + (i as f64 + 1.0) * bay_clear + i as f64 * tu;
        let ux1 = ux0 + tu;
        out.push(UprightDividerMm {
            x0_mm: ux0,
            x1_mm: ux1,
            z0_mm: 0.0,
            z1_mm: 0.0, // filled by caller with inner depth span
        });
    }
    Some(out)
}

/// Shelf boards and optional upright dividers for preview / BOM. Returns `None` when interior absent or stub.
pub fn layout_interior_mm(
    spec: &WardrobeSpec,
) -> Option<(Vec<ShelfBoardMm>, Vec<UprightDividerMm>)> {
    let inner = inner_volume_for_spec_parts(&spec.layout, spec.clearance.as_ref());
    let (x0, x1, z0, z1) = shelf_span_xz_for_inner(spec, inner.width_mm, inner.depth_mm);
    let h = inner.height_mm;
    let interior = spec.interior.as_ref()?;

    let mut uprights: Vec<UprightDividerMm> = Vec::new();

    match interior {
        InteriorSpec::Stub => return None,
        InteriorSpec::EqualVerticalBaysEqualSpacingShelves {
            bay_count,
            shelf_count,
            upright_thickness_mm,
            shelf_thickness_mm,
        } => {
            let t = shelf_thickness_mm.unwrap_or(DEFAULT_SHELF_THICKNESS_MM);
            let tu = upright_thickness_mm.unwrap_or(18.0);
            let bottoms = equal_spacing_shelf_bottoms_mm(h, *shelf_count, t).ok()?;
            let m = *bay_count;
            if m < 1 {
                return None;
            }
            let mut boards = Vec::new();
            if m == 1 {
                for &yb in &bottoms {
                    boards.push(ShelfBoardMm {
                        y_bottom_mm: yb,
                        x0_mm: x0,
                        x1_mm: x1,
                        z0_mm: z0,
                        z1_mm: z1,
                    });
                }
                return Some((boards, uprights));
            }
            let mut u = upright_dividers_mm(x0, x1, m, tu)?;
            for q in &mut u {
                q.z0_mm = z0;
                q.z1_mm = z1;
            }
            uprights = u;
            let usable_w = (x1 - x0).max(0.0);
            let denom = m as f64;
            let bay_clear = (usable_w - (m - 1) as f64 * tu) / denom;
            if bay_clear <= 1e-6 {
                return None;
            }
            for b in 0..m {
                let bx0 = x0 + b as f64 * (bay_clear + tu);
                let bx1 = bx0 + bay_clear;
                for &yb in &bottoms {
                    boards.push(ShelfBoardMm {
                        y_bottom_mm: yb,
                        x0_mm: bx0,
                        x1_mm: bx1,
                        z0_mm: z0,
                        z1_mm: z1,
                    });
                }
            }
            return Some((boards, uprights));
        }
        InteriorSpec::GridUprightsExplicitRowsShelves {
            bay_count,
            shelf_bottom_y_mm,
            upright_thickness_mm,
            shelf_thickness_mm,
            min_gap_mm,
        } => {
            let t = shelf_thickness_mm.unwrap_or(DEFAULT_SHELF_THICKNESS_MM);
            let tu = upright_thickness_mm.unwrap_or(18.0);
            let bottoms =
                validate_explicit_shelf_bottoms_mm(h, shelf_bottom_y_mm, t, *min_gap_mm).ok()?;
            let m = *bay_count;
            if m < 1 {
                return None;
            }
            let mut boards = Vec::new();
            if m == 1 {
                for &yb in &bottoms {
                    boards.push(ShelfBoardMm {
                        y_bottom_mm: yb,
                        x0_mm: x0,
                        x1_mm: x1,
                        z0_mm: z0,
                        z1_mm: z1,
                    });
                }
                return Some((boards, uprights));
            }
            let mut u = upright_dividers_mm(x0, x1, m, tu)?;
            for q in &mut u {
                q.z0_mm = z0;
                q.z1_mm = z1;
            }
            uprights = u;
            let usable_w = (x1 - x0).max(0.0);
            let denom = m as f64;
            let bay_clear = (usable_w - (m - 1) as f64 * tu) / denom;
            if bay_clear <= 1e-6 {
                return None;
            }
            for b in 0..m {
                let bx0 = x0 + b as f64 * (bay_clear + tu);
                let bx1 = bx0 + bay_clear;
                for &yb in &bottoms {
                    boards.push(ShelfBoardMm {
                        y_bottom_mm: yb,
                        x0_mm: bx0,
                        x1_mm: bx1,
                        z0_mm: z0,
                        z1_mm: z1,
                    });
                }
            }
            return Some((boards, uprights));
        }
        _ => {}
    }

    let (bottoms, _t) = match interior {
        InteriorSpec::EqualSpacingShelves {
            shelf_count,
            shelf_thickness_mm,
        } => {
            let t = shelf_thickness_mm.unwrap_or(DEFAULT_SHELF_THICKNESS_MM);
            (equal_spacing_shelf_bottoms_mm(h, *shelf_count, t).ok()?, t)
        }
        InteriorSpec::ExplicitShelfHeights {
            shelf_bottom_y_mm,
            shelf_thickness_mm,
            min_gap_mm,
        } => {
            let t = shelf_thickness_mm.unwrap_or(DEFAULT_SHELF_THICKNESS_MM);
            (
                validate_explicit_shelf_bottoms_mm(h, shelf_bottom_y_mm, t, *min_gap_mm).ok()?,
                t,
            )
        }
        InteriorSpec::ZonesEqualFillShelves {
            bottom_zone_mm,
            top_reserve_mm,
            shelf_count,
            shelf_thickness_mm,
        } => {
            let t = shelf_thickness_mm.unwrap_or(DEFAULT_SHELF_THICKNESS_MM);
            (
                zones_equal_fill_shelf_bottoms_mm(
                    h,
                    *bottom_zone_mm,
                    *top_reserve_mm,
                    *shelf_count,
                    t,
                )
                .ok()?,
                t,
            )
        }
        InteriorSpec::GoldenRatioLadderShelves {
            rungs,
            shelf_thickness_mm,
        } => {
            let t = shelf_thickness_mm.unwrap_or(DEFAULT_SHELF_THICKNESS_MM);
            (golden_ratio_ladder_shelf_bottoms_mm(h, *rungs, t).ok()?, t)
        }
        InteriorSpec::TwoTierRhythmShelves {
            transition_y_mm,
            gap_lower_mm,
            gap_upper_mm,
            top_reserve_mm,
            shelf_thickness_mm,
        } => {
            let t = shelf_thickness_mm.unwrap_or(DEFAULT_SHELF_THICKNESS_MM);
            (
                two_tier_rhythm_shelf_bottoms_mm(
                    h,
                    *top_reserve_mm,
                    *transition_y_mm,
                    *gap_lower_mm,
                    *gap_upper_mm,
                    t,
                )
                .ok()?,
                t,
            )
        }
        InteriorSpec::MaxShelvesMinSegmentShelves {
            min_vertical_segment_mm,
            bottom_reserve_mm,
            top_reserve_mm,
            shelf_count,
            shelf_thickness_mm,
        } => {
            let t = shelf_thickness_mm.unwrap_or(DEFAULT_SHELF_THICKNESS_MM);
            (
                max_shelves_min_segment_shelf_bottoms_mm(
                    h,
                    *bottom_reserve_mm,
                    *top_reserve_mm,
                    *min_vertical_segment_mm,
                    t,
                    *shelf_count,
                )
                .ok()?,
                t,
            )
        }
        InteriorSpec::SeededRandomMinGapShelves {
            seed,
            shelf_count,
            min_gap_mm,
            bottom_reserve_mm,
            top_reserve_mm,
            shelf_thickness_mm,
        } => {
            let t = shelf_thickness_mm.unwrap_or(DEFAULT_SHELF_THICKNESS_MM);
            (
                seeded_random_min_gap_shelf_bottoms_mm(
                    h,
                    *bottom_reserve_mm,
                    *top_reserve_mm,
                    *shelf_count,
                    t,
                    *min_gap_mm,
                    *seed,
                )
                .ok()?,
                t,
            )
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
            let t = shelf_thickness_mm.unwrap_or(DEFAULT_SHELF_THICKNESS_MM);
            (
                weighted_random_band_shelf_bottoms_mm(
                    h,
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
                .ok()?,
                t,
            )
        }
        InteriorSpec::Stub
        | InteriorSpec::EqualVerticalBaysEqualSpacingShelves { .. }
        | InteriorSpec::GridUprightsExplicitRowsShelves { .. } => return None,
    };

    let mut boards = Vec::with_capacity(bottoms.len());
    for yb in bottoms {
        boards.push(ShelfBoardMm {
            y_bottom_mm: yb,
            x0_mm: x0,
            x1_mm: x1,
            z0_mm: z0,
            z1_mm: z1,
        });
    }
    Some((boards, uprights))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_wardrobe_spec_json;

    #[test]
    fn grid_stress_many_parts_stable_ordering() {
        let bays = 40u32;
        let rows = 25u32;
        let json = format!(
            r#"{{"version":1,"layout":{{"type":"straight_run","width_mm":4000.0,"height_mm":2800.0,"depth_mm":600.0}},"interior":{{"type":"equal_vertical_bays_equal_spacing_shelves","bay_count":{bays},"shelf_count":{rows}}}}}"#
        );
        let spec = parse_wardrobe_spec_json(&json).unwrap();
        let (boards, uprights) = layout_interior_mm(&spec).unwrap();
        assert_eq!(uprights.len(), (bays - 1) as usize);
        assert_eq!(boards.len(), (bays * rows) as usize);
        assert!(boards[0].x0_mm < boards[1].x0_mm || boards[0].y_bottom_mm < boards[1].y_bottom_mm);
    }
}
