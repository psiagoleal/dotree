// Caminho relativo: app/core/tests/common/mod.rs
//! Helpers shared by the integration tests: reference case R1 and the R2 replay.
#![allow(dead_code)]

use std::collections::BTreeMap;

use dotree_core::expr::analyze;
use dotree_core::matrix::DotMatrix;
use dotree_core::reduce::{CellKind, PointId, Reduction};

pub fn reduction(src: &str, widths: &[(char, u32)]) -> Reduction {
    let w: BTreeMap<char, u32> = widths.iter().copied().collect();
    Reduction::new(&DotMatrix::build(&analyze(src, &w).unwrap()).unwrap())
}

/// Reference case R1 (spec 00).
pub fn r1() -> Reduction {
    reduction(
        "A*B + C*D + 3(E+F) + G",
        &[
            ('A', 3),
            ('B', 3),
            ('C', 2),
            ('D', 2),
            ('E', 2),
            ('F', 1),
            ('G', 1),
        ],
    )
}

/// Ids of the current-level points with these signals in this column (signals repeat
/// across columns, e.g. `e0` in columns 0 and 1).
pub fn pick(r: &Reduction, column: u32, signals: &[&str]) -> Vec<PointId> {
    signals
        .iter()
        .map(|s| {
            r.points()
                .iter()
                .find(|p| p.column == column && r.signal(p) == *s)
                .unwrap_or_else(|| panic!("no {s} in column {column} at level {}", r.level()))
                .id
        })
        .collect()
}

pub fn form(r: &mut Reduction, column: u32, signals: &[&str]) -> CellKind {
    // Selected back to front on purpose: the cell must still list its inputs in canonical
    // order, so the R2 tables also guard that rule.
    let mut ids = pick(r, column, signals);
    ids.reverse();
    r.form_cell(&ids)
        .unwrap_or_else(|e| panic!("{signals:?}: {e:?}"))
}

/// Levels 0 to 2 of R2 (spec 00), i.e. everything before the CPA.
pub fn replay_r2_tree(r: &mut Reduction) {
    form(r, 1, &["a0_b1", "a1_b0", "c0_d1"]);
    form(r, 1, &["c1_d0", "e1", "e0"]);
    form(r, 2, &["a0_b2", "a1_b1"]);
    r.conclude_level().unwrap();
    form(r, 1, &["f0", "s1", "s2"]);
    form(r, 2, &["a2_b0", "c1_d1", "e1"]);
    form(r, 2, &["s3", "k1", "k2"]);
    form(r, 3, &["a1_b2", "a2_b1", "k3"]);
    r.conclude_level().unwrap();
    form(r, 0, &["a0_b0", "c0_d0", "e0"]);
    form(r, 2, &["s5", "k4", "s6"]);
    form(r, 3, &["k5", "s7", "k6"]);
    form(r, 4, &["a2_b2", "k7"]);
    r.conclude_level().unwrap();
}

/// Small deterministic generator for the random reduction.
pub struct Lcg(pub u64);
impl Lcg {
    pub fn next(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % n
    }
}

/// Reduces at random until the circuit is complete. Every FA and HA lowers
/// sum(height x (W - column)), so this always ends.
pub fn reduce_at_random(r: &mut Reduction, g: &mut Lcg) {
    while !r.is_complete() {
        if r.cpa_status().is_ok() && g.next(2) == 0 {
            r.close_with_cpa().unwrap();
            continue;
        }
        let mut formed = false;
        for column in 0..r.heights().len() as u32 {
            loop {
                let used: Vec<usize> = r.pending().iter().flat_map(|c| c.inputs.clone()).collect();
                let free: Vec<usize> = r
                    .points()
                    .iter()
                    .filter(|p| p.column == column && !used.contains(&p.id))
                    .map(|p| p.id)
                    .collect();
                if free.len() < 2 || (formed && g.next(3) == 0) {
                    break;
                }
                let n = if free.len() >= 3 && g.next(3) != 0 {
                    3
                } else {
                    2
                };
                r.form_cell(&free[..n]).unwrap();
                formed = true;
            }
        }
        r.conclude_level().unwrap();
    }
}
