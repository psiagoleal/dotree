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
    /// True for the final ripple-carry adder (CMP-11).
    pub cpa: bool,
}

/// Why "close with CPA" is not available on the current level (CMP-10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CpaBlock {
    /// A column has more than 3 points.
    ColumnTooTall { column: u32, height: u32 },
    /// A column with 3 points that is not the lowest column with 2 or more.
    ThreeNotFirst { column: u32 },
    /// No column has 2 or more points: there is nothing to add.
    NothingToAdd,
    /// Cells were formed over this level and not concluded yet.
    PendingCells,
}

/// Counts and costs of the circuit built so far (CMP-14, CMP-15, GER-04).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Metrics {
    pub fa: usize,
    pub ha: usize,
    pub and_gates: usize,
    /// Latest readiness among the current points and the pending cells.
    pub delay: Time,
}

impl Metrics {
    /// Area in half `A_FA` units (FA = 2, HA = 1), exact.
    pub fn area_halves(&self) -> u32 {
        (2 * self.fa + self.ha) as u32
    }

    /// Area in `A_FA`.
    pub fn area_a_fa(&self) -> f64 {
        f64::from(self.area_halves()) / 2.0
    }
}

/// Cells on some path of maximum delay (CMP-15, DIA-05).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Critical {
    /// Numbers of concluded cells, ascending.
    pub cells: Vec<usize>,
    /// Indices into [`Reduction::pending`], ascending.
    pub pending: Vec<usize>,
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
    /// CMP-10: the CPA cannot close this level yet.
    CpaUnavailable(CpaBlock),
    /// Going back to a level that is not before the current one.
    NoSuchLevel { level: usize },
}

/// Everything an action can change. The history keeps one per successful action.
#[derive(Debug, Clone, PartialEq, Eq)]
struct State {
    levels: Vec<Level>,
    points: Vec<Point>,
    pending: Vec<PendingCell>,
    /// Every point ever created, indexed by id; ids are never reused, even after going back.
    registry: Vec<Point>,
}

impl State {
    fn new_point(&mut self, column: u32, ready: Time, origin: Origin) -> Point {
        let p = Point {
            id: self.registry.len(),
            column,
            ready,
            origin,
        };
        self.registry.push(p);
        p
    }
}

/// A reduction in progress, with its undo history.
#[derive(Debug, Clone)]
pub struct Reduction {
    output_width: u32,
    and_gates: usize,
    input_signals: Vec<String>,
    history: Vec<State>,
    cursor: usize,
}

impl Reduction {
    pub fn new(matrix: &DotMatrix) -> Reduction {
        // Dots come in canonical generation order, which is the canonical order at t = 0.
        let points: Vec<Point> = matrix
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
            pending: Vec::new(),
            registry: points.clone(),
            points,
        };
        Reduction {
            output_width: matrix.output_width,
            and_gates: matrix.and_gates(),
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
            let sum = next.new_point(pc.column, pc.ready, Origin::Sum { cell: number });
            outputs.push(sum);
            let carry = if pc.column + 1 < self.output_width {
                let k = next.new_point(pc.column + 1, pc.ready, Origin::Carry { cell: number });
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
            cpa: false,
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
        // The CPA's cells chain carries that are not points of the level, so they cannot be
        // edited as pending cells: going back to the CPA's level just removes it.
        let cells = if reopened.cpa {
            Vec::new()
        } else {
            reopened.cells
        };
        next.pending = cells
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

    /// Any point ever created in this reduction, including the CPA's internal carries.
    pub fn point(&self, id: PointId) -> Option<&Point> {
        self.state().registry.get(id)
    }

    /// Whether the current level can be closed with a CPA (CMP-10).
    pub fn cpa_status(&self) -> Result<(), CpaBlock> {
        if !self.pending().is_empty() {
            return Err(CpaBlock::PendingCells);
        }
        let h = self.heights();
        if let Some((column, &height)) = h.iter().enumerate().find(|&(_, &x)| x > 3) {
            return Err(CpaBlock::ColumnTooTall {
                column: column as u32,
                height,
            });
        }
        let first = h
            .iter()
            .position(|&x| x >= 2)
            .ok_or(CpaBlock::NothingToAdd)?;
        if let Some(column) = h
            .iter()
            .enumerate()
            .position(|(c, &x)| x == 3 && c != first)
        {
            return Err(CpaBlock::ThreeNotFirst {
                column: column as u32,
            });
        }
        Ok(())
    }

    /// Closes the current level with a ripple-carry adder (CMP-11).
    pub fn close_with_cpa(&mut self) -> Result<(), Rejection> {
        self.cpa_status().map_err(Rejection::CpaUnavailable)?;
        let st = self.state();
        let mut next = st.clone();
        let level = next.levels.len();
        let mut number = next.levels.iter().map(|l| l.cells.len()).sum::<usize>();
        let first = self.heights().iter().position(|&x| x >= 2).unwrap_or(0) as u32;

        let mut cells = Vec::new();
        let mut result: Vec<Point> = Vec::new();
        let mut carry: Option<Point> = None;
        for column in 0..self.output_width {
            let mut inputs: Vec<Point> = st
                .points
                .iter()
                .filter(|p| p.column == column)
                .copied()
                .collect();
            // Below the CPA every column already holds its result bit (at most one point).
            if column < first {
                result.extend(inputs);
                continue;
            }
            // The incoming carry is always the last input (spec 02, "Ordem das entradas").
            inputs.extend(carry.take());
            let kind = match inputs.len() {
                0 => continue,
                1 => {
                    result.push(inputs[0]); // a wire: the point itself is the result bit
                    continue;
                }
                2 => CellKind::Ha,
                _ => CellKind::Fa, // 3: CMP-10 guarantees no more
            };
            number += 1;
            let delay = if kind == CellKind::Fa {
                Time::FA_DELAY
            } else {
                Time::HA_DELAY
            };
            let ready = Time(inputs.iter().map(|p| p.ready.0).max().unwrap_or(0) + delay.0);
            let sum = next.new_point(column, ready, Origin::Sum { cell: number });
            result.push(sum);
            carry = (column + 1 < self.output_width)
                .then(|| next.new_point(column + 1, ready, Origin::Carry { cell: number }));
            cells.push(Cell {
                number,
                kind,
                level,
                column,
                inputs: inputs.iter().map(|p| p.id).collect(),
                sum: sum.id,
                carry: carry.map(|k| k.id),
                ready,
            });
        }

        result.sort_by(canonical);
        next.levels.push(Level {
            points: std::mem::replace(&mut next.points, result),
            cells,
            cpa: true,
        });
        self.commit(next);
        Ok(())
    }

    /// Every column has at most one point (CMP-12).
    pub fn is_complete(&self) -> bool {
        self.heights().iter().all(|&h| h <= 1)
    }

    /// Result bits `R_0 .. R_{W-1}` when complete: the point that carries each bit, or `None`
    /// for a constant 0 (CMP-12).
    pub fn result(&self) -> Option<Vec<Option<PointId>>> {
        self.is_complete().then(|| {
            (0..self.output_width)
                .map(|c| self.points().iter().find(|p| p.column == c).map(|p| p.id))
                .collect()
        })
    }

    /// FA, HA and AND counts and the current delay (CMP-14, CMP-15).
    pub fn metrics(&self) -> Metrics {
        let kinds = self
            .levels()
            .iter()
            .flat_map(|l| l.cells.iter().map(|c| c.kind))
            .chain(self.pending().iter().map(|c| c.kind));
        let (mut fa, mut ha) = (0, 0);
        for k in kinds {
            match k {
                CellKind::Fa => fa += 1,
                CellKind::Ha => ha += 1,
            }
        }
        let delay = self
            .points()
            .iter()
            .map(|p| p.ready)
            .chain(self.pending().iter().map(|c| c.ready))
            .max()
            .unwrap_or_default();
        Metrics {
            fa,
            ha,
            and_gates: self.and_gates,
            delay,
        }
    }

    /// Every cell on a path of maximum delay (CMP-15).
    pub fn critical(&self) -> Critical {
        let delay = self.metrics().delay;
        let mut out = Critical::default();
        if delay == Time(0) {
            return out;
        }
        let delay_of = |k: CellKind| {
            if k == CellKind::Fa {
                Time::FA_DELAY.0
            } else {
                Time::HA_DELAY.0
            }
        };
        // Walk back from every point (or pending cell) that finishes at the maximum delay:
        // a cell is critical when it produces a critical point; its critical inputs are those
        // ready exactly one cell delay before it.
        let mut stack: Vec<PointId> = self
            .points()
            .iter()
            .filter(|p| p.ready == delay)
            .map(|p| p.id)
            .collect();
        for (i, pc) in self.pending().iter().enumerate() {
            if pc.ready == delay {
                out.pending.push(i);
                let want = pc.ready.0 - delay_of(pc.kind);
                stack.extend(
                    pc.inputs
                        .iter()
                        .copied()
                        .filter(|&id| self.state().registry[id].ready.0 == want),
                );
            }
        }
        let cells: Vec<&Cell> = self.levels().iter().flat_map(|l| &l.cells).collect();
        while let Some(id) = stack.pop() {
            let number = match self.state().registry[id].origin {
                Origin::Input { .. } => continue,
                Origin::Sum { cell } | Origin::Carry { cell } => cell,
            };
            if out.cells.contains(&number) {
                continue;
            }
            out.cells.push(number);
            let cell = cells[number - 1];
            let want = cell.ready.0 - delay_of(cell.kind);
            stack.extend(
                cell.inputs
                    .iter()
                    .copied()
                    .filter(|&i| self.state().registry[i].ready.0 == want),
            );
        }
        out.cells.sort_unstable();
        out
    }

    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }

    pub fn can_redo(&self) -> bool {
        self.cursor + 1 < self.history.len()
    }

    pub fn undo(&mut self) -> bool {
        if !self.can_undo() {
            return false;
        }
        self.cursor -= 1;
        true
    }

    pub fn redo(&mut self) -> bool {
        if !self.can_redo() {
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
