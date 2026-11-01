// Caminho relativo: app/core/src/help/mod.rs
//! On-demand help (spec 03; MT-8): Dadda's method, the next move, hints and the lower bound.
//!
//! The automatic solution is literally "follow the next move until done", so the suggestion
//! the student gets and the solution in the side panel can never disagree. Everything here
//! returns codes and numbers; the interface words them (IDI-05) and asks before showing
//! anything (AJU-02).

use crate::matrix::DotMatrix;
use crate::reduce::{CellKind, PointId, Reduction, Time};

/// Largest number of Dadda's sequence 2, 3, 4, 6, 9, 13, … below `height`
/// (`d_{j+1} = floor(3 d_j / 2)`); `None` when `height <= 2`.
pub fn dadda_target(height: u32) -> Option<u32> {
    dadda_below(height).last()
}

/// Dadda's numbers strictly below `height`, ascending.
fn dadda_below(height: u32) -> impl Iterator<Item = u32> {
    std::iter::successors(Some(2u32), |&d| Some(d * 3 / 2)).take_while(move |&d| d < height)
}

/// What Dadda's method would do next from the current state (AJU-04).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NextMove {
    /// Form this cell; the points are in canonical order.
    FormCell {
        kind: CellKind,
        points: Vec<PointId>,
    },
    ConcludeLevel,
    CloseWithCpa,
    /// The circuit is complete.
    Complete,
}

pub fn next_move(reduction: &Reduction) -> NextMove {
    let r = reduction;
    if r.is_complete() {
        return NextMove::Complete;
    }
    let heights = r.heights();
    let width = heights.len();
    let tallest = heights.iter().copied().max().unwrap_or(0);
    let Some(target) = dadda_target(tallest) else {
        // Every column has at most 2 points: what remains is the CPA.
        return if r.pending().is_empty() {
            NextMove::CloseWithCpa
        } else {
            NextMove::ConcludeLevel
        };
    };

    // Height each column will have in the next level, given the cells already formed over
    // this one: free points + sums of cells in the column + carries from the column below.
    let used: Vec<PointId> = r
        .pending()
        .iter()
        .flat_map(|c| c.inputs.iter().copied())
        .collect();
    let mut sums = vec![0u32; width];
    let mut carries_in = vec![0u32; width + 1];
    for c in r.pending() {
        sums[c.column as usize] += 1;
        carries_in[c.column as usize + 1] += 1;
    }
    for column in 0..width {
        let free: Vec<PointId> = r
            .points()
            .iter()
            .filter(|p| p.column as usize == column && !used.contains(&p.id))
            .map(|p| p.id)
            .collect();
        let next_height = free.len() as u32 + sums[column] + carries_in[column];
        if next_height <= target {
            continue;
        }
        // Dadda: an FA removes 2 from this column's next height, an HA removes 1. The first
        // column over its target gets the next cell, on its earliest points (canonical order).
        let excess = next_height - target;
        if excess >= 2 && free.len() >= 3 {
            return NextMove::FormCell {
                kind: CellKind::Fa,
                points: free[..3].to_vec(),
            };
        }
        if free.len() >= 2 {
            return NextMove::FormCell {
                kind: CellKind::Ha,
                points: free[..2].to_vec(),
            };
        }
        // Not enough free points left here (the student's own cells used them): move on.
    }
    if r.pending().is_empty() {
        // Unreachable from a level whose tallest column exceeds the target (that column has
        // free points), kept as a safe answer.
        if r.cpa_status().is_ok() {
            NextMove::CloseWithCpa
        } else {
            NextMove::Complete
        }
    } else {
        NextMove::ConcludeLevel
    }
}

/// Dadda's solution from the initial matrix, as a complete reduction (AJU-05).
pub fn dadda_solution(matrix: &DotMatrix) -> Reduction {
    let mut r = Reduction::new(matrix);
    // Each step forms a cell or closes a level, and every FA/HA lowers
    // sum(height x (W - column)); this bound only guards against a bug turning into a hang.
    let limit = 4 * matrix.dots.len() * (matrix.output_width as usize + 1) + 16;
    for _ in 0..limit {
        let ok = match next_move(&r) {
            NextMove::FormCell { points, .. } => r.form_cell(&points).is_ok(),
            NextMove::ConcludeLevel => r.conclude_level().is_ok(),
            NextMove::CloseWithCpa => r.close_with_cpa().is_ok(),
            NextMove::Complete => return r,
        };
        assert!(ok, "Dadda's next move must always be valid");
    }
    panic!("Dadda's method did not finish");
}

/// Lower bound on FA + dropped carries (AJU-06), with the numbers behind it for step 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LowerBound {
    pub dots: usize,
    pub output_width: u32,
    pub value: usize,
}

pub fn lower_bound(matrix: &DotMatrix) -> LowerBound {
    let dots = matrix.dots.len();
    let output_width = matrix.output_width;
    LowerBound {
        dots,
        output_width,
        value: dots.saturating_sub(output_width as usize),
    }
}

/// Hint catalog (spec 03). Only numbers: no point or cell identifiers (AJU-08).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hint {
    /// `DICA_NIVEIS_RESTANTES`: with tallest column `height`, at least `levels` more levels
    /// are needed before the CPA.
    MinLevelsLeft { height: u32, levels: u32 },
    /// `DICA_COLUNA_MAIS_ALTA`.
    TallestColumn { column: u32, height: u32 },
    /// `DICA_PONTO_PARADO`: points ready at t = 0, in no cell yet, in a column with 3 or more.
    IdlePoints,
    /// `DICA_CPA_DISPONIVEL`: the current level can already be closed with the CPA.
    CpaReady,
}

/// Every hint that applies now, in catalog order; the interface shows the next one on each
/// request, cycling (AJU-04, AJU-08).
pub fn hints(reduction: &Reduction) -> Vec<Hint> {
    let r = reduction;
    if r.is_complete() {
        return Vec::new();
    }
    let heights = r.heights();
    let height = heights.iter().copied().max().unwrap_or(0);
    let cpa_ready = r.cpa_status().is_ok();
    let mut out = Vec::new();
    if !cpa_ready && height > 2 {
        out.push(Hint::MinLevelsLeft {
            height,
            levels: dadda_below(height).count() as u32,
        });
    }
    if let Some(column) = heights.iter().position(|&h| h == height) {
        out.push(Hint::TallestColumn {
            column: column as u32,
            height,
        });
    }
    let used: Vec<PointId> = r
        .pending()
        .iter()
        .flat_map(|c| c.inputs.iter().copied())
        .collect();
    let idle = r
        .points()
        .iter()
        .any(|p| p.ready == Time(0) && !used.contains(&p.id) && heights[p.column as usize] >= 3);
    if idle {
        out.push(Hint::IdlePoints);
    }
    if cpa_ready {
        out.push(Hint::CpaReady);
    }
    out
}

/// One revealed step of a help (AJU-07). The rules are not counted: they are always visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpStep {
    Hint,
    NextMove,
    /// Step 1 of the lower bound: the number.
    LowerBound,
    /// Step 2: the formula and why.
    LowerBoundPath,
    /// Step 1 of the automatic solution: its final metrics.
    Solution,
    /// Step 2: the solution level by level.
    SolutionPath,
}

/// How many times each help step was revealed, for the summary of a complete circuit
/// (AJU-07). It is not part of the undo history: undoing does not take a help back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HelpLog {
    pub hints: u32,
    pub next_moves: u32,
    pub lower_bound: u32,
    pub lower_bound_path: u32,
    pub solution: u32,
    pub solution_path: u32,
}

impl HelpLog {
    pub fn record(&mut self, step: HelpStep) {
        let n = match step {
            HelpStep::Hint => &mut self.hints,
            HelpStep::NextMove => &mut self.next_moves,
            HelpStep::LowerBound => &mut self.lower_bound,
            HelpStep::LowerBoundPath => &mut self.lower_bound_path,
            HelpStep::Solution => &mut self.solution,
            HelpStep::SolutionPath => &mut self.solution_path,
        };
        *n += 1;
    }

    pub fn is_empty(&self) -> bool {
        *self == HelpLog::default()
    }
}
