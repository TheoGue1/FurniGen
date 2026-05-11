//! Thin boundary crate for the browser. Delegate to [`furnigen_core`]; add `wasm-bindgen` exports in a later milestone.

pub use furnigen_core::validate_depth_mm;
pub use furnigen_core::ValidationError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reexports_core_validation() {
        assert!(validate_depth_mm(500.0).is_ok());
    }
}
