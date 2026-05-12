//! Bill of materials / cut list derived from panel blanks.

use serde::Serialize;

use crate::panels::panel_blanks_for_spec;
use crate::WardrobeSpec;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BomDocument {
    pub unit: &'static str,
    pub parts: Vec<BomPartRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BomPartRow {
    pub id: String,
    pub label: String,
    pub width_mm: f64,
    pub height_mm: f64,
    pub qty: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thickness_mm: Option<f64>,
}

/// Builds a BOM document (nominal cut rectangles; thickness not in spec v1).
pub fn build_bom(spec: &WardrobeSpec) -> BomDocument {
    let parts = panel_blanks_for_spec(spec)
        .into_iter()
        .map(|p| BomPartRow {
            id: p.id,
            label: p.label,
            width_mm: p.width_mm,
            height_mm: p.height_mm,
            qty: 1,
            thickness_mm: None,
        })
        .collect();
    BomDocument { unit: "mm", parts }
}

/// CSV with header row (`thickness_mm` empty when unknown).
pub fn bom_to_csv(doc: &BomDocument) -> String {
    use std::fmt::Write;
    let mut s = String::from("id,label,width_mm,height_mm,qty,thickness_mm\n");
    for p in &doc.parts {
        let t = p.thickness_mm.map(|v| v.to_string()).unwrap_or_default();
        writeln!(
            &mut s,
            "{},{},{},{},{},{}",
            p.id, p.label, p.width_mm, p.height_mm, p.qty, t
        )
        .expect("fmt to String");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_wardrobe_spec_json;

    #[test]
    fn straight_run_bom_has_five_parts() {
        const FIXTURE: &str = include_str!("../../spec-fixtures/wardrobe-spec-v1-minimal.json");
        let spec = parse_wardrobe_spec_json(FIXTURE.trim()).unwrap();
        let bom = build_bom(&spec);
        assert_eq!(bom.parts.len(), 5);
        assert_eq!(bom.unit, "mm");
        let back = bom.parts.iter().find(|p| p.id == "back").unwrap();
        assert!((back.width_mm - 2400.0).abs() < 1e-9);
        assert!((back.height_mm - 2200.0).abs() < 1e-9);
        assert!(back.thickness_mm.is_none());
    }

    #[test]
    fn bom_csv_contains_header_and_back_row() {
        const FIXTURE: &str = include_str!("../../spec-fixtures/wardrobe-spec-v1-minimal.json");
        let spec = parse_wardrobe_spec_json(FIXTURE.trim()).unwrap();
        let csv = bom_to_csv(&build_bom(&spec));
        assert!(csv.starts_with("id,label,width_mm,height_mm,qty,thickness_mm\n"));
        assert!(csv.contains("back,Back,2400,2200,1,"));
    }

    #[test]
    fn bom_includes_shelf_rows_when_interior_explicit_heights() {
        let json = r#"{"version":1,"layout":{"type":"straight_run","width_mm":2400.0,"height_mm":2200.0,"depth_mm":600.0},"interior":{"type":"explicit_shelf_heights","shelf_bottom_y_mm":[400.0,1100.0]}}"#;
        let spec = parse_wardrobe_spec_json(json).unwrap();
        let bom = build_bom(&spec);
        assert_eq!(bom.parts.len(), 7);
        assert!(bom.parts.iter().any(|p| p.id == "shelf_01"));
        assert!(bom.parts.iter().any(|p| p.id == "shelf_02"));
    }

    #[test]
    fn bom_includes_shelf_rows_when_interior_equal_spacing() {
        let json = r#"{"version":1,"layout":{"type":"straight_run","width_mm":2400.0,"height_mm":2200.0,"depth_mm":600.0},"interior":{"type":"equal_spacing_shelves","shelf_count":2}}"#;
        let spec = parse_wardrobe_spec_json(json).unwrap();
        let bom = build_bom(&spec);
        assert_eq!(bom.parts.len(), 7);
        assert!(bom.parts.iter().any(|p| p.id == "shelf_01"));
        assert!(bom.parts.iter().any(|p| p.id == "shelf_02"));
    }
}
