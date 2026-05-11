//! Nominal rectangular panel blanks for the current layout (mm). Aligns with preview mesh faces.

use serde::Serialize;

use crate::{LayoutSpec, WardrobeSpec};

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
        } => straight_run_open_front_panels(*w, *h, *d),
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
