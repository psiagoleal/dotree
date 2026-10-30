// Caminho relativo: app/wasm/src/lib.rs
//! Bridge between `dotree-core` and the browser. No domain rules live here (ADR 0001).

use wasm_bindgen::prelude::*;

/// Core version, used by the interface to prove the WASM module loaded.
#[wasm_bindgen(js_name = coreVersion)]
pub fn core_version() -> String {
    dotree_core::version().to_owned()
}
