//! Shelf placement from interior spec (mm): equal spacing, zones, explicit heights, golden-ratio ladders, …

/// Nominal shelf thickness when JSON omits `shelf_thickness_mm` (mm).
pub const DEFAULT_SHELF_THICKNESS_MM: f64 = 18.0;

const MIN_AIR_GAP_MM: f64 = 0.01;

fn golden_ratio_phi() -> f64 {
    (1.0 + 5.0_f64.sqrt()) / 2.0
}

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

/// Bottom **Y** of each horizontal shelf board (mm), ordered upward, with equal air gaps **only inside the middle band**.
///
/// The band is the open vertical interval `(bottom_zone_mm, inner_height_mm - top_reserve_mm)` in inner coordinates.
/// `bottom_zone_mm` and `top_reserve_mm` are reserved measured from the inner floor and inner ceiling respectively
/// (non-negative; their sum must be strictly less than `inner_height_mm`). Shelf boards of thickness
/// `shelf_thickness_mm` never intrude into the reserved bands: there are `shelf_count + 1` equal air gaps spanning
/// the band (below the first shelf, between boards, above the last shelf), same rule as [`equal_spacing_shelf_bottoms_mm`]
/// but with effective inner height `inner_height_mm - bottom_zone_mm - top_reserve_mm`.
pub fn zones_equal_fill_shelf_bottoms_mm(
    inner_height_mm: f64,
    bottom_zone_mm: f64,
    top_reserve_mm: f64,
    shelf_count: u32,
    shelf_thickness_mm: f64,
) -> Result<Vec<f64>, String> {
    if shelf_count < 1 {
        return Err("shelf_count must be at least 1".to_owned());
    }
    if !bottom_zone_mm.is_finite() || bottom_zone_mm < 0.0 {
        return Err("bottom_zone_mm must be finite and non-negative".to_owned());
    }
    if !top_reserve_mm.is_finite() || top_reserve_mm < 0.0 {
        return Err("top_reserve_mm must be finite and non-negative".to_owned());
    }
    if !inner_height_mm.is_finite() || inner_height_mm <= 0.0 {
        return Err("inner height must be finite and positive".to_owned());
    }
    let band_mm = inner_height_mm - bottom_zone_mm - top_reserve_mm;
    if band_mm <= 0.0 {
        return Err(format!(
            "bottom_zone_mm ({bottom_zone_mm}) + top_reserve_mm ({top_reserve_mm}) must be strictly less than inner height ({inner_height_mm} mm) so the middle band has positive height"
        ));
    }
    let in_band = equal_spacing_shelf_bottoms_mm(band_mm, shelf_count, shelf_thickness_mm)?;
    Ok(in_band.into_iter().map(|y| y + bottom_zone_mm).collect())
}

/// Bottom **Y** of each horizontal shelf board (mm), ordered upward, from a **golden-ratio air-gap ladder**.
///
/// For `rungs` shelf boards there are `rungs + 1` vertical air gaps (below the first shelf, between boards,
/// above the last). Gap *i* (0-based from the inner floor) has weight φ^i with φ = (1+√5)/2, so the bottom
/// gap is the smallest and the top gap is the largest. Total air is `inner_height_mm - rungs * shelf_thickness_mm`;
/// gaps are normalized to sum to that total. Fails if any gap would fall below [`MIN_AIR_GAP_MM`] or the
/// boards do not fit.
pub fn golden_ratio_ladder_shelf_bottoms_mm(
    inner_height_mm: f64,
    rungs: u32,
    shelf_thickness_mm: f64,
) -> Result<Vec<f64>, String> {
    if rungs < 1 {
        return Err("rungs must be at least 1".to_owned());
    }
    if !shelf_thickness_mm.is_finite() || shelf_thickness_mm <= 0.0 {
        return Err("shelf_thickness_mm must be finite and positive".to_owned());
    }
    if !inner_height_mm.is_finite() || inner_height_mm <= 0.0 {
        return Err("inner height must be finite and positive".to_owned());
    }
    let n = rungs as usize;
    let num_gaps = n + 1;
    let phi = golden_ratio_phi();
    let mut weight_sum = 0.0;
    let mut w = 1.0;
    for _ in 0..num_gaps {
        weight_sum += w;
        w *= phi;
    }
    let usable = inner_height_mm - (rungs as f64) * shelf_thickness_mm;
    if usable <= 0.0 {
        return Err(format!(
            "inner height {inner_height_mm} mm is too small for {rungs} shelf board(s) of {shelf_thickness_mm} mm each (need positive air)"
        ));
    }
    let mut bottoms = Vec::with_capacity(n);
    let mut y_cursor = 0.0;
    let mut w = 1.0;
    for i in 0..num_gaps {
        let g = usable * w / weight_sum;
        if g < MIN_AIR_GAP_MM {
            return Err("golden-ratio ladder air gap would be below minimum".to_owned());
        }
        y_cursor += g;
        if i < n {
            bottoms.push(y_cursor);
            y_cursor += shelf_thickness_mm;
        }
        w *= phi;
    }
    Ok(bottoms)
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
    fn zones_equal_fill_matches_band_subtraction() {
        let h = 2200.0;
        let bottom_z = 500.0;
        let top_r = 300.0;
        let band = h - bottom_z - top_r;
        let t = 18.0;
        let n = 3u32;
        let zoned = zones_equal_fill_shelf_bottoms_mm(h, bottom_z, top_r, n, t).unwrap();
        let in_band = equal_spacing_shelf_bottoms_mm(band, n, t).unwrap();
        assert_eq!(zoned.len(), in_band.len());
        for (a, b) in zoned.iter().zip(in_band.iter()) {
            assert!((a - (b + bottom_z)).abs() < 1e-9);
        }
        assert!(zoned[0] > bottom_z);
        assert!(zoned[n as usize - 1] + t < h - top_r);
    }

    #[test]
    fn zones_equal_fill_rejects_zones_that_consume_full_height() {
        assert!(zones_equal_fill_shelf_bottoms_mm(2200.0, 1000.0, 1200.0, 1, 18.0).is_err());
    }

    #[test]
    fn zones_equal_fill_rejects_negative_zone() {
        assert!(zones_equal_fill_shelf_bottoms_mm(2200.0, -1.0, 0.0, 1, 18.0).is_err());
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

    #[test]
    fn golden_ratio_ladder_three_rungs_stable_positions() {
        let h = 2200.0;
        let t = 18.0;
        let bottoms = golden_ratio_ladder_shelf_bottoms_mm(h, 3, t).unwrap();
        assert_eq!(bottoms.len(), 3);
        let expected = [226.559248114181, 611.139812028545, 1222.27962405709];
        for (i, (&a, &e)) in bottoms.iter().zip(expected.iter()).enumerate() {
            assert!((a - e).abs() < 1e-6, "bottom[{i}] got {a} want {e}");
        }
        assert!(bottoms[0] > 0.0);
        for i in 0..bottoms.len() - 1 {
            assert!(bottoms[i + 1] > bottoms[i] + t - 1e-6);
        }
        assert!(bottoms[2] + t < h - 1e-6);
    }

    #[test]
    fn golden_ratio_ladder_consecutive_air_gaps_ratio_phi() {
        let h = 1000.0;
        let t = 10.0;
        let rungs = 4u32;
        let bottoms = golden_ratio_ladder_shelf_bottoms_mm(h, rungs, t).unwrap();
        let phi = super::golden_ratio_phi();
        let mut gaps = Vec::with_capacity(rungs as usize + 1);
        gaps.push(bottoms[0]);
        for i in 0..bottoms.len() - 1 {
            gaps.push(bottoms[i + 1] - bottoms[i] - t);
        }
        gaps.push(h - bottoms[bottoms.len() - 1] - t);
        assert_eq!(gaps.len(), rungs as usize + 1);
        for i in 0..gaps.len() - 1 {
            let r = gaps[i + 1] / gaps[i];
            assert!((r - phi).abs() < 1e-9, "gap ratio {i}: {r} vs phi {phi}");
        }
    }

    #[test]
    fn golden_ratio_ladder_rejects_zero_rungs() {
        assert!(golden_ratio_ladder_shelf_bottoms_mm(2200.0, 0, 18.0).is_err());
    }

    #[test]
    fn golden_ratio_ladder_rejects_too_many_boards_for_height() {
        assert!(golden_ratio_ladder_shelf_bottoms_mm(50.0, 10, 18.0).is_err());
    }
}
