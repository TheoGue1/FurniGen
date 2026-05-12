//! Shelf placement from interior spec (mm): equal spacing, zones, explicit heights, golden-ratio ladders, …

/// Nominal shelf thickness when JSON omits `shelf_thickness_mm` (mm).
pub const DEFAULT_SHELF_THICKNESS_MM: f64 = 18.0;

/// Upper bound on shelf board count for greedy / packing modes (matches other interior modes).
pub const MAX_SHELF_BOARDS: u32 = 500;

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

/// Bottom **Y** of each horizontal shelf board (mm), ordered upward, from a **two-tier vertical rhythm**.
///
/// Shelves are packed bottom-up from the inner floor. The first air gap (floor → first shelf bottom)
/// is `gap_lower_mm`. After each shelf, the next air gap is `gap_lower_mm` while the **previous**
/// shelf’s top is strictly below `transition_y_mm`, otherwise `gap_upper_mm` (typically wider).
/// Packing stops when another shelf would violate a minimum ceiling air gap (0.01 mm, same as other modes)
/// below `inner_height_mm - top_reserve_mm`.
///
/// Requires at least one shelf that fits. `gap_lower_mm` and `gap_upper_mm` must be ≥ the core minimum air gap;
/// `gap_upper_mm` must be ≥ `gap_lower_mm`. `transition_y_mm` lies on the open interval
/// `(0, inner_height_mm)`.
pub fn two_tier_rhythm_shelf_bottoms_mm(
    inner_height_mm: f64,
    top_reserve_mm: f64,
    transition_y_mm: f64,
    gap_lower_mm: f64,
    gap_upper_mm: f64,
    shelf_thickness_mm: f64,
) -> Result<Vec<f64>, String> {
    if !inner_height_mm.is_finite() || inner_height_mm <= 0.0 {
        return Err("inner height must be finite and positive".to_owned());
    }
    if !top_reserve_mm.is_finite() || top_reserve_mm < 0.0 {
        return Err("top_reserve_mm must be finite and non-negative".to_owned());
    }
    if top_reserve_mm >= inner_height_mm {
        return Err(format!(
            "top_reserve_mm ({top_reserve_mm}) must be strictly less than inner height ({inner_height_mm} mm)"
        ));
    }
    if !transition_y_mm.is_finite()
        || transition_y_mm <= 0.0
        || transition_y_mm >= inner_height_mm
    {
        return Err(format!(
            "transition_y_mm must be finite and strictly between 0 and inner height ({inner_height_mm} mm)"
        ));
    }
    if !gap_lower_mm.is_finite() || gap_lower_mm < MIN_AIR_GAP_MM {
        return Err(format!(
            "gap_lower_mm must be finite and at least {MIN_AIR_GAP_MM} mm"
        ));
    }
    if !gap_upper_mm.is_finite() || gap_upper_mm < MIN_AIR_GAP_MM {
        return Err(format!(
            "gap_upper_mm must be finite and at least {MIN_AIR_GAP_MM} mm"
        ));
    }
    if gap_upper_mm + 1e-12 < gap_lower_mm {
        return Err("gap_upper_mm must be greater than or equal to gap_lower_mm".to_owned());
    }
    if !shelf_thickness_mm.is_finite() || shelf_thickness_mm <= 0.0 {
        return Err("shelf_thickness_mm must be finite and positive".to_owned());
    }
    let h_eff = inner_height_mm - top_reserve_mm;
    let mut bottoms: Vec<f64> = Vec::new();
    let mut y = gap_lower_mm;
    if y + shelf_thickness_mm > h_eff - MIN_AIR_GAP_MM {
        return Err(format!(
            "inner usable height ({h_eff} mm) cannot fit a shelf with floor gap {gap_lower_mm} mm, thickness {shelf_thickness_mm} mm, and minimum top air {MIN_AIR_GAP_MM} mm"
        ));
    }
    for _ in 0..MAX_SHELF_BOARDS {
        if y + shelf_thickness_mm > h_eff - MIN_AIR_GAP_MM {
            break;
        }
        bottoms.push(y);
        let shelf_top = y + shelf_thickness_mm;
        let g = if shelf_top < transition_y_mm {
            gap_lower_mm
        } else {
            gap_upper_mm
        };
        let y_next = shelf_top + g;
        if y_next + shelf_thickness_mm > h_eff - MIN_AIR_GAP_MM {
            break;
        }
        y = y_next;
    }
    if bottoms.is_empty() {
        return Err(
            "two-tier rhythm could not place any shelf with the given gaps and reserves".to_owned(),
        );
    }
    Ok(bottoms)
}

/// Computes **maximum** shelf count `n` (≥ 1) such that `band_mm - n·t ≥ (n+1)·m`, i.e. `n` boards fit
/// in band height `band_mm` with shelf thickness `t` and **every** vertical air segment (floor–first
/// shelf, between boards, last shelf–ceiling) at least `m` when gaps are distributed **equally**.
///
/// Returns `0` when no such `n` exists (including when inputs are invalid).
pub fn max_shelf_count_for_min_segment_mm(
    band_mm: f64,
    shelf_thickness_mm: f64,
    min_vertical_segment_mm: f64,
) -> u32 {
    if !band_mm.is_finite()
        || band_mm <= 0.0
        || !shelf_thickness_mm.is_finite()
        || shelf_thickness_mm <= 0.0
        || !min_vertical_segment_mm.is_finite()
        || min_vertical_segment_mm < MIN_AIR_GAP_MM
    {
        return 0;
    }
    let mut n_max: u32 = 0;
    for n in 1..=MAX_SHELF_BOARDS {
        let nf = f64::from(n);
        if band_mm - nf * shelf_thickness_mm >= (nf + 1.0) * min_vertical_segment_mm {
            n_max = n;
        } else {
            break;
        }
    }
    n_max
}

/// Bottom **Y** of each horizontal shelf board (mm), ordered upward, by **minimum vertical segment**
/// packing inside a band `[bottom_reserve_mm, inner_height_mm - top_reserve_mm)`.
///
/// **Algorithm**
/// 1. `band_mm = inner_height_mm - bottom_reserve_mm - top_reserve_mm` must be positive.
/// 2. `n_max` is the largest integer `n ≥ 1` with `band_mm - n·t ≥ (n+1)·m` where `t` is shelf thickness
///    and `m` is `min_vertical_segment_mm` (each of the `n+1` equal air gaps is then ≥ `m`).
/// 3. If `shelf_count` is `None`, use `n = n_max`. If `Some(k)`, require `1 ≤ k ≤ n_max` and use `n = k`
///    (gaps widen: `(band_mm - n·t)/(n+1) ≥ m` still holds).
/// 4. Shelf bottoms match [`equal_spacing_shelf_bottoms_mm`] on `band_mm` with count `n`, shifted by
///    `+ bottom_reserve_mm`.
pub fn max_shelves_min_segment_shelf_bottoms_mm(
    inner_height_mm: f64,
    bottom_reserve_mm: f64,
    top_reserve_mm: f64,
    min_vertical_segment_mm: f64,
    shelf_thickness_mm: f64,
    shelf_count: Option<u32>,
) -> Result<Vec<f64>, String> {
    if !inner_height_mm.is_finite() || inner_height_mm <= 0.0 {
        return Err("inner height must be finite and positive".to_owned());
    }
    if !bottom_reserve_mm.is_finite() || bottom_reserve_mm < 0.0 {
        return Err("bottom_reserve_mm must be finite and non-negative".to_owned());
    }
    if !top_reserve_mm.is_finite() || top_reserve_mm < 0.0 {
        return Err("top_reserve_mm must be finite and non-negative".to_owned());
    }
    if bottom_reserve_mm + top_reserve_mm >= inner_height_mm {
        return Err(format!(
            "bottom_reserve_mm ({bottom_reserve_mm}) + top_reserve_mm ({top_reserve_mm}) must be strictly less than inner height ({inner_height_mm} mm)"
        ));
    }
    if !min_vertical_segment_mm.is_finite() || min_vertical_segment_mm < MIN_AIR_GAP_MM {
        return Err(format!(
            "min_vertical_segment_mm must be finite and at least {MIN_AIR_GAP_MM} mm"
        ));
    }
    if !shelf_thickness_mm.is_finite() || shelf_thickness_mm <= 0.0 {
        return Err("shelf_thickness_mm must be finite and positive".to_owned());
    }
    let band_mm = inner_height_mm - bottom_reserve_mm - top_reserve_mm;
    let n_max = max_shelf_count_for_min_segment_mm(band_mm, shelf_thickness_mm, min_vertical_segment_mm);
    if n_max < 1 {
        return Err(format!(
            "cannot fit any shelf: band height {band_mm} mm with thickness {shelf_thickness_mm} mm requires each of (n+1) air gaps ≥ {min_vertical_segment_mm} mm for some n ≥ 1"
        ));
    }
    let n = match shelf_count {
        None => n_max,
        Some(k) => {
            if k < 1 {
                return Err("shelf_count must be at least 1 when provided".to_owned());
            }
            if k > n_max {
                return Err(format!(
                    "shelf_count {k} exceeds maximum feasible {n_max} for min_vertical_segment_mm {min_vertical_segment_mm} mm in this band"
                ));
            }
            k
        }
    };
    let in_band = equal_spacing_shelf_bottoms_mm(band_mm, n, shelf_thickness_mm)?;
    Ok(in_band
        .into_iter()
        .map(|y| y + bottom_reserve_mm)
        .collect())
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

    #[test]
    fn two_tier_rhythm_monotonic_strictly_increasing_bottoms() {
        let h = 2200.0;
        let t = 18.0;
        let bottoms = two_tier_rhythm_shelf_bottoms_mm(h, 0.0, 900.0, 80.0, 200.0, t).unwrap();
        assert!(bottoms.len() >= 2);
        for i in 0..bottoms.len() - 1 {
            assert!(
                bottoms[i + 1] > bottoms[i] + t - 1e-6,
                "shelf bottoms must clear previous thickness"
            );
        }
    }

    #[test]
    fn two_tier_rhythm_bounds_floor_ceiling_and_gap_regimes() {
        let h = 2200.0;
        let t = 18.0;
        let transition = 900.0;
        let g_lo = 80.0;
        let g_hi = 200.0;
        let bottoms = two_tier_rhythm_shelf_bottoms_mm(h, 0.0, transition, g_lo, g_hi, t).unwrap();
        assert!((bottoms[0] - g_lo).abs() < 1e-6, "first air gap should match gap_lower");
        assert!(bottoms[0] > 0.0);
        let last = *bottoms.last().unwrap();
        assert!(last + t < h - 1e-6);
        let top_air = h - (last + t);
        assert!(top_air >= MIN_AIR_GAP_MM - 1e-6);

        for i in 0..bottoms.len() - 1 {
            let shelf_top = bottoms[i] + t;
            let g = bottoms[i + 1] - shelf_top;
            let expected = if shelf_top < transition { g_lo } else { g_hi };
            assert!(
                (g - expected).abs() < 1e-3,
                "inter-shelf gap {i}: got {g}, expected {expected} (shelf_top={shelf_top})"
            );
        }
    }

    #[test]
    fn two_tier_rhythm_rejects_inverted_gaps() {
        assert!(two_tier_rhythm_shelf_bottoms_mm(2200.0, 0.0, 900.0, 200.0, 80.0, 18.0).is_err());
    }

    #[test]
    fn max_shelf_count_matches_feasible_equal_spacing() {
        let band = 2200.0;
        let t = 18.0;
        let m = 100.0;
        let n_max = max_shelf_count_for_min_segment_mm(band, t, m);
        assert_eq!(n_max, 17);
        let bottoms = max_shelves_min_segment_shelf_bottoms_mm(2200.0, 0.0, 0.0, m, t, None).unwrap();
        assert_eq!(bottoms.len() as u32, n_max);
        let g = (band - f64::from(n_max) * t) / (f64::from(n_max) + 1.0);
        assert!(g + 1e-6 >= m);
    }

    #[test]
    fn max_shelves_min_segment_explicit_count_widens_gaps() {
        let h = 2200.0;
        let t = 18.0;
        let m = 100.0;
        let max_only = max_shelves_min_segment_shelf_bottoms_mm(h, 0.0, 0.0, m, t, None).unwrap();
        let fewer = max_shelves_min_segment_shelf_bottoms_mm(h, 0.0, 0.0, m, t, Some(3)).unwrap();
        assert_eq!(fewer.len(), 3);
        assert!(fewer.len() < max_only.len());
        let g3 = (h - 3.0 * t) / 4.0;
        assert!(g3 > (h - f64::from(max_only.len() as u32) * t) / (f64::from(max_only.len() as u32) + 1.0));
        assert!(g3 + 1e-6 >= m);
    }

    #[test]
    fn max_shelves_min_segment_rejects_impossible_count() {
        let err = max_shelves_min_segment_shelf_bottoms_mm(2200.0, 0.0, 0.0, 100.0, 18.0, Some(99))
            .unwrap_err();
        assert!(err.contains("exceeds maximum"));
    }
}
