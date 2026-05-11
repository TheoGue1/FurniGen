//! Thin boundary crate for the browser. Delegate to [`furnigen_core`]; keep JSON and
//! heavy work behind small entrypoints as the API grows.

use wasm_bindgen::prelude::*;

#[cfg(all(feature = "debug-panic-hook", target_arch = "wasm32"))]
#[wasm_bindgen(start)]
pub fn wasm_start() {
    console_error_panic_hook::set_once();
}

/// Returns the `furnigen-wasm` crate version (useful for cache-busting and diagnostics).
#[wasm_bindgen(js_name = wasmVersion)]
pub fn wasm_version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

/// Validates a straight-run carcass depth in millimeters (positive, finite).
#[wasm_bindgen(js_name = validateDepthMm)]
pub fn validate_depth_mm_js(depth_mm: f64) -> Result<(), JsValue> {
    furnigen_core::validate_depth_mm(depth_mm).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Parses and validates a [`WardrobeSpec`](furnigen_core::WardrobeSpec) JSON document (mm units).
#[wasm_bindgen(js_name = validateWardrobeSpecJson)]
pub fn validate_wardrobe_spec_json(json: &str) -> Result<(), JsValue> {
    furnigen_core::parse_wardrobe_spec_json(json)
        .map(drop)
        .map_err(js_spec_err)
}

/// Canonical JSON serialization after parse + validate (stable ordering of object keys is not guaranteed).
#[wasm_bindgen(js_name = normalizeWardrobeSpecJson)]
pub fn normalize_wardrobe_spec_json(json: &str) -> Result<String, JsValue> {
    let spec = furnigen_core::parse_wardrobe_spec_json(json).map_err(js_spec_err)?;
    serde_json::to_string(&spec).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Builds a preview triangle mesh (mm units) from a validated WardrobeSpec JSON document.
#[wasm_bindgen(js_name = buildWardrobePreviewMeshJson)]
pub fn build_wardrobe_preview_mesh_json(json: &str) -> Result<String, JsValue> {
    let spec = furnigen_core::parse_wardrobe_spec_json(json).map_err(js_spec_err)?;
    let mesh = furnigen_core::build_preview_mesh(&spec);
    serde_json::to_string(&mesh).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Cut list / BOM as JSON (`unit`, `parts[]` with nominal `width_mm` × `height_mm` per panel).
#[wasm_bindgen(js_name = buildWardrobeBomJson)]
pub fn build_wardrobe_bom_json(json: &str) -> Result<String, JsValue> {
    let spec = furnigen_core::parse_wardrobe_spec_json(json).map_err(js_spec_err)?;
    let bom = furnigen_core::build_bom(&spec);
    serde_json::to_string(&bom).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Same BOM as [`buildWardrobeBomJson`](crate::build_wardrobe_bom_json), CSV with header row.
#[wasm_bindgen(js_name = buildWardrobeBomCsv)]
pub fn build_wardrobe_bom_csv(json: &str) -> Result<String, JsValue> {
    let spec = furnigen_core::parse_wardrobe_spec_json(json).map_err(js_spec_err)?;
    Ok(furnigen_core::bom_to_csv(&furnigen_core::build_bom(&spec)))
}

/// 2D panel outlines as a single SVG document (mm geometry).
#[wasm_bindgen(js_name = buildWardrobePanelsSvg)]
pub fn build_wardrobe_panels_svg(json: &str) -> Result<String, JsValue> {
    let spec = furnigen_core::parse_wardrobe_spec_json(json).map_err(js_spec_err)?;
    Ok(furnigen_core::build_panels_svg(&spec))
}

/// 2D panel rectangles as minimal ASCII DXF (mm in drawing XY).
#[wasm_bindgen(js_name = buildWardrobePanelsDxf)]
pub fn build_wardrobe_panels_dxf(json: &str) -> Result<String, JsValue> {
    let spec = furnigen_core::parse_wardrobe_spec_json(json).map_err(js_spec_err)?;
    Ok(furnigen_core::build_panels_dxf(&spec))
}

/// Preview mesh as Wavefront OBJ (mm, triangular faces).
#[wasm_bindgen(js_name = buildWardrobePreviewObj)]
pub fn build_wardrobe_preview_obj(json: &str) -> Result<String, JsValue> {
    let spec = furnigen_core::parse_wardrobe_spec_json(json).map_err(js_spec_err)?;
    let mesh = furnigen_core::build_preview_mesh(&spec);
    furnigen_core::preview_mesh_to_obj(&mesh, "wardrobe")
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Preview mesh as glTF 2.0 JSON with an embedded `data:` buffer (mm).
#[wasm_bindgen(js_name = buildWardrobePreviewGltf)]
pub fn build_wardrobe_preview_gltf(json: &str) -> Result<String, JsValue> {
    let spec = furnigen_core::parse_wardrobe_spec_json(json).map_err(js_spec_err)?;
    let mesh = furnigen_core::build_preview_mesh(&spec);
    furnigen_core::preview_mesh_to_gltf(&mesh).map_err(|e| JsValue::from_str(&e.to_string()))
}

fn js_spec_err(e: furnigen_core::SpecError) -> JsValue {
    JsValue::from_str(&e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_depth_ok() {
        validate_depth_mm_js(500.0).unwrap();
    }

    #[test]
    fn validate_depth_err_matches_core() {
        assert!(furnigen_core::validate_depth_mm(0.0).is_err());
    }

    #[test]
    fn wardrobe_spec_json_round_trip() {
        const FIXTURE: &str = include_str!("../../spec-fixtures/wardrobe-spec-v1-minimal.json");
        validate_wardrobe_spec_json(FIXTURE.trim()).unwrap();
        let out = normalize_wardrobe_spec_json(FIXTURE.trim()).unwrap();
        assert!(out.contains("\"version\":1"));
        assert!(out.contains("straight_run"));
    }

    #[test]
    fn preview_mesh_json_shape() {
        const FIXTURE: &str = include_str!("../../spec-fixtures/wardrobe-spec-v1-minimal.json");
        let json = build_wardrobe_preview_mesh_json(FIXTURE.trim()).unwrap();
        assert!(json.contains("\"positions\""));
        assert!(json.contains("\"indices\""));
    }

    #[test]
    fn export_entrypoints_smoke() {
        const FIXTURE: &str = include_str!("../../spec-fixtures/wardrobe-spec-v1-minimal.json");
        let j = FIXTURE.trim();
        assert!(build_wardrobe_bom_json(j).unwrap().contains("\"parts\""));
        assert!(build_wardrobe_bom_csv(j).unwrap().contains("left_side"));
        assert!(build_wardrobe_panels_svg(j).unwrap().contains("<svg"));
        assert!(build_wardrobe_panels_dxf(j).unwrap().contains("EOF"));
        assert!(build_wardrobe_preview_obj(j)
            .unwrap()
            .starts_with("# FurniGen"));
        let gltf = build_wardrobe_preview_gltf(j).unwrap();
        assert!(gltf.contains("\"asset\"") && gltf.contains("base64,"));
    }
}
