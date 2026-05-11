//! Inner carcass volume in **mm** (nominal). v1 ignores real panel thickness; matches outer straight-run box.

use crate::LayoutSpec;

/// Nominal inner width × height × depth for a straight run (mm).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StraightRunInnerVolume {
    pub width_mm: f64,
    pub height_mm: f64,
    pub depth_mm: f64,
}

/// Returns inner volume for the current layout. v1: straight run inner equals outer dimensions.
pub fn inner_volume_mm(layout: &LayoutSpec) -> StraightRunInnerVolume {
    match layout {
        LayoutSpec::StraightRun {
            width_mm,
            height_mm,
            depth_mm,
        } => StraightRunInnerVolume {
            width_mm: *width_mm,
            height_mm: *height_mm,
            depth_mm: *depth_mm,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LayoutSpec;

    #[test]
    fn straight_run_inner_matches_outer() {
        let layout = LayoutSpec::StraightRun {
            width_mm: 2400.0,
            height_mm: 2200.0,
            depth_mm: 600.0,
        };
        let inner = inner_volume_mm(&layout);
        assert!((inner.width_mm - 2400.0).abs() < 1e-9);
        assert!((inner.height_mm - 2200.0).abs() < 1e-9);
        assert!((inner.depth_mm - 600.0).abs() < 1e-9);
    }
}
