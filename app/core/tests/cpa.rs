// Caminho relativo: app/core/tests/cpa.rs
//! Spec 02 — CPA, completion, metrics and critical path (MT-6). R2 in full (spec 00, 05).

mod common;

use common::{form, r1, reduction, replay_r2_tree};
use dotree_core::reduce::{CellKind, CpaBlock, Reduction, Rejection, Time};

fn signal_of(r: &Reduction, id: usize) -> String {
    r.signal(r.point(id).unwrap_or_else(|| panic!("no point {id}")))
}

fn r2() -> Reduction {
    let mut r = r1();
    replay_r2_tree(&mut r);
    r.close_with_cpa().unwrap();
    r
}

// ---------------------------------------------------------------- CMP-10

#[test]
fn cmp10_initial_r1_is_too_tall() {
    assert_eq!(
        r1().cpa_status(),
        Err(CpaBlock::ColumnTooTall {
            column: 0,
            height: 5
        })
    );
}

#[test]
fn cmp10_r2_before_the_cpa_is_ready() {
    let mut r = r1();
    replay_r2_tree(&mut r);
    assert_eq!(r.cpa_status(), Ok(()));
}

#[test]
fn cmp10_three_points_only_in_the_first_column() {
    // Heights [2, 3, 0, 0]: the 3 is not in the lowest column with 2 or more.
    let r = reduction(
        "A + B + 2C + 2D + 2E",
        &[('A', 1), ('B', 1), ('C', 1), ('D', 1), ('E', 1)],
    );
    assert_eq!(r.heights(), vec![2, 3, 0, 0]);
    assert_eq!(r.cpa_status(), Err(CpaBlock::ThreeNotFirst { column: 1 }));
}

#[test]
fn cmp10_nothing_to_add() {
    let r = reduction("A", &[('A', 4)]);
    assert_eq!(r.cpa_status(), Err(CpaBlock::NothingToAdd));
}

#[test]
fn cmp10_pending_cells_block_the_cpa() {
    let mut r = r1();
    replay_r2_tree(&mut r);
    form(&mut r, 3, &["k9", "s10"]);
    assert_eq!(r.cpa_status(), Err(CpaBlock::PendingCells));
    assert_eq!(
        r.close_with_cpa(),
        Err(Rejection::CpaUnavailable(CpaBlock::PendingCells))
    );
}

// ---------------------------------------------------------------- CMP-11, CMP-12: R2

#[test]
fn cmp11_r2_cpa_matches_the_reference_table() {
    let r = r2();
    let cpa = r.levels().last().unwrap();
    assert!(cpa.cpa);
    let rows: Vec<(usize, CellKind, u32, Vec<String>)> = cpa
        .cells
        .iter()
        .map(|c| {
            let ins = c.inputs.iter().map(|&id| signal_of(&r, id)).collect();
            (c.number, c.kind, c.column, ins)
        })
        .collect();
    use CellKind::{Fa, Ha};
    let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    assert_eq!(
        rows,
        vec![
            (12, Fa, 0, s(&["f0", "g0", "s8"])),
            (13, Fa, 1, s(&["k8", "s4", "k12"])),
            (14, Ha, 2, s(&["s9", "k13"])),
            (15, Fa, 3, s(&["k9", "s10", "k14"])),
            (16, Fa, 4, s(&["s11", "k10", "k15"])),
            (17, Ha, 5, s(&["k11", "k16"])),
        ]
    );
}

#[test]
fn cmp12_r2_is_complete_with_named_result_bits() {
    let r = r2();
    assert!(r.is_complete());
    let bits: Vec<String> = r
        .result()
        .unwrap()
        .into_iter()
        .map(|b| b.map(|id| signal_of(&r, id)).unwrap_or_else(|| "0".into()))
        .collect();
    assert_eq!(bits, ["s12", "s13", "s14", "s15", "s16", "s17", "k17"]);
}

#[test]
fn cmp12_constant_zero_bit_and_trivial_circuit() {
    // 2A: column 0 is empty, so R0 is the constant 0.
    let r = reduction("2A", &[('A', 2)]);
    assert!(r.is_complete());
    let bits: Vec<Option<String>> = r
        .result()
        .unwrap()
        .into_iter()
        .map(|b| b.map(|id| signal_of(&r, id)))
        .collect();
    assert_eq!(bits, [None, Some("a0".into()), Some("a1".into())]);

    // R3: nothing to do, delay 0.
    let r = reduction("A", &[('A', 4)]);
    assert!(r.is_complete());
    assert_eq!(r.metrics().delay, Time(0));
}

#[test]
fn incomplete_circuit_has_no_result() {
    assert!(!r1().is_complete());
    assert_eq!(r1().result(), None);
}

#[test]
fn cmp11_empty_column_inside_the_cpa_becomes_a_wire() {
    // A + B + 4C, 1 bit each: heights [2, 0, 1]. The HA's carry crosses the empty column 1
    // as a wire; column 2 keeps c0 alone.
    let mut r = reduction("A + B + 4C", &[('A', 1), ('B', 1), ('C', 1)]);
    assert_eq!(r.heights(), vec![2, 0, 1]);
    r.close_with_cpa().unwrap();
    let cpa = r.levels().last().unwrap();
    assert_eq!(cpa.cells.len(), 1);
    assert_eq!(cpa.cells[0].kind, CellKind::Ha);
    let bits: Vec<String> = r
        .result()
        .unwrap()
        .into_iter()
        .map(|b| signal_of(&r, b.unwrap()))
        .collect();
    assert_eq!(bits, ["s1", "k1", "c0"]);
}

#[test]
fn after_the_cpa_nothing_else_can_be_formed_but_undo_works() {
    let mut r = r2();
    assert_eq!(r.conclude_level(), Err(Rejection::EmptyLevel));
    assert_eq!(
        r.close_with_cpa(),
        Err(Rejection::CpaUnavailable(CpaBlock::NothingToAdd))
    );
    assert!(r.undo());
    assert!(!r.is_complete());
    assert_eq!(r.cpa_status(), Ok(()));
    assert!(r.redo());
    assert!(r.is_complete());
}

// ---------------------------------------------------------------- CMP-13 to CMP-15, GER-04

#[test]
fn ger04_r2_metrics() {
    let m = r2().metrics();
    assert_eq!((m.fa, m.ha, m.and_gates), (13, 4, 13));
    assert_eq!(m.area_halves(), 30);
    assert_eq!(m.area_a_fa(), 15.0);
    assert_eq!(m.delay, Time(12)); // 6 T_FA
}

#[test]
fn cmp14_metrics_follow_every_formed_cell() {
    let mut r = r1();
    assert_eq!(r.metrics().fa, 0);
    assert_eq!(r.metrics().delay, Time(0));
    form(&mut r, 2, &["a0_b2", "a1_b1"]);
    let m = r.metrics();
    assert_eq!((m.fa, m.ha, m.delay), (0, 1, Time(1)));
    form(&mut r, 1, &["a0_b1", "a1_b0", "c0_d1"]);
    assert_eq!((r.metrics().fa, r.metrics().delay), (1, Time(2)));
}

#[test]
fn cmp15_r2_critical_cells() {
    // Worked out by hand from the readiness times of R2 (each critical cell has an input
    // that is ready exactly one cell delay before it).
    let c = r2().critical();
    assert_eq!(c.cells, vec![1, 2, 4, 6, 8, 9, 12, 13, 14, 15, 16, 17]);
    assert!(c.pending.is_empty());
}

#[test]
fn cmp15_critical_path_includes_pending_cells() {
    let mut r = r1();
    form(&mut r, 2, &["a0_b2", "a1_b1"]); // HA, ready at 0.5
    form(&mut r, 1, &["a0_b1", "a1_b0", "c0_d1"]); // FA, ready at 1: the critical one
    let c = r.critical();
    assert_eq!(c.pending, vec![1]);
    assert!(c.cells.is_empty());
}
