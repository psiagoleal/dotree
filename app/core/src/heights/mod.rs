// Caminho relativo: app/core/src/heights/mod.rs
//! The height table (spec 02, CMP-18; MT-30): the height of each column at each level and
//! the cells formed over it, the tabular form of the course reference (Parhami, fig. 8.11).

use crate::reduce::{CellKind, Point, Reduction};

/// What a row of the height table stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKind {
    /// A concluded tree level.
    Level,
    /// The level closed with the CPA.
    Cpa,
    /// The level being worked on, with the cells formed over it so far.
    Current,
    /// The complete circuit: one point or none per column.
    Result,
}

/// One row: `heights[j]` is the height of column `2^j`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeightRow {
    pub level: usize,
    pub kind: RowKind,
    pub heights: Vec<u32>,
    pub fa: usize,
    pub ha: usize,
}

/// Rows of the concluded levels, then the current level or, once complete, the result.
pub fn height_table(r: &Reduction) -> Vec<HeightRow> {
    let width = r.heights().len();
    let count = |kinds: &mut dyn Iterator<Item = CellKind>| {
        kinds.fold((0, 0), |(fa, ha), k| match k {
            CellKind::Fa => (fa + 1, ha),
            CellKind::Ha => (fa, ha + 1),
        })
    };
    let mut rows: Vec<HeightRow> = r
        .levels()
        .iter()
        .enumerate()
        .map(|(level, l)| {
            let (fa, ha) = count(&mut l.cells.iter().map(|c| c.kind));
            HeightRow {
                level,
                kind: if l.cpa { RowKind::Cpa } else { RowKind::Level },
                heights: heights(&l.points, width),
                fa,
                ha,
            }
        })
        .collect();
    let (fa, ha) = count(&mut r.pending().iter().map(|c| c.kind));
    rows.push(HeightRow {
        level: r.level(),
        kind: if r.is_complete() {
            RowKind::Result
        } else {
            RowKind::Current
        },
        heights: r.heights(),
        fa,
        ha,
    });
    rows
}

fn heights(points: &[Point], width: usize) -> Vec<u32> {
    let mut h = vec![0; width];
    for p in points {
        h[p.column as usize] += 1;
    }
    h
}

/// Words of the table, translated by the interface.
#[derive(Debug, Clone, Copy)]
pub struct HeightHeaders<'a> {
    pub level: &'a str,
    /// Marks the current level: `3 (atual)`.
    pub current: &'a str,
    /// Names the row of the complete circuit.
    pub result: &'a str,
}

/// The table as Markdown (CMP-18), highest column first, as in the level panels.
pub fn height_table_markdown(rows: &[HeightRow], headers: HeightHeaders) -> String {
    let width = rows.first().map_or(0, |r| r.heights.len());
    let columns: Vec<String> = (0..width).rev().map(|j| format!("$2^{j}$")).collect();
    let mut out = format!(
        "| {} | {} | FA | HA |\n",
        headers.level,
        columns.join(" | ")
    );
    out.push_str(&"|---".repeat(width + 3));
    out.push_str("|\n");
    for r in rows {
        let label = match r.kind {
            RowKind::Level => r.level.to_string(),
            RowKind::Cpa => format!("{} (CPA)", r.level),
            RowKind::Current => format!("{} ({})", r.level, headers.current),
            RowKind::Result => headers.result.to_string(),
        };
        let cells: Vec<String> = r.heights.iter().rev().map(u32::to_string).collect();
        let (fa, ha) = match r.kind {
            RowKind::Result => ("—".to_string(), "—".to_string()),
            _ => (r.fa.to_string(), r.ha.to_string()),
        };
        out.push_str(&format!(
            "| {label} | {} | {fa} | {ha} |\n",
            cells.join(" | ")
        ));
    }
    out
}
