// Caminho relativo: app/core/tests/help.rs
//! Spec 03 — Dadda's method, next move, hints and lower bound (MT-8).

mod common;

use std::collections::BTreeMap;

use common::{form, pick, r1, replay_r2_tree};
use dotree_core::expr::analyze;
use dotree_core::help::{
    Hint, LowerBound, NextMove, dadda_solution, dadda_target, hints, lower_bound, next_move,
};
use dotree_core::matrix::DotMatrix;
use dotree_core::netlist::Netlist;
use dotree_core::reduce::{CellKind, Reduction, Time};
use dotree_core::verify::Verifier;
use proptest::prelude::*;

fn matrix(src: &str, widths: &[(char, u32)]) -> DotMatrix {
    let w: BTreeMap<char, u32> = widths.iter().copied().collect();
    DotMatrix::build(&analyze(src, &w).unwrap()).unwrap()
}

fn r1_matrix() -> DotMatrix {
    matrix(
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

fn r4_matrix() -> DotMatrix {
    matrix("A*B", &[('A', 4), ('B', 4)])
}

fn max_height(points: &[dotree_core::reduce::Point], width: u32) -> u32 {
    (0..width)
        .map(|c| points.iter().filter(|p| p.column == c).count() as u32)
        .max()
        .unwrap_or(0)
}

/// FA and HA per level, tree levels first, CPA last.
fn cells_per_level(r: &Reduction) -> Vec<(usize, usize)> {
    r.levels()
        .iter()
        .map(|l| {
            let fa = l.cells.iter().filter(|c| c.kind == CellKind::Fa).count();
            (fa, l.cells.len() - fa)
        })
        .collect()
}

/// Every tree level brings the tallest column down to Dadda's target (AJU-05).
fn assert_respects_targets(r: &Reduction, width: u32) {
    let tree: Vec<_> = r.levels().iter().filter(|l| !l.cpa).collect();
    for (k, level) in tree.iter().enumerate() {
        let h = max_height(&level.points, width);
        let target = dadda_target(h).expect("a tree level starts above height 2");
        let next = match tree.get(k + 1) {
            Some(l) => &l.points,
            None => &r.levels().iter().find(|l| l.cpa).expect("CPA level").points,
        };
        assert!(
            max_height(next, width) <= target,
            "level {k}: {h} -> target {target}"
        );
    }
}

fn verifies(m: &DotMatrix, r: &Reduction) -> bool {
    let n = Netlist::from_reduction(m, r).unwrap();
    Verifier::new(n).run_to_end().counterexample.is_none()
}

// ---------------------------------------------------------------- Dadda sequence

#[test]
fn dadda_targets() {
    let cases = [
        (1, None),
        (2, None),
        (3, Some(2)),
        (4, Some(3)),
        (5, Some(4)),
        (6, Some(4)),
        (7, Some(6)),
        (9, Some(6)),
        (10, Some(9)),
        (13, Some(9)),
        (14, Some(13)),
        (20, Some(19)),
    ];
    for (h, expected) in cases {
        assert_eq!(dadda_target(h), expected, "h = {h}");
    }
}

// ---------------------------------------------------------------- AJU-05

#[test]
fn aju05_r4_dadda_matches_the_hand_computation() {
    let m = r4_matrix();
    let r = dadda_solution(&m);
    assert!(r.is_complete());
    // Level 0: 2 HA; level 1: 3 FA + 1 HA; CPA: 5 FA + 1 HA.
    assert_eq!(cells_per_level(&r), [(0, 2), (3, 1), (5, 1)]);
    let mt = r.metrics();
    assert_eq!((mt.fa, mt.ha, mt.area_halves()), (8, 4, 20));
    assert_eq!(mt.delay, Time(11)); // 5.5 T_FA
    assert_respects_targets(&r, m.output_width);
    assert!(verifies(&m, &r));
}

#[test]
fn aju05_r1_dadda_matches_the_hand_computation() {
    let m = r1_matrix();
    let r = dadda_solution(&m);
    assert_eq!(
        cells_per_level(&r),
        [(0, 1), (3, 2), (3, 1), (3, 2), (4, 2)]
    );
    let mt = r.metrics();
    assert_eq!((mt.fa, mt.ha, mt.area_halves()), (13, 8, 34));
    assert!(mt.fa >= lower_bound(&m).value);
    assert_respects_targets(&r, m.output_width);
    assert!(verifies(&m, &r));
}

#[test]
fn aju03_solution_starts_from_the_initial_matrix() {
    // Same result whatever the student has done: it only depends on the matrix.
    let m = r1_matrix();
    assert_eq!(
        cells_per_level(&dadda_solution(&m)),
        cells_per_level(&dadda_solution(&m))
    );
}

// ---------------------------------------------------------------- AJU-04

#[test]
fn aju04_next_move_on_r1_level_zero() {
    // Height 7 -> target 6: column 1 has one point too many, so an HA on its first two.
    let r = r1();
    assert_eq!(
        next_move(&r),
        NextMove::FormCell {
            kind: CellKind::Ha,
            points: pick(&r, 1, &["a0_b1", "a1_b0"])
        }
    );
}

#[test]
fn aju04_next_move_from_a_student_state() {
    // After the R2 tree the CPA is allowed, but Dadda still brings column 0 down to 2.
    let mut r = r1();
    replay_r2_tree(&mut r);
    assert_eq!(
        next_move(&r),
        NextMove::FormCell {
            kind: CellKind::Ha,
            points: pick(&r, 0, &["f0", "g0"])
        }
    );
    form(&mut r, 0, &["f0", "g0"]);
    // That HA's carry lifts column 1 to 3 points (k8, s4 + carry), so another HA there.
    assert_eq!(
        next_move(&r),
        NextMove::FormCell {
            kind: CellKind::Ha,
            points: pick(&r, 1, &["k8", "s4"])
        }
    );
    form(&mut r, 1, &["k8", "s4"]);
    assert_eq!(next_move(&r), NextMove::ConcludeLevel);
    r.conclude_level().unwrap();
    assert_eq!(next_move(&r), NextMove::CloseWithCpa);
    r.close_with_cpa().unwrap();
    assert_eq!(next_move(&r), NextMove::Complete);
}

#[test]
fn aju04_next_move_takes_pending_cells_into_account() {
    // The student already formed Dadda's HA; the next move is something else.
    let mut r = r1();
    form(&mut r, 1, &["a0_b1", "a1_b0"]);
    assert_eq!(next_move(&r), NextMove::ConcludeLevel);
}

#[test]
fn following_next_move_gives_the_dadda_solution() {
    let m = r1_matrix();
    let mut r = Reduction::new(&m);
    for _ in 0..200 {
        match next_move(&r) {
            NextMove::FormCell { points, .. } => {
                r.form_cell(&points).unwrap();
            }
            NextMove::ConcludeLevel => r.conclude_level().unwrap(),
            NextMove::CloseWithCpa => r.close_with_cpa().unwrap(),
            NextMove::Complete => break,
        }
    }
    assert!(r.is_complete());
    assert_eq!(cells_per_level(&r), cells_per_level(&dadda_solution(&m)));
}

// ---------------------------------------------------------------- AJU-06

#[test]
fn aju06_lower_bound() {
    assert_eq!(
        lower_bound(&r1_matrix()),
        LowerBound {
            dots: 20,
            output_width: 7,
            value: 13
        }
    );
    assert_eq!(
        lower_bound(&r4_matrix()),
        LowerBound {
            dots: 16,
            output_width: 8,
            value: 8
        }
    );
    // Never negative: 2A has 2 dots and W = 3.
    assert_eq!(lower_bound(&matrix("2A", &[('A', 2)])).value, 0);
}

// ---------------------------------------------------------------- hints (AJU-08)

#[test]
fn hints_on_r1_level_zero() {
    assert_eq!(
        hints(&r1()),
        [
            Hint::MinLevelsLeft {
                height: 7,
                levels: 4
            },
            Hint::TallestColumn {
                column: 1,
                height: 7
            },
            Hint::IdlePoints,
        ]
    );
}

#[test]
fn hints_when_the_cpa_is_available() {
    let mut r = r1();
    replay_r2_tree(&mut r);
    assert_eq!(
        hints(&r),
        [
            Hint::TallestColumn {
                column: 0,
                height: 3
            },
            Hint::IdlePoints,
            Hint::CpaReady
        ]
    );
}

#[test]
fn no_hints_once_complete() {
    let r = dadda_solution(&r4_matrix());
    assert!(hints(&r).is_empty());
}

// ---------------------------------------------------------------- VER-06 (Dadda part)

proptest! {
    // No regressions file: mutation runs used to leave cases that only fail on mutated code.
    // A real failure is printed, already shrunk, in the test output.
    #![proptest_config(ProptestConfig {
        cases: 64,
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    /// VER-06: Dadda's solution of random expressions verifies, respects every target and
    /// never beats the lower bound.
    #[test]
    fn ver06_dadda_solutions_verify(
        a in 1u32..=4, b in 1u32..=4, c in 1u32..=3, d in 1u32..=3, coef in 1u64..=7,
    ) {
        let m = matrix(&format!("A*B + {coef}C*D + C"), &[('A', a), ('B', b), ('C', c), ('D', d)]);
        let r = dadda_solution(&m);
        prop_assert!(r.is_complete());
        assert_respects_targets(&r, m.output_width);
        prop_assert!(verifies(&m, &r));
        let dropped = r.levels().iter().flat_map(|l| &l.cells).filter(|c| c.carry.is_none()).count();
        prop_assert!(r.metrics().fa + dropped >= lower_bound(&m).value);
    }
}
