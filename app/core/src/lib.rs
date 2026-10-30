// Caminho relativo: app/core/src/lib.rs
//! Domain core of dotree.
//!
//! Pure Rust, with no browser or WebAssembly dependency (ADR 0001). The domain modules
//! (expression, dot matrix, cells, verification, VHDL) arrive with their tickets; this
//! skeleton only proves the build chain end to end.

/// Version of the core crate, shown by the interface to prove the WASM module loaded.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_matches_manifest() {
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
        assert!(!version().is_empty());
    }
}
