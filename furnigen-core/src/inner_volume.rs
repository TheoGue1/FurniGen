//! Inner carcass volume in **mm** (nominal). Optional [`ClearanceSpec`] shrinks the straight-run inner box.

use serde::{Deserialize, Serialize};

/// Optional panel thickness and shelf setbacks (mm). Omitted fields behave like **zero** / absent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ClearanceSpec {
    /// Nominal thickness subtracted from left/right, top/bottom, and back only (front stays open).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub carcass_panel_thickness_mm: Option<f64>,
    /// Extra horizontal inset of shelf boards from the **inner** side faces (mm per side).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side_inset_mm: Option<f64>,
    /// Reduces shelf depth from the inner front plane (mm).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub front_setback_mm: Option<f64>,
    /// Extends shelf boards toward the room beyond the setback inner front (mm).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shelf_nosing_mm: Option<f64>,
}

/// Nominal inner width × height × depth for a straight run (mm).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StraightRunInnerVolume {
    pub width_mm: f64,
    pub height_mm: f64,
    pub depth_mm: f64,
}

/// v1 straight run: subtracts **one** thickness from depth (back panel) and **two** from width and height.
pub fn straight_run_inner_volume_mm(
    width_mm: f64,
    height_mm: f64,
    depth_mm: f64,
    clearance: Option<&ClearanceSpec>,
) -> StraightRunInnerVolume {
    let t = clearance
        .and_then(|c| c.carcass_panel_thickness_mm)
        .filter(|v| v.is_finite() && *v >= 0.0)
        .unwrap_or(0.0);
    StraightRunInnerVolume {
        width_mm: (width_mm - 2.0 * t).max(0.0),
        height_mm: (height_mm - 2.0 * t).max(0.0),
        depth_mm: (depth_mm - t).max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_run_inner_matches_outer_without_clearance() {
        let inner = straight_run_inner_volume_mm(2400.0, 2200.0, 600.0, None);
        assert!((inner.width_mm - 2400.0).abs() < 1e-9);
        assert!((inner.height_mm - 2200.0).abs() < 1e-9);
        assert!((inner.depth_mm - 600.0).abs() < 1e-9);
    }

    #[test]
    fn clearance_thickness_shrinks_inner_box() {
        let c = ClearanceSpec {
            carcass_panel_thickness_mm: Some(18.0),
            side_inset_mm: None,
            front_setback_mm: None,
            shelf_nosing_mm: None,
        };
        let inner = straight_run_inner_volume_mm(2400.0, 2200.0, 600.0, Some(&c));
        assert!((inner.width_mm - (2400.0 - 36.0)).abs() < 1e-9);
        assert!((inner.height_mm - (2200.0 - 36.0)).abs() < 1e-9);
        assert!((inner.depth_mm - (600.0 - 18.0)).abs() < 1e-9);
    }
}
