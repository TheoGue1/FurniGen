//! Shelf placement from interior spec (mm). v1: equal vertical gaps between shelf boards and inner floor/ceiling.

/// Nominal shelf thickness when JSON omits `shelf_thickness_mm` (mm).
pub const DEFAULT_SHELF_THICKNESS_MM: f64 = 18.0;

const MIN_AIR_GAP_MM: f64 = 0.01;

/// Bottom **Y** of each horizontal shelf board (mm), ordered upward.
///
/// `inner_height_mm` is inner box height. There are `shelf_count + 1` equal air gaps: under the first
/// shelf, between boards, and above the last shelf (below the inner top).
pub fn equal_spacing_shelf_bottoms_mm(
    inner_height_mm: f64,
    shelf_count: u32,
    shelf_thickness_mm: f64,
) -> Result<Vec<f64>, String> {
    if shelf_count < 1 {
        return Err("shelf_count must be at least 1".to_owned());
    }
    if !shelf_thickness_mm.is_finite() || shelf_thickness_mm <= 0.0 {
        return Err("shelf_thickness_mm must be finite and positive".to_owned());
    }
    if !inner_height_mm.is_finite() || inner_height_mm <= 0.0 {
        return Err("inner height must be finite and positive".to_owned());
    }
    let n = f64::from(shelf_count);
    let usable = inner_height_mm - n * shelf_thickness_mm;
    if usable <= 0.0 {
        return Err(format!(
            "inner height {inner_height_mm} mm is too small for {shelf_count} shelf board(s) of {shelf_thickness_mm} mm each (need positive gaps)"
        ));
    }
    let g = usable / (n + 1.0);
    if g < MIN_AIR_GAP_MM {
        return Err("equal vertical gap would be below minimum".to_owned());
    }
    let mut out = Vec::with_capacity(shelf_count as usize);
    for k in 0..shelf_count {
        let k = f64::from(k);
        let y_bottom = g + k * (g + shelf_thickness_mm);
        out.push(y_bottom);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_shelf_centered_air() {
        let h = 2200.0;
        let t = 18.0;
        let bottoms = equal_spacing_shelf_bottoms_mm(h, 1, t).unwrap();
        assert_eq!(bottoms.len(), 1);
        let g = (h - t) / 2.0;
        assert!((bottoms[0] - g).abs() < 1e-9);
    }

    #[test]
    fn rejects_zero_shelves() {
        assert!(equal_spacing_shelf_bottoms_mm(2200.0, 0, 18.0).is_err());
    }
}
