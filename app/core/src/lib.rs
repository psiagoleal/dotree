// Caminho relativo: app/core/src/lib.rs
//! Domain core of dotree.
//!
//! Pure Rust, with no browser or WebAssembly dependency (ADR 0001). Modules arrive with
//! their tickets (docs/plano.md): `expr` (MT-3) is the first.

pub mod expr;

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
