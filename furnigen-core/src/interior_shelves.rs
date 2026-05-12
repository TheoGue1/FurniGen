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

/// Validates user-supplied shelf bottom **Y** coordinates (mm from inner floor, ascending).
///
/// Rules:
/// - Empty list is valid (no shelf boards).
/// - Each value is finite; `shelf_thickness_mm` is finite and positive.
/// - Each bottom satisfies `0 < y < inner_height_mm` and `y + shelf_thickness_mm < inner_height_mm`.
/// - Bottoms are strictly ascending.
/// - For consecutive shelves, `next_bottom >= prev_bottom + shelf_thickness_mm + min_gap`.
///   `min_gap_mm` defaults to `0` when omitted; when present it must be finite and non-negative.
pub fn validate_explicit_shelf_bottoms_mm(
    inner_height_mm: f64,
    shelf_bottom_y_mm: &[f64],
    shelf_thickness_mm: f64,
    min_gap_mm: Option<f64>,
) -> Result<Vec<f64>, String> {
    if !inner_height_mm.is_finite() || inner_height_mm <= 0.0 {
        return Err("inner height must be finite and positive".to_owned());
    }
    if !shelf_thickness_mm.is_finite() || shelf_thickness_mm <= 0.0 {
        return Err("shelf_thickness_mm must be finite and positive".to_owned());
    }
    let min_gap = match min_gap_mm {
        None => 0.0,
        Some(g) => {
            if !g.is_finite() {
                return Err("min_gap_mm must be finite".to_owned());
            }
            if g < 0.0 {
                return Err("min_gap_mm must be non-negative".to_owned());
            }
            g
        }
    };
    if shelf_bottom_y_mm.is_empty() {
        return Ok(Vec::new());
    }
    for (i, &y) in shelf_bottom_y_mm.iter().enumerate() {
        if !y.is_finite() {
            return Err(format!("shelf_bottom_y_mm[{i}] must be finite"));
        }
        if y <= 0.0 {
            return Err(format!(
                "shelf_bottom_y_mm[{i}] must be strictly greater than 0 (inner floor), got {y}"
            ));
        }
        if y >= inner_height_mm {
            return Err(format!(
                "shelf_bottom_y_mm[{i}] must be strictly less than inner height {inner_height_mm} mm, got {y}"
            ));
        }
        let top = y + shelf_thickness_mm;
        if top >= inner_height_mm {
            return Err(format!(
                "shelf_bottom_y_mm[{i}] with thickness {shelf_thickness_mm} mm would reach or exceed inner top (height {inner_height_mm} mm)"
            ));
        }
    }
    for i in 0..shelf_bottom_y_mm.len() - 1 {
        let a = shelf_bottom_y_mm[i];
        let b = shelf_bottom_y_mm[i + 1];
        if b <= a {
            return Err(format!(
                "shelf_bottom_y_mm must be strictly ascending; index {i} is {a} but index {} is {b}",
                i + 1
            ));
        }
        let min_next = a + shelf_thickness_mm + min_gap;
        if b + 1e-9 < min_next {
            return Err(format!(
                "shelf overlap or insufficient vertical gap: shelf at y={a} mm (thickness {shelf_thickness_mm} mm) requires next bottom ≥ {min_next} mm (min_gap {min_gap} mm), but next is {b} mm"
            ));
        }
    }
    Ok(shelf_bottom_y_mm.to_vec())
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

    #[test]
    fn four_shelves_have_equal_air_gaps_and_non_overlapping_tops() {
        let h = 2200.0;
        let t = 18.0;
        let n = 4u32;
        let bottoms = equal_spacing_shelf_bottoms_mm(h, n, t).unwrap();
        assert_eq!(bottoms.len(), 4);
        let g = (h - f64::from(n) * t) / (f64::from(n) + 1.0);
        for (i, &yb) in bottoms.iter().enumerate() {
            let expected = g + f64::from(i as u32) * (g + t);
            assert!((yb - expected).abs() < 1e-6, "y[{i}]");
        }
        for i in 0..bottoms.len() - 1 {
            let gap_mid = bottoms[i] + t + (bottoms[i + 1] - (bottoms[i] + t)) / 2.0;
            assert!(gap_mid > bottoms[i] + t - 1e-6);
            assert!(gap_mid < bottoms[i + 1] + 1e-6);
        }
        assert!(bottoms[0] > 0.0);
        assert!(bottoms[3] + t < h);
    }

    #[test]
    fn explicit_empty_ok() {
        let v = validate_explicit_shelf_bottoms_mm(2200.0, &[], 18.0, None).unwrap();
        assert!(v.is_empty());
    }

    #[test]
    fn explicit_three_shelves_valid() {
        let bottoms = [400.0, 900.0, 1500.0];
        let v = validate_explicit_shelf_bottoms_mm(2200.0, &bottoms, 18.0, None).unwrap();
        assert_eq!(v, bottoms);
    }

    #[test]
    fn explicit_rejects_non_ascending() {
        let bottoms = [400.0, 400.0];
        assert!(validate_explicit_shelf_bottoms_mm(2200.0, &bottoms, 18.0, None).is_err());
    }

    #[test]
    fn explicit_rejects_overlap_without_min_gap() {
        let bottoms = [400.0, 410.0];
        assert!(validate_explicit_shelf_bottoms_mm(2200.0, &bottoms, 18.0, None).is_err());
    }

    #[test]
    fn explicit_min_gap_enforced() {
        let bottoms = [400.0, 420.0];
        assert!(validate_explicit_shelf_bottoms_mm(2200.0, &bottoms, 18.0, Some(5.0)).is_err());
        let ok = [400.0, 430.0];
        validate_explicit_shelf_bottoms_mm(2200.0, &ok, 18.0, Some(5.0)).unwrap();
    }

    #[test]
    fn explicit_rejects_bottom_at_or_below_zero() {
        assert!(validate_explicit_shelf_bottoms_mm(2200.0, &[0.0], 18.0, None).is_err());
    }

    #[test]
    fn explicit_rejects_top_past_inner_height() {
        assert!(validate_explicit_shelf_bottoms_mm(2200.0, &[2190.0], 18.0, None).is_err());
    }
}
