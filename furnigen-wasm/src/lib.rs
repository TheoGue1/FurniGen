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
}
