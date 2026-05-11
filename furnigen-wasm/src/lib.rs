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
}
