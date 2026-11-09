// Caminho relativo: app/core/tests/help.rs
//! Spec 03 — Dadda's method, next move, hints and lower bound (MT-8).

mod common;

use std::collections::BTreeMap;

use common::{form, pick, r1, replay_r2_tree};
use dotree_core::expr::analyze;
use dotree_core::expr::{Analysis, parse};
use dotree_core::help::{
    HelpLog, HelpStep, Hint, LowerBound, Method, NextMove, dadda_solution, dadda_target, hints,
    lower_bound, next_move, solution, wallace_solution, wallace_target,
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
        // After the last tree level comes the CPA, or nothing when that level already
        // completed the circuit.
        let next = match tree.get(k + 1) {
            Some(l) => &l.points,
            None => r
                .levels()
                .iter()
                .find(|l| l.cpa)
                .map_or(r.points(), |l| &l.points),
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

// ---------------------------------------------------------------- AJU-07

#[test]
fn aju07_the_log_counts_each_step_of_each_help() {
    let mut log = HelpLog::default();
    assert!(log.is_empty());
    for step in [
        HelpStep::Hint,
        HelpStep::Hint,
        HelpStep::NextMove,
        HelpStep::LowerBound,
        HelpStep::LowerBoundPath,
        HelpStep::Solution,
    ] {
        log.record(step);
    }
    assert_eq!(
        log,
        HelpLog {
            hints: 2,
            next_moves: 1,
            lower_bound: 1,
            lower_bound_path: 1,
            solution: 1,
            solution_path: 0,
            solution_applied: 0,
            ..HelpLog::default()
        }
    );
    assert!(!log.is_empty());
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

// ------------------------------------------------------- AJU-09: using the solution (MT-20)

#[test]
fn aju09_adopting_the_solution_is_one_undo_step() {
    let m = r1_matrix();
    let solution = dadda_solution(&m);
    let mut r = Reduction::new(&m);
    form(&mut r, 1, &["a0_b1", "a1_b0"]);
    let before = (
        r.levels().to_vec(),
        r.points().to_vec(),
        r.pending().to_vec(),
    );

    assert!(r.adopt(&solution));
    assert_eq!(r.levels(), solution.levels());
    assert_eq!(r.points(), solution.points());
    assert_eq!(r.metrics(), solution.metrics());
    assert_eq!(r.result(), solution.result());
    assert!(
        Netlist::from_reduction(&m, &r).is_ok(),
        "the circuit is complete"
    );

    assert!(r.undo());
    assert_eq!(
        (
            r.levels().to_vec(),
            r.points().to_vec(),
            r.pending().to_vec()
        ),
        before,
        "one undo gives the student's circuit back"
    );
    assert!(r.redo());
    assert_eq!(r.result(), solution.result());
}

#[test]
fn aju09_adopting_drops_what_could_be_redone() {
    let m = r1_matrix();
    let mut r = Reduction::new(&m);
    form(&mut r, 1, &["a0_b1", "a1_b0"]);
    r.undo();
    assert!(r.can_redo());
    assert!(r.adopt(&dadda_solution(&m)));
    assert!(!r.can_redo());
}

#[test]
fn aju09_adopting_the_same_circuit_changes_nothing() {
    let m = r1_matrix();
    let solution = dadda_solution(&m);
    let mut r = solution.clone();
    let undoable = r.can_undo();
    assert!(!r.adopt(&solution));
    assert_eq!(r.can_undo(), undoable);
    assert_eq!(r.result(), solution.result());
}

#[test]
fn aju09_a_solution_of_another_matrix_is_refused() {
    let mut r = Reduction::new(&r1_matrix());
    assert!(!r.adopt(&dadda_solution(&r4_matrix())));
    assert!(!r.can_undo());
}

#[test]
fn aju09_applying_is_recorded_in_the_help_log() {
    let mut log = HelpLog::default();
    log.record(HelpStep::SolutionApplied);
    assert_eq!(log.solution_applied, 1);
    assert!(!log.is_empty());
}

// ------------------------------------------------------- AJU-10: Wallace's solution (MT-29)

/// Seven 6-bit operands, the example of the course reference (Parhami, *Computer
/// Arithmetic*, fig. 8.10). 42 input bits is over the interface's limit, so the
/// analysis is assembled here; the matrix is built as usual.
fn seven_operands() -> DotMatrix {
    let widths: BTreeMap<char, u32> = ('A'..='G').map(|v| (v, 6)).collect();
    let analysis = Analysis {
        polynomial: parse("A + B + C + D + E + F + G").unwrap(),
        widths,
        warnings: Vec::new(),
    };
    DotMatrix::build(&analysis).unwrap()
}

/// Every tree level brings the tallest column down to Wallace's target (AJU-10).
fn assert_respects_wallace_targets(r: &Reduction, width: u32) {
    let tree: Vec<_> = r.levels().iter().filter(|l| !l.cpa).collect();
    for (k, level) in tree.iter().enumerate() {
        let h = max_height(&level.points, width);
        let target = wallace_target(h).expect("a tree level starts above height 2");
        // After the last tree level comes the CPA, or nothing when that level already
        // completed the circuit.
        let next = match tree.get(k + 1) {
            Some(l) => &l.points,
            None => r
                .levels()
                .iter()
                .find(|l| l.cpa)
                .map_or(r.points(), |l| &l.points),
        };
        assert!(
            max_height(next, width) <= target,
            "level {k}: {h} -> target {target}"
        );
    }
}

#[test]
fn wallace_targets() {
    // 2 floor(h/3) + h mod 3: the rows left after grouping them in threes.
    let cases = [
        (1, None),
        (2, None),
        (3, Some(2)),
        (4, Some(3)),
        (5, Some(4)),
        (6, Some(4)),
        (7, Some(5)),
        (9, Some(6)),
        (13, Some(9)),
        (19, Some(13)),
    ];
    for (h, expected) in cases {
        assert_eq!(wallace_target(h), expected, "h = {h}");
    }
}

#[test]
fn aju10_r4_wallace_matches_the_hand_computation() {
    let m = r4_matrix();
    let r = wallace_solution(&m);
    assert!(r.is_complete());
    // Heights 1 2 3 4 3 2 1, target 3: an FA in each of 2^2, 2^3, 2^4. Heights
    // 1 2 1 3 2 3 1, target 2: FA in 2^3, HA in 2^4 (2 points and a carry), FA in 2^5.
    // CPA over 1 2 1 1 2 2 2: HA, HA, HA, FA, FA, FA.
    assert_eq!(cells_per_level(&r), [(3, 0), (2, 1), (3, 3)]);
    let mt = r.metrics();
    assert_eq!((mt.fa, mt.ha, mt.area_halves()), (8, 4, 20));
    assert_eq!(mt.delay, Time(11)); // 5.5 T_FA, as Dadda's
    assert_respects_wallace_targets(&r, m.output_width);
    assert!(verifies(&m, &r));
}

#[test]
fn aju10_r1_wallace_matches_the_hand_computation() {
    let m = r1_matrix();
    let r = wallace_solution(&m);
    // 5 7 5 2 1 -> 3 4 5 3 1 -> 1 3 4 2 2 -> 1 1 3 3 2 -> 1 1 1 2 2 1, then the CPA.
    assert_eq!(
        cells_per_level(&r),
        [(4, 0), (4, 0), (2, 0), (2, 1), (1, 2)]
    );
    let mt = r.metrics();
    // 13 FA and 3 HA, 14.5 A_FA, like the other solution of spec 00, but 6 T_FA instead of
    // 6.5; Dadda's solution has 17 A_FA and 7 T_FA.
    assert_eq!((mt.fa, mt.ha, mt.area_halves()), (13, 3, 29));
    assert_eq!(mt.delay, Time(12));
    assert!(mt.fa >= lower_bound(&m).value);
    assert_respects_wallace_targets(&r, m.output_width);
    assert!(verifies(&m, &r));
}

#[test]
fn aju10_seven_operands_as_in_the_course_reference() {
    let m = seven_operands();
    // Fig. 8.10: 12 FA, 6 FA, 6 FA, 4 FA + 1 HA, then a 7-bit adder (2^1 to 2^7).
    let r = wallace_solution(&m);
    assert_eq!(
        cells_per_level(&r),
        [(12, 0), (6, 0), (6, 0), (4, 1), (5, 2)]
    );
    assert_respects_wallace_targets(&r, m.output_width);
    assert!(verifies(&m, &r));
}

#[test]
fn aju10_a_tree_level_can_complete_the_circuit_without_cpa() {
    // Found by the property test in CI: 1-bit A*B + 5C*D + C has 3 points in 2^0 and one
    // in 2^2; Wallace's FA leaves one point per column, so no CPA follows.
    let m = matrix("A*B + 5C*D + C", &[('A', 1), ('B', 1), ('C', 1), ('D', 1)]);
    let r = wallace_solution(&m);
    assert!(r.is_complete());
    assert_eq!(cells_per_level(&r), [(1, 0)]);
    assert!(r.levels().iter().all(|l| !l.cpa));
    assert_respects_wallace_targets(&r, m.output_width);
    assert!(verifies(&m, &r));
}

#[test]
fn aju05_classic_dadda_on_seven_operands() {
    // The classic rule (an HA when only 1 is left to reduce), not the variant drawn in the
    // course reference (Parhami, fig. 8.14: 6, 11, 7 and 4 FA + 1 HA, 33 FA and 3 HA in
    // all). The figure also puts FA in columns already at their target, which no
    // column-by-column rule reproduces; spec 03 keeps the classic method (MT-29b).
    let m = seven_operands();
    let r = dadda_solution(&m);
    assert_eq!(
        cells_per_level(&r),
        [(5, 1), (10, 1), (5, 2), (6, 1), (7, 1)]
    );
    let mt = r.metrics();
    assert_eq!((mt.fa, mt.ha, mt.area_halves()), (33, 6, 72));
    assert_eq!(mt.delay, Time(20)); // 10 T_FA
    assert_respects_targets(&r, m.output_width);
    assert!(verifies(&m, &r));
}

#[test]
fn aju10_solution_by_method() {
    let m = r1_matrix();
    assert_eq!(
        solution(&m, Method::Dadda).levels(),
        dadda_solution(&m).levels()
    );
    assert_eq!(
        solution(&m, Method::Wallace).levels(),
        wallace_solution(&m).levels()
    );
}

#[test]
fn aju10_the_log_counts_wallace_apart() {
    let mut log = HelpLog::default();
    for step in [
        HelpStep::Wallace,
        HelpStep::WallacePath,
        HelpStep::WallacePath,
        HelpStep::WallaceApplied,
    ] {
        log.record(step);
    }
    assert_eq!(
        (log.wallace, log.wallace_path, log.wallace_applied),
        (1, 2, 1)
    );
    assert_eq!(
        (log.solution, log.solution_path, log.solution_applied),
        (0, 0, 0)
    );
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    /// VER-06: Wallace's solution of random expressions verifies, respects every target and
    /// never beats the lower bound.
    #[test]
    fn ver06_wallace_solutions_verify(
        a in 1u32..=4, b in 1u32..=4, c in 1u32..=3, d in 1u32..=3, coef in 1u64..=7,
        k in 0u64..=9,
    ) {
        // k = 0 is no constant: "+ 0" is not a term.
        let constant = if k == 0 { String::new() } else { format!(" + {k}") };
        let m = matrix(
            &format!("A*B + {coef}C*D + C{constant}"),
            &[('A', a), ('B', b), ('C', c), ('D', d)],
        );
        let r = wallace_solution(&m);
        prop_assert!(r.is_complete());
        assert_respects_wallace_targets(&r, m.output_width);
        prop_assert!(verifies(&m, &r));
        let dropped = r.levels().iter().flat_map(|l| &l.cells).filter(|c| c.carry.is_none()).count();
        prop_assert!(r.metrics().fa + dropped >= lower_bound(&m).value);
    }
}
