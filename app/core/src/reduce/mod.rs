// Caminho relativo: app/core/src/reduce/mod.rs
//! Level-by-level reduction with FA and HA cells (spec 02; MT-5).
//!
//! The user forms cells over the current level; concluding the level numbers the cells
//! (column ascending, then formation order) and builds the next level from the unused points
//! plus the cells' outputs. Every successful action is undoable (CMP-03).

use std::cmp::Ordering;
use std::fmt;

use crate::matrix::DotMatrix;

/// Readiness time in **half** `T_FA` units, so 0.5 `T_FA` steps stay exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Time(pub u32);

impl Time {
    pub const FA_DELAY: Time = Time(2);
    pub const HA_DELAY: Time = Time(1);

    /// Value in `T_FA`, e.g. `Time(3).t_fa() == 1.5`.
    pub fn t_fa(self) -> f64 {
        f64::from(self.0) / 2.0
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 % 2 == 0 {
            write!(f, "{}", self.0 / 2)
        } else {
            write!(f, "{}.5", self.0 / 2)
        }
    }
}

/// Stable identifier of a point across the whole reduction. Input points reuse the dot id.
pub type PointId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// Input dot of the level-0 matrix (index into `DotMatrix::dots`).
    Input { dot: usize },
    /// Sum output of cell `cell` (its 1-based number).
    Sum { cell: usize },
    /// Carry output of cell `cell`.
    Carry { cell: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub id: PointId,
    pub column: u32,
    pub ready: Time,
    pub origin: Origin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellKind {
    Fa,
    Ha,
}

/// A cell formed over the current level, not numbered yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingCell {
    pub kind: CellKind,
    pub column: u32,
    /// Inputs in canonical order (spec 02, "Ordem das entradas").
    pub inputs: Vec<PointId>,
    pub ready: Time,
}

/// A numbered cell of a concluded level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub number: usize,
    pub kind: CellKind,
    /// Index of the level whose points it consumed.
    pub level: usize,
    pub column: u32,
    pub inputs: Vec<PointId>,
    pub sum: PointId,
    /// `None` when the carry fell at or beyond column `W` and was dropped (CMP-09).
    pub carry: Option<PointId>,
    pub ready: Time,
}

/// A concluded level: its points and the cells formed over them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Level {
    pub points: Vec<Point>,
    pub cells: Vec<Cell>,
}

/// Why an action was refused (GER-02). Carries only ids the user selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejection {
    /// CMP-06: "a cell adds bits of the same weight".
    MixedColumns,
    /// CMP-07: "FA takes 3 points; HA takes 2".
    WrongCount { count: usize },
    /// CMP-08: "each point goes into one cell per level".
    AlreadyInCell { point: PointId },
    /// The selected point is not in the current level.
    NotInCurrentLevel { point: PointId },
    /// The same point selected twice.
    DuplicateSelection { point: PointId },
    /// CMP-16: "a level needs at least one cell".
    EmptyLevel,
    /// Going back to a level that is not before the current one.
    NoSuchLevel { level: usize },
}

/// Everything an action can change. The history keeps one per successful action.
#[derive(Debug, Clone, PartialEq, Eq)]
struct State {
    levels: Vec<Level>,
    points: Vec<Point>,
    pending: Vec<PendingCell>,
    /// Next free point id; ids are never reused, even after going back.
    next_point: PointId,
}

/// A reduction in progress, with its undo history.
#[derive(Debug, Clone)]
pub struct Reduction {
    output_width: u32,
    input_signals: Vec<String>,
    history: Vec<State>,
    cursor: usize,
}

impl Reduction {
    pub fn new(matrix: &DotMatrix) -> Reduction {
        // Dots come in canonical generation order, which is the canonical order at t = 0.
        let points = matrix
            .dots
            .iter()
            .map(|d| Point {
                id: d.id,
                column: d.column,
                ready: Time(0),
                origin: Origin::Input { dot: d.id },
            })
            .collect();
        let input_signals = matrix
            .dots
            .iter()
            .map(|d| matrix.signals[d.signal].ident())
            .collect();
        let initial = State {
            levels: Vec::new(),
            points,
            pending: Vec::new(),
            next_point: matrix.dots.len(),
        };
        Reduction {
            output_width: matrix.output_width,
            input_signals,
            history: vec![initial],
            cursor: 0,
        }
    }

    fn state(&self) -> &State {
        &self.history[self.cursor]
    }

    /// Records a new state, dropping whatever could still be redone.
    fn commit(&mut self, state: State) {
        self.history.truncate(self.cursor + 1);
        self.history.push(state);
        self.cursor += 1;
    }

    /// Index of the current level (0 = the initial matrix).
    pub fn level(&self) -> usize {
        self.state().levels.len()
    }

    /// Points of the current level, in canonical order.
    pub fn points(&self) -> &[Point] {
        &self.state().points
    }

    /// Cells formed over the current level, in formation order.
    pub fn pending(&self) -> &[PendingCell] {
        &self.state().pending
    }

    /// Concluded levels, oldest first.
    pub fn levels(&self) -> &[Level] {
        &self.state().levels
    }

    /// Number of points per column `0..W` in the current level.
    pub fn heights(&self) -> Vec<u32> {
        let mut h = vec![0; self.output_width as usize];
        for p in self.points() {
            // Carries at or beyond W are dropped (CMP-09), so every column is below W.
            h[p.column as usize] += 1;
        }
        h
    }

    /// Identifier of the signal a point carries: `a1_b0`, `e0`, `s3`, `k3`.
    pub fn signal(&self, point: &Point) -> String {
        match point.origin {
            Origin::Input { dot } => self.input_signals[dot].clone(),
            Origin::Sum { cell } => format!("s{cell}"),
            Origin::Carry { cell } => format!("k{cell}"),
        }
    }

    /// Forms an FA (3 points) or an HA (2 points) over the current level (CMP-01, CMP-02).
    pub fn form_cell(&mut self, selection: &[PointId]) -> Result<CellKind, Rejection> {
        let kind = match selection.len() {
            3 => CellKind::Fa,
            2 => CellKind::Ha,
            count => return Err(Rejection::WrongCount { count }),
        };
        for (i, id) in selection.iter().enumerate() {
            if selection[..i].contains(id) {
                return Err(Rejection::DuplicateSelection { point: *id });
            }
        }
        let st = self.state();
        let mut inputs = Vec::with_capacity(selection.len());
        for &id in selection {
            let p = st
                .points
                .iter()
                .find(|p| p.id == id)
                .ok_or(Rejection::NotInCurrentLevel { point: id })?;
            if st.pending.iter().any(|c| c.inputs.contains(&id)) {
                return Err(Rejection::AlreadyInCell { point: id });
            }
            inputs.push(*p);
        }
        let column = inputs[0].column;
        if inputs.iter().any(|p| p.column != column) {
            return Err(Rejection::MixedColumns);
        }
        inputs.sort_by(canonical);
        let delay = if kind == CellKind::Fa {
            Time::FA_DELAY
        } else {
            Time::HA_DELAY
        };
        let ready = Time(inputs.iter().map(|p| p.ready.0).max().unwrap_or(0) + delay.0);

        let mut next = st.clone();
        next.pending.push(PendingCell {
            kind,
            column,
            inputs: inputs.iter().map(|p| p.id).collect(),
            ready,
        });
        self.commit(next);
        Ok(kind)
    }

    /// Numbers the pending cells and builds the next level.
    pub fn conclude_level(&mut self) -> Result<(), Rejection> {
        let st = self.state();
        if st.pending.is_empty() {
            return Err(Rejection::EmptyLevel);
        }
        let mut next = st.clone();
        let level = next.levels.len();
        let first_number = next.levels.iter().map(|l| l.cells.len()).sum::<usize>() + 1;

        // Numbering: column ascending, then formation order (stable sort keeps the latter).
        let mut order: Vec<&PendingCell> = st.pending.iter().collect();
        order.sort_by_key(|c| c.column);

        let mut cells = Vec::with_capacity(order.len());
        let mut outputs = Vec::with_capacity(2 * order.len());
        for (i, pc) in order.into_iter().enumerate() {
            let number = first_number + i;
            let sum = Point {
                id: next.next_point,
                column: pc.column,
                ready: pc.ready,
                origin: Origin::Sum { cell: number },
            };
            next.next_point += 1;
            outputs.push(sum);
            let carry = if pc.column + 1 < self.output_width {
                let k = Point {
                    id: next.next_point,
                    column: pc.column + 1,
                    ready: pc.ready,
                    origin: Origin::Carry { cell: number },
                };
                next.next_point += 1;
                outputs.push(k);
                Some(k.id)
            } else {
                None // always 0: the total never reaches 2^W (CMP-09)
            };
            cells.push(Cell {
                number,
                kind: pc.kind,
                level,
                column: pc.column,
                inputs: pc.inputs.clone(),
                sum: sum.id,
                carry,
                ready: pc.ready,
            });
        }

        let used: Vec<PointId> = st
            .pending
            .iter()
            .flat_map(|c| c.inputs.iter().copied())
            .collect();
        let mut points: Vec<Point> = st
            .points
            .iter()
            .filter(|p| !used.contains(&p.id))
            .copied()
            .collect();
        points.extend(outputs);
        points.sort_by(canonical);

        next.levels.push(Level {
            points: std::mem::replace(&mut next.points, points),
            cells,
        });
        next.pending.clear();
        self.commit(next);
        Ok(())
    }

    /// Returns to an earlier level: its cells become pending again, later ones are dropped
    /// (CMP-04).
    pub fn go_to_level(&mut self, level: usize) -> Result<(), Rejection> {
        let st = self.state();
        if level >= st.levels.len() {
            return Err(Rejection::NoSuchLevel { level });
        }
        let mut next = st.clone();
        let reopened = next.levels.swap_remove(level);
        next.levels.truncate(level);
        next.points = reopened.points;
        next.pending = reopened
            .cells
            .into_iter()
            .map(|c| PendingCell {
                kind: c.kind,
                column: c.column,
                inputs: c.inputs,
                ready: c.ready,
            })
            .collect();
        self.commit(next);
        Ok(())
    }

    pub fn undo(&mut self) -> bool {
        if self.cursor == 0 {
            return false;
        }
        self.cursor -= 1;
        true
    }

    pub fn redo(&mut self) -> bool {
        if self.cursor + 1 >= self.history.len() {
            return false;
        }
        self.cursor += 1;
        true
    }
}

/// Canonical order of points (spec 02): readiness, then inputs before outputs; inputs by
/// generation order, outputs by cell number with the sum before the carry.
pub fn canonical(a: &Point, b: &Point) -> Ordering {
    fn key(p: &Point) -> (Time, u8, usize, u8) {
        match p.origin {
            Origin::Input { dot } => (p.ready, 0, dot, 0),
            Origin::Sum { cell } => (p.ready, 1, cell, 0),
            Origin::Carry { cell } => (p.ready, 1, cell, 1),
        }
    }
    key(a).cmp(&key(b))
}
