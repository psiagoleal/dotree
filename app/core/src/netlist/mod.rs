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
use crate::reduce::{CellKind, Costs, Origin, Reduction, Time};

/// A wire of the circuit. Its identifier is the one used by every artifact (spec 05).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Wire {
    /// Input signal, index into [`Netlist::signals`] (`e0`, `a1_b0`).
    Input(usize),
    /// Sum output of cell `n` (`s<n>`).
    Sum(usize),
    /// Carry output of cell `n` (`k<n>`).
    Carry(usize),
    /// Output of weight `2^bit` of counter `n` (`k<n>_<bit>`, ADR 0005).
    Upper(usize, u8),
}

impl Wire {
    /// The cell that drives this wire, `None` for an input.
    pub fn cell(self) -> Option<usize> {
        match self {
            Wire::Input(_) => None,
            Wire::Sum(n) | Wire::Carry(n) | Wire::Upper(n, _) => Some(n),
        }
    }
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
    /// Outputs of weight 2 and 3 of a counter that fall below `W` (ADR 0005).
    pub upper: u8,
}

impl NetCell {
    /// Output wires, bit 0 first: sum, carry and a counter's bits 2 and 3, without the
    /// dropped ones.
    pub fn outputs(&self) -> Vec<Wire> {
        let mut out = vec![Wire::Sum(self.number)];
        if self.has_carry {
            out.push(Wire::Carry(self.number));
        }
        out.extend((0..self.upper).map(|i| Wire::Upper(self.number, i + 2)));
        out
    }
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
    /// The costs of the reduction it came from: readiness, delay and area use them.
    pub costs: Costs,
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
                Origin::Upper { cell, bit } => Wire::Upper(cell, bit),
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
                upper: c.upper.len() as u8,
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
            costs: reduction.costs(),
        }
    }

    /// Identifier of a wire: `a1_b0`, `e0`, `s3`, `k3`, `k3_2`.
    pub fn ident(&self, wire: Wire) -> String {
        match wire {
            Wire::Input(i) => self.signals[i].ident(),
            Wire::Sum(n) => format!("s{n}"),
            Wire::Carry(n) => format!("k{n}"),
            Wire::Upper(n, bit) => format!("k{n}_{bit}"),
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
                .map(|&w| w.cell().map_or(0, |p| ready[p].0))
                .max()
                .unwrap_or(0);
            ready[c.number] = Time(latest + self.costs.delay(c.kind).0);
        }
        ready
    }

    /// Delay of the circuit: the latest cell output (0 without cells).
    pub fn delay(&self) -> Time {
        self.ready().into_iter().max().unwrap_or_default()
    }

    /// Area of the cells in hundredths of `A_FA`.
    pub fn area(&self) -> u32 {
        self.cells.iter().map(|c| self.costs.area(c.kind)).sum()
    }

    /// Number of cells of `kind`.
    pub fn count(&self, kind: CellKind) -> usize {
        self.cells.iter().filter(|c| c.kind == kind).count()
    }

    /// Number of FA and HA cells.
    pub fn cell_counts(&self) -> (usize, usize) {
        (self.count(CellKind::Fa), self.count(CellKind::Ha))
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
