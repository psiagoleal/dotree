// Caminho relativo: app/core/src/diagram/mod.rs
//! Block diagram layout and wiring table (spec 05; MT-9).
//!
//! The layout is a grid, not pixels: one row per level (inputs and AND gates on top, then
//! the cells formed over level 0, 1, …, then the CPA) and one column per weight. The
//! interface turns it into SVG. No natural-language text here: table headers come from the
//! interface (IDI-05).

use std::collections::{BTreeSet, HashMap};

use crate::netlist::{Netlist, Wire};
use crate::reduce::CellKind;

/// A cell drawn as a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block {
    /// Cell number.
    pub cell: usize,
    pub kind: CellKind,
    /// 0 is the input and AND block; cells over level `n` sit in row `n + 1`.
    pub row: usize,
    /// Weight `2^column`; drawn with the highest weight on the left.
    pub column: u32,
    /// Position among the blocks sharing this row and column, by cell number.
    pub slot: usize,
}

/// One end of a connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Endpoint {
    /// The input and AND block at the top.
    Inputs,
    Cell(usize),
    /// Result bit `R_j`.
    Output(u32),
}

/// A labeled connection (DIA-03).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    pub from: Endpoint,
    pub to: Endpoint,
    /// Identifier of the wire: `a1_b0`, `s3`, `k3`.
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Layout {
    pub output_width: u32,
    /// Number of rows, the input block included.
    pub rows: usize,
    pub and_gates: usize,
    /// Blocks in cell-number order.
    pub blocks: Vec<Block>,
    pub edges: Vec<Edge>,
    /// Wire identifier behind each result bit, `None` for a constant 0. Empty while the
    /// circuit is incomplete (DIA-07).
    pub outputs: Vec<Option<String>>,
    /// Cells on some path of maximum delay, ascending (DIA-05).
    pub critical: Vec<usize>,
    pub complete: bool,
}

impl Layout {
    /// Lays out a (possibly partial) netlist (DIA-01 to DIA-03, DIA-05, DIA-07).
    pub fn new(netlist: &Netlist) -> Layout {
        let n = netlist;
        let source = |w: Wire| w.cell().map_or(Endpoint::Inputs, Endpoint::Cell);

        let mut blocks: Vec<Block> = Vec::with_capacity(n.cells.len());
        let mut edges = Vec::new();
        for c in &n.cells {
            let row = c.level + 1;
            let slot = blocks
                .iter()
                .filter(|b| b.row == row && b.column == c.column)
                .count();
            blocks.push(Block {
                cell: c.number,
                kind: c.kind,
                row,
                column: c.column,
                slot,
            });
            edges.extend(c.inputs.iter().map(|&w| Edge {
                from: source(w),
                to: Endpoint::Cell(c.number),
                label: n.ident(w),
            }));
        }
        for (j, w) in n.result.iter().enumerate() {
            if let Some(w) = *w {
                edges.push(Edge {
                    from: source(w),
                    to: Endpoint::Output(j as u32),
                    label: n.ident(w),
                });
            }
        }

        Layout {
            output_width: n.output_width,
            rows: 1 + blocks.iter().map(|b| b.row).max().unwrap_or(0),
            and_gates: n.signals.iter().filter(|s| s.is_and()).count(),
            blocks,
            edges,
            outputs: n.result.iter().map(|w| w.map(|w| n.ident(w))).collect(),
            critical: critical_cells(n),
            complete: n.complete,
        }
    }

    /// Every label in the diagram plus the result names `R(j)` (DIA-03).
    pub fn identifiers(&self) -> BTreeSet<String> {
        let mut ids: BTreeSet<String> = self.edges.iter().map(|e| e.label.clone()).collect();
        if self.complete {
            ids.extend((0..self.output_width).map(|j| format!("R({j})")));
        }
        ids
    }
}

/// One row of the wiring table (DIA-04).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WiringRow {
    /// `FA1`, `HA3`, `C73_4`.
    pub cell: String,
    /// Level the cell consumed; `None` for the CPA.
    pub level: Option<usize>,
    pub column: u32,
    /// x, y, z (`z` is `None` for an HA), or the 7 or 15 inputs of a counter, in canonical
    /// order.
    pub inputs: Vec<Option<String>>,
    pub sum: String,
    /// Result bit the sum drives, if any.
    pub sum_result: Option<u32>,
    /// `None` when the carry was dropped (CMP-09).
    pub carry: Option<String>,
    pub carry_result: Option<u32>,
    /// Bits 2 and 3 of a counter that were not dropped; empty for FA and HA.
    pub upper: Vec<Output>,
}

/// An output wire and the result bit it drives, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub name: String,
    pub result: Option<u32>,
}

/// Rows of the wiring table, in cell-number order.
pub fn wiring_table(netlist: &Netlist) -> Vec<WiringRow> {
    let n = netlist;
    let result_of: HashMap<Wire, u32> = n
        .result
        .iter()
        .enumerate()
        .filter_map(|(j, w)| w.map(|w| (w, j as u32)))
        .collect();
    n.cells
        .iter()
        .map(|c| {
            let input = |i: usize| c.inputs.get(i).map(|&w| n.ident(w));
            let carry = c.has_carry.then_some(Wire::Carry(c.number));
            let upper = c
                .outputs()
                .into_iter()
                .filter(|w| matches!(w, Wire::Upper(..)))
                .map(|w| Output {
                    name: n.ident(w),
                    result: result_of.get(&w).copied(),
                })
                .collect();
            WiringRow {
                cell: c.kind.instance(c.number),
                level: (!c.cpa).then_some(c.level),
                column: c.column,
                inputs: (0..c.inputs.len().max(3)).map(input).collect(),
                sum: n.ident(Wire::Sum(c.number)),
                sum_result: result_of.get(&Wire::Sum(c.number)).copied(),
                carry: carry.map(|w| n.ident(w)),
                carry_result: carry.and_then(|w| result_of.get(&w).copied()),
                upper,
            }
        })
        .collect()
}

/// Cells on a path of maximum delay: walk back from the latest cells through inputs that are
/// ready exactly one cell delay earlier.
fn critical_cells(n: &Netlist) -> Vec<usize> {
    let ready: Vec<u32> = n.ready().into_iter().map(|t| t.0).collect();
    let delay = ready.iter().copied().max().unwrap_or(0);
    if delay == 0 {
        return Vec::new();
    }
    let mut critical = BTreeSet::new();
    let mut stack: Vec<usize> = n
        .cells
        .iter()
        .filter(|c| ready[c.number] == delay)
        .map(|c| c.number)
        .collect();
    while let Some(number) = stack.pop() {
        if !critical.insert(number) {
            continue;
        }
        let c = &n.cells[number - 1];
        let want = ready[number] - n.costs.delay(c.kind).0;
        for &w in &c.inputs {
            if let Some(p) = w.cell().filter(|&p| ready[p] == want) {
                stack.push(p);
            }
        }
    }
    critical.into_iter().collect()
}

/// Column titles, translated by the interface.
#[derive(Debug, Clone, Copy)]
pub struct TableHeaders<'a> {
    pub cell: &'a str,
    pub level: &'a str,
    pub column: &'a str,
    pub sum: &'a str,
    pub carry: &'a str,
    /// Used only when the table has a counter: one column with every input, and one with
    /// the bits 2 and 3.
    pub inputs: &'a str,
    pub upper: &'a str,
}

/// The wiring table as Markdown (DIA-06), in the format of spec 05. A table with a counter
/// lists all the inputs of each row in one column, and the bits 2 and 3 in another.
pub fn wiring_table_markdown(rows: &[WiringRow], headers: TableHeaders) -> String {
    let code = |s: &str| format!("`{s}`");
    let driving = |name: &str, result: Option<u32>| match result {
        Some(j) => format!("{} → `R({j})`", code(name)),
        None => code(name),
    };
    let wide = rows.iter().any(|r| r.inputs.len() > 3);
    let h = headers;
    let mut out = if wide {
        format!(
            "| {} | {} | {} | {} | {} | {} | {} |\n|---|---|---|---|---|---|---|\n",
            h.cell, h.level, h.column, h.inputs, h.sum, h.carry, h.upper
        )
    } else {
        format!(
            "| {} | {} | {} | x | y | z | {} | {} |\n|---|---|---|---|---|---|---|---|\n",
            h.cell, h.level, h.column, h.sum, h.carry
        )
    };
    for r in rows {
        let level = r.level.map_or("CPA".to_string(), |l| l.to_string());
        let carry = r
            .carry
            .as_deref()
            .map_or("—".to_string(), |k| driving(k, r.carry_result));
        let inputs: Vec<String> = if wide {
            let all: Vec<String> = r.inputs.iter().flatten().map(|s| code(s)).collect();
            vec![all.join(", ")]
        } else {
            r.inputs
                .iter()
                .map(|i| i.as_deref().map_or("—".to_string(), code))
                .collect()
        };
        let mut cells = vec![r.cell.clone(), level, format!("$2^{}$", r.column)];
        cells.extend(inputs);
        cells.push(driving(&r.sum, r.sum_result));
        cells.push(carry);
        if wide {
            let upper: Vec<String> = r.upper.iter().map(|o| driving(&o.name, o.result)).collect();
            cells.push(if upper.is_empty() {
                "—".to_string()
            } else {
                upper.join(", ")
            });
        }
        out.push_str(&format!("| {} |\n", cells.join(" | ")));
    }
    out
}
