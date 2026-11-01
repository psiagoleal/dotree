// Caminho relativo: app/core/src/lib.rs
//! Domain core of dotree.
//!
//! Pure Rust, with no browser or WebAssembly dependency (ADR 0001). Modules arrive with
//! their tickets (docs/plano.md): `expr` (MT-3), `matrix` (MT-4), `reduce` (MT-5, MT-6),
//! `netlist` and `verify` (MT-7), `help` (MT-8), `diagram` (MT-9), `vhdl` (MT-10).

pub mod diagram;
pub mod expr;
pub mod help;
pub mod matrix;
pub mod netlist;
pub mod reduce;
pub mod verify;
pub mod vhdl;

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
