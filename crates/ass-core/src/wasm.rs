//! WASM bindings for ass-core.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn normalize_ass(input: &str) -> String {
    let script = crate::Script::parse(input.as_bytes());
    script.serialize()
}
