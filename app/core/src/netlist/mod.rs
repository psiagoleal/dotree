// Caminho relativo: app/core/src/netlist/mod.rs
//! The finished circuit as wires and cells (MT-7).
//!
//! A complete [`Reduction`] becomes a [`Netlist`]: input wires (single bits or partial
//! products), one entry per cell in number order, and the wire behind each result bit. The
//! verifier simulates it; the VHDL generator (MT-10) and the diagram (MT-9) read it. Unlike a
//! reduction, a netlist can be edited, which is how the tests build a broken circuit (VER-03).

use std::collections::BTreeMap;

use crate::expr::Monomial;
use crate::matrix::{DotMatrix, Signal};
use crate::reduce::{CellKind, Origin, Reduction};

/// A wire of the circuit. Its identifier is the one used by every artifact (spec 05).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Wire {
    /// Input signal, index into [`Netlist::signals`] (`e0`, `a1_b0`).
    Input(usize),
    /// Sum output of cell `n` (`s<n>`).
    Sum(usize),
    /// Carry output of cell `n` (`k<n>`).
    Carry(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetCell {
    pub number: usize,
    pub kind: CellKind,
    pub column: u32,
    /// Inputs in canonical order (x, y, z).
    pub inputs: Vec<Wire>,
    /// False when the carry fell at or beyond `W` and was dropped (CMP-09).
    pub has_carry: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Netlist {
    /// Input variables and widths, in packing order: alphabetical, first variable in the
    /// lowest bits of a packed input vector.
    pub variables: Vec<(char, u32)>,
    /// The expression, to compute the expected value.
    pub terms: Vec<Monomial>,
    pub signals: Vec<Signal>,
    /// Cells in number order: `cells[n - 1]` is cell `n`. Every input comes from a lower
    /// number or from an input signal, so this is also a valid evaluation order.
    pub cells: Vec<NetCell>,
    /// Wire behind each result bit `R_0 .. R_{W-1}`; `None` is the constant 0.
    pub result: Vec<Option<Wire>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetlistError {
    /// The circuit is not complete yet (VER-05).
    NotComplete,
}

impl Netlist {
    /// Builds the netlist of a complete reduction of `matrix`.
    pub fn from_reduction(
        matrix: &DotMatrix,
        reduction: &Reduction,
    ) -> Result<Netlist, NetlistError> {
        let result_points = reduction.result().ok_or(NetlistError::NotComplete)?;
        let wire_of = |id: usize| -> Wire {
            let p = reduction
                .point(id)
                .expect("every point id comes from this reduction");
            match p.origin {
                Origin::Input { dot } => Wire::Input(matrix.dots[dot].signal),
                Origin::Sum { cell } => Wire::Sum(cell),
                Origin::Carry { cell } => Wire::Carry(cell),
            }
        };
        let mut cells: Vec<NetCell> = reduction
            .levels()
            .iter()
            .flat_map(|l| &l.cells)
            .map(|c| NetCell {
                number: c.number,
                kind: c.kind,
                column: c.column,
                inputs: c.inputs.iter().map(|&id| wire_of(id)).collect(),
                has_carry: c.carry.is_some(),
            })
            .collect();
        cells.sort_by_key(|c| c.number);
        Ok(Netlist {
            variables: matrix.widths.iter().map(|(&v, &w)| (v, w)).collect(),
            terms: matrix.terms.clone(),
            signals: matrix.signals.clone(),
            cells,
            result: result_points
                .into_iter()
                .map(|id| id.map(wire_of))
                .collect(),
        })
    }

    /// Identifier of a wire: `a1_b0`, `e0`, `s3`, `k3`.
    pub fn ident(&self, wire: Wire) -> String {
        match wire {
            Wire::Input(i) => self.signals[i].ident(),
            Wire::Sum(n) => format!("s{n}"),
            Wire::Carry(n) => format!("k{n}"),
        }
    }

    /// Total input bits, i.e. the width of a packed input vector.
    pub fn input_bits(&self) -> u32 {
        self.variables.iter().map(|&(_, w)| w).sum()
    }

    /// Bit offset of each variable in a packed input vector.
    pub fn offsets(&self) -> BTreeMap<char, u32> {
        let mut at = 0;
        self.variables
            .iter()
            .map(|&(v, w)| {
                let o = (v, at);
                at += w;
                o
            })
            .collect()
    }

    /// Value of each variable in a packed input vector.
    pub fn unpack(&self, packed: u64) -> BTreeMap<char, u64> {
        let offsets = self.offsets();
        self.variables
            .iter()
            .map(|&(v, w)| (v, (packed >> offsets[&v]) & ((1u64 << w) - 1)))
            .collect()
    }

    /// Value of the expression for a packed input vector.
    pub fn expected(&self, packed: u64) -> u128 {
        let v = self.unpack(packed);
        self.terms
            .iter()
            .map(|t| {
                t.vars
                    .iter()
                    .fold(u128::from(t.coefficient), |acc, x| acc * u128::from(v[x]))
            })
            .sum()
    }
}
