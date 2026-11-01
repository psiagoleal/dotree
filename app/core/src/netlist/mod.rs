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
use crate::reduce::{CellKind, Origin, Reduction, Time};

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
    /// Index of the level whose points the cell consumed.
    pub level: usize,
    /// True for cells of the final ripple-carry adder.
    pub cpa: bool,
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
    /// Wire behind each result bit `R_0 .. R_{W-1}`; `None` is the constant 0. Empty while
    /// the circuit is incomplete.
    pub result: Vec<Option<Wire>>,
    /// Output width `W`.
    pub output_width: u32,
    /// Every column ends with at most one point (CMP-12).
    pub complete: bool,
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
        if !reduction.is_complete() {
            return Err(NetlistError::NotComplete);
        }
        Ok(Netlist::partial(matrix, reduction))
    }

    /// The concluded part of a reduction, complete or not. Pending cells are left out; the
    /// result is empty until the circuit is complete (DIA-07).
    pub fn partial(matrix: &DotMatrix, reduction: &Reduction) -> Netlist {
        let result_points = reduction.result().unwrap_or_default();
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
            .flat_map(|l| l.cells.iter().map(move |c| (c, l.cpa)))
            .map(|(c, cpa)| NetCell {
                number: c.number,
                kind: c.kind,
                level: c.level,
                cpa,
                column: c.column,
                inputs: c.inputs.iter().map(|&id| wire_of(id)).collect(),
                has_carry: c.carry.is_some(),
            })
            .collect();
        cells.sort_by_key(|c| c.number);
        Netlist {
            variables: matrix.widths.iter().map(|(&v, &w)| (v, w)).collect(),
            terms: matrix.terms.clone(),
            signals: matrix.signals.clone(),
            cells,
            result: result_points
                .into_iter()
                .map(|id| id.map(wire_of))
                .collect(),
            output_width: matrix.output_width,
            complete: reduction.is_complete(),
        }
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
    /// Readiness of each cell's outputs, indexed by cell number (index 0 unused).
    pub fn ready(&self) -> Vec<Time> {
        let mut ready = vec![Time(0); self.cells.len() + 1];
        for c in &self.cells {
            let latest = c
                .inputs
                .iter()
                .map(|&w| match w {
                    Wire::Input(_) => 0,
                    Wire::Sum(p) | Wire::Carry(p) => ready[p].0,
                })
                .max()
                .unwrap_or(0);
            ready[c.number] = Time(latest + cell_delay(c.kind).0);
        }
        ready
    }

    /// Delay of the circuit: the latest cell output (0 without cells).
    pub fn delay(&self) -> Time {
        self.ready().into_iter().max().unwrap_or_default()
    }

    /// Number of FA and HA cells.
    pub fn cell_counts(&self) -> (usize, usize) {
        let fa = self.cells.iter().filter(|c| c.kind == CellKind::Fa).count();
        (fa, self.cells.len() - fa)
    }

    /// Number of AND gates: input signals that are partial products.
    pub fn and_gates(&self) -> usize {
        self.signals.iter().filter(|s| s.is_and()).count()
    }

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

/// Delay of one cell kind (spec 00, "Convenção de custo").
pub fn cell_delay(kind: CellKind) -> Time {
    match kind {
        CellKind::Fa => Time::FA_DELAY,
        CellKind::Ha => Time::HA_DELAY,
    }
}
