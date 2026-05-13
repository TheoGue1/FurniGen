//! 2D panel outline exports (nominal rectangles in mm). Kerf and thickness offsets are out of scope for v1.

use crate::panels::panel_blanks_for_spec;
use crate::WardrobeSpec;

const SVG_NS: &str = "http://www.w3.org/2000/svg";
const MARGIN: f64 = 40.0;
const GAP: f64 = 40.0;

/// Single SVG sheet: each panel as a stroked rectangle, arranged horizontally (cut-list style).
pub fn build_panels_svg(spec: &WardrobeSpec) -> String {
    let panels = panel_blanks_for_spec(spec);
    let mut x = MARGIN;
    let mut max_h = 0.0_f64;
    for p in &panels {
        max_h = max_h.max(p.height_mm);
        x += p.width_mm + GAP;
    }
    let total_w = x - GAP + MARGIN;
    let total_h = max_h + 2.0 * MARGIN;

    let mut body = String::new();
    x = MARGIN;
    for p in &panels {
        use std::fmt::Write;
        writeln!(
            &mut body,
            r##"  <g id="{id}" transform="translate({x},{y})">
    <rect x="0" y="0" width="{w}" height="{h}" fill="none" stroke="#111" stroke-width="2"/>
    <text x="4" y="18" font-size="16" fill="#333">{label}</text>
    <text x="4" y="{ty}" font-size="12" fill="#666">{w:.0} × {h:.0} mm</text>
  </g>"##,
            id = p.id,
            x = x,
            y = MARGIN,
            w = p.width_mm,
            h = p.height_mm,
            label = escape_xml(&p.label),
            ty = p.height_mm - 6.0,
        )
        .expect("fmt");
        x += p.width_mm + GAP;
    }

    format!(
        r#"<svg xmlns="{ns}" width="{tw:.0}" height="{th:.0}" viewBox="0 0 {tw:.0} {th:.0}">
{body}</svg>"#,
        ns = SVG_NS,
        tw = total_w,
        th = total_h,
        body = body,
    )
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Minimal ASCII DXF (R12-style): one closed rectangle of LINE entities per panel, separated in X.
pub fn build_panels_dxf(spec: &WardrobeSpec) -> String {
    let panels = panel_blanks_for_spec(spec);
    let mut out = String::from(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1014\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n",
    );
    let mut origin_x = 0.0_f64;
    for p in &panels {
        let w = p.width_mm;
        let h = p.height_mm;
        let ox = origin_x;
        let oy = 0.0;
        let layer = p.id.as_str();
        // CCW rectangle in XY (shop drawing plane)
        line_dxf(&mut out, layer, (ox, oy, 0.0), (ox + w, oy, 0.0));
        line_dxf(&mut out, layer, (ox + w, oy, 0.0), (ox + w, oy + h, 0.0));
        line_dxf(&mut out, layer, (ox + w, oy + h, 0.0), (ox, oy + h, 0.0));
        line_dxf(&mut out, layer, (ox, oy + h, 0.0), (ox, oy, 0.0));
        origin_x += w + GAP;
    }
    out.push_str("0\nENDSEC\n0\nEOF\n");
    out
}

fn line_dxf(out: &mut String, layer: &str, a: (f64, f64, f64), b: (f64, f64, f64)) {
    use std::fmt::Write;
    writeln!(
        out,
        "0\nLINE\n8\n{layer}\n10\n{}\n20\n{}\n30\n{}\n11\n{}\n21\n{}\n31\n{}",
        a.0, a.1, a.2, b.0, b.1, b.2
    )
    .expect("fmt");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_wardrobe_spec_json;

    #[test]
    fn svg_contains_panel_groups() {
        const FIXTURE: &str = include_str!("../../spec-fixtures/wardrobe-spec-v1-minimal.json");
        let spec = parse_wardrobe_spec_json(FIXTURE.trim()).unwrap();
        let svg = build_panels_svg(&spec);
        assert!(svg.contains(&format!("xmlns=\"{SVG_NS}\"")));
        assert!(svg.contains("id=\"back\""));
        assert!(svg.contains("id=\"left_side\""));
    }

    #[test]
    fn svg_includes_shelf_rectangles_when_equal_spacing_interior() {
        const FIXTURE: &str =
            include_str!("../../spec-fixtures/wardrobe-spec-v1-equal-spacing-shelves.json");
        let spec = parse_wardrobe_spec_json(FIXTURE.trim()).unwrap();
        let svg = build_panels_svg(&spec);
        assert!(svg.contains("id=\"shelf_001\""));
        assert!(svg.contains("id=\"shelf_004\""));
    }

    #[test]
    fn svg_includes_shelf_rectangles_when_explicit_interior() {
        const FIXTURE: &str =
            include_str!("../../spec-fixtures/wardrobe-spec-v1-explicit-shelf-heights.json");
        let spec = parse_wardrobe_spec_json(FIXTURE.trim()).unwrap();
        let svg = build_panels_svg(&spec);
        assert!(svg.contains("id=\"shelf_001\""));
        assert!(svg.contains("id=\"shelf_003\""));
    }

    #[test]
    fn svg_includes_shelf_rectangles_when_zones_equal_fill_interior() {
        const FIXTURE: &str =
            include_str!("../../spec-fixtures/wardrobe-spec-v1-zones-equal-fill-shelves.json");
        let spec = parse_wardrobe_spec_json(FIXTURE.trim()).unwrap();
        let svg = build_panels_svg(&spec);
        assert!(svg.contains("id=\"shelf_001\""));
        assert!(svg.contains("id=\"shelf_003\""));
    }

    #[test]
    fn svg_includes_shelf_rectangles_when_golden_ratio_ladder_interior() {
        const FIXTURE: &str =
            include_str!("../../spec-fixtures/wardrobe-spec-v1-golden-ratio-ladder-shelves.json");
        let spec = parse_wardrobe_spec_json(FIXTURE.trim()).unwrap();
        let svg = build_panels_svg(&spec);
        assert!(svg.contains("id=\"shelf_001\""));
        assert!(svg.contains("id=\"shelf_003\""));
    }

    #[test]
    fn dxf_has_entities_and_eof() {
        const FIXTURE: &str = include_str!("../../spec-fixtures/wardrobe-spec-v1-minimal.json");
        let spec = parse_wardrobe_spec_json(FIXTURE.trim()).unwrap();
        let dxf = build_panels_dxf(&spec);
        assert!(dxf.contains("ENTITIES"));
        assert!(dxf.contains("LINE"));
        assert!(dxf.ends_with("0\nEOF\n"));
    }
}
