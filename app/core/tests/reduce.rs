// Caminho relativo: app/core/tests/reduce.rs
//! Spec 02 — cells and levels (MT-5). The R2 replay follows the wiring table of spec 05.

mod common;

use common::{form, pick, r1, reduction, replay_r2_tree};
use dotree_core::reduce::{CellKind, Origin, PointId, Reduction, Rejection, Time};

/// Snapshot of everything observable, to compare states across undo/redo.
fn observe(r: &Reduction) -> String {
    format!(
        "{} {:?} {:?} {:?}",
        r.level(),
        r.points(),
        r.pending(),
        r.levels()
    )
}

// ---------------------------------------------------------------- initial state

#[test]
fn starts_at_level_zero_with_the_matrix() {
    let r = r1();
    assert_eq!(r.level(), 0);
    assert_eq!(r.points().len(), 20);
    assert_eq!(r.heights(), vec![5, 7, 5, 2, 1, 0, 0]);
    assert!(r.pending().is_empty());
    assert!(r.levels().is_empty());
    assert!(r.points().iter().all(|p| p.ready == Time(0)));
}

// ---------------------------------------------------------------- CMP-01, CMP-02

#[test]
fn cmp01_fa_puts_sum_and_carry_in_the_next_level() {
    let mut r = r1();
    assert_eq!(form(&mut r, 1, &["a0_b1", "a1_b0", "c0_d1"]), CellKind::Fa);
    assert_eq!(r.pending().len(), 1);
    assert_eq!(r.pending()[0].ready, Time(2));
    r.conclude_level().unwrap();
    assert_eq!(r.level(), 1);
    // column 1 loses 3 and gains the sum; column 2 gains the carry.
    assert_eq!(r.heights(), vec![5, 5, 6, 2, 1, 0, 0]);
    let s1 = r.points().iter().find(|p| r.signal(p) == "s1").unwrap();
    let k1 = r.points().iter().find(|p| r.signal(p) == "k1").unwrap();
    assert_eq!((s1.column, s1.ready), (1, Time(2)));
    assert_eq!((k1.column, k1.ready), (2, Time(2)));
}

#[test]
fn cmp02_ha_takes_half_a_t_fa() {
    let mut r = r1();
    assert_eq!(form(&mut r, 2, &["a0_b2", "a1_b1"]), CellKind::Ha);
    assert_eq!(r.pending()[0].ready, Time(1));
    assert_eq!(Time(1).t_fa(), 0.5);
    assert_eq!(Time(3).to_string(), "1.5");
    r.conclude_level().unwrap();
    assert_eq!(r.heights(), vec![5, 7, 4, 3, 1, 0, 0]);
}

// ---------------------------------------------------------------- R2 replay (spec 05 table)

#[test]
fn r2_tree_matches_the_reference_wiring_table() {
    let mut r = r1();
    replay_r2_tree(&mut r);
    assert_eq!(r.level(), 3);
    assert_eq!(r.heights(), vec![3, 2, 1, 2, 2, 1, 0]);

    let table: Vec<(usize, CellKind, usize, u32, Vec<String>, Time)> = r
        .levels()
        .iter()
        .flat_map(|l| &l.cells)
        .map(|c| {
            let ins = c
                .inputs
                .iter()
                .map(|id| {
                    let level = &r.levels()[c.level];
                    r.signal(level.points.iter().find(|p| p.id == *id).unwrap())
                })
                .collect();
            (c.number, c.kind, c.level, c.column, ins, c.ready)
        })
        .collect();

    use CellKind::{Fa, Ha};
    let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    let expected = vec![
        (1, Fa, 0, 1, s(&["a0_b1", "a1_b0", "c0_d1"]), Time(2)),
        (2, Fa, 0, 1, s(&["c1_d0", "e1", "e0"]), Time(2)),
        (3, Ha, 0, 2, s(&["a0_b2", "a1_b1"]), Time(1)),
        (4, Fa, 1, 1, s(&["f0", "s1", "s2"]), Time(4)),
        (5, Fa, 1, 2, s(&["a2_b0", "c1_d1", "e1"]), Time(2)),
        (6, Fa, 1, 2, s(&["s3", "k1", "k2"]), Time(4)),
        (7, Fa, 1, 3, s(&["a1_b2", "a2_b1", "k3"]), Time(3)),
        (8, Fa, 2, 0, s(&["a0_b0", "c0_d0", "e0"]), Time(2)),
        (9, Fa, 2, 2, s(&["s5", "k4", "s6"]), Time(6)),
        (10, Fa, 2, 3, s(&["k5", "s7", "k6"]), Time(6)),
        (11, Ha, 2, 4, s(&["a2_b2", "k7"]), Time(4)),
    ];
    assert_eq!(table, expected);
}

#[test]
fn inputs_come_out_in_canonical_order_whatever_the_selection_order() {
    let mut r = r1();
    let ids = pick(&r, 1, &["c0_d1", "a0_b1", "a1_b0"]);
    r.form_cell(&ids).unwrap();
    assert_eq!(
        r.pending()[0].inputs,
        pick(&r, 1, &["a0_b1", "a1_b0", "c0_d1"])
    );
}

#[test]
fn cells_are_numbered_by_column_then_formation_order() {
    let mut r = r1();
    form(&mut r, 2, &["a0_b2", "a1_b1"]); // formed first, but in column 2
    form(&mut r, 1, &["c1_d0", "e1", "e0"]);
    form(&mut r, 1, &["a0_b1", "a1_b0", "c0_d1"]);
    r.conclude_level().unwrap();
    let cells: Vec<(usize, u32, CellKind)> = r.levels()[0]
        .cells
        .iter()
        .map(|c| (c.number, c.column, c.kind))
        .collect();
    assert_eq!(
        cells,
        [
            (1, 1, CellKind::Fa),
            (2, 1, CellKind::Fa),
            (3, 2, CellKind::Ha)
        ]
    );
    // Inside column 1, formation order decides: (c1_d0, e1, e0) was formed first, so it is
    // FA1 and (a0_b1, a1_b0, c0_d1) is FA2.
    let first_input = |cell: usize| {
        let c = &r.levels()[0].cells[cell];
        r.signal(
            r.levels()[0]
                .points
                .iter()
                .find(|p| p.id == c.inputs[0])
                .unwrap(),
        )
    };
    assert_eq!(first_input(0), "c1_d0");
    assert_eq!(first_input(1), "a0_b1");
}

#[test]
fn next_level_points_are_in_canonical_order() {
    let mut r = r1();
    replay_r2_tree(&mut r);
    let col2: Vec<String> = r
        .points()
        .iter()
        .filter(|p| p.column == 2)
        .map(|p| r.signal(p))
        .collect();
    assert_eq!(col2, ["s9"]);
    let col0: Vec<String> = r
        .points()
        .iter()
        .filter(|p| p.column == 0)
        .map(|p| r.signal(p))
        .collect();
    // f0 and g0 (t = 0) before s8 (t = 1).
    assert_eq!(col0, ["f0", "g0", "s8"]);
}

// ---------------------------------------------------------------- refusals (GER-02)

#[test]
fn cmp06_mixed_columns() {
    let mut r = r1();
    let mut ids = pick(&r, 1, &["a0_b1", "a1_b0"]);
    ids.extend(pick(&r, 2, &["a0_b2"]));
    assert_eq!(r.form_cell(&ids), Err(Rejection::MixedColumns));
}

#[test]
fn cmp07_wrong_count() {
    let mut r = r1();
    let four = pick(&r, 1, &["a0_b1", "a1_b0", "c0_d1", "c1_d0"]);
    assert_eq!(r.form_cell(&four), Err(Rejection::WrongCount { count: 4 }));
    assert_eq!(
        r.form_cell(&four[..1]),
        Err(Rejection::WrongCount { count: 1 })
    );
    assert_eq!(r.form_cell(&[]), Err(Rejection::WrongCount { count: 0 }));
}

#[test]
fn cmp08_point_already_in_a_cell() {
    let mut r = r1();
    form(&mut r, 1, &["a0_b1", "a1_b0", "c0_d1"]);
    let again = pick(&r, 1, &["c0_d1", "c1_d0"]);
    assert_eq!(
        r.form_cell(&again),
        Err(Rejection::AlreadyInCell { point: again[0] })
    );
}

#[test]
fn point_outside_the_current_level_or_repeated() {
    let mut r = r1();
    let old = pick(&r, 1, &["a0_b1", "a1_b0", "c0_d1"]);
    r.form_cell(&old).unwrap();
    r.conclude_level().unwrap();
    assert_eq!(
        r.form_cell(&old),
        Err(Rejection::NotInCurrentLevel { point: old[0] })
    );
    let p = pick(&r, 1, &["c1_d0"])[0];
    assert_eq!(
        r.form_cell(&[p, p]),
        Err(Rejection::DuplicateSelection { point: p })
    );
}

#[test]
fn cmp16_empty_level_cannot_be_concluded() {
    let mut r = r1();
    assert_eq!(r.conclude_level(), Err(Rejection::EmptyLevel));
}

#[test]
fn refusals_do_not_change_state_or_history() {
    let mut r = r1();
    let before = observe(&r);
    let _ = r.form_cell(&[]);
    let _ = r.conclude_level();
    assert_eq!(observe(&r), before);
    assert!(!r.undo());
}

// ---------------------------------------------------------------- CMP-09

#[test]
fn cmp09_carry_beyond_the_output_width_is_dropped() {
    // A + B + C with 1 bit each: v_max = 3, W = 2. After two HAs, column 1 holds two carries
    // that can never both be 1; adding them drops a carry into column 2 = W.
    let mut r = reduction("A + B + C", &[('A', 1), ('B', 1), ('C', 1)]);
    form(&mut r, 0, &["a0", "b0"]);
    r.conclude_level().unwrap();
    form(&mut r, 0, &["c0", "s1"]);
    r.conclude_level().unwrap();
    assert_eq!(r.heights(), vec![1, 2]);
    form(&mut r, 1, &["k1", "k2"]);
    r.conclude_level().unwrap();
    let cell = &r.levels()[2].cells[0];
    assert_eq!(cell.carry, None);
    assert_eq!(r.heights(), vec![1, 1]);
    assert!(r.points().iter().all(|p| p.column < 2));
}

// ---------------------------------------------------------------- CMP-04

#[test]
fn cmp04_going_back_reopens_that_level() {
    let mut r = r1();
    replay_r2_tree(&mut r);
    r.go_to_level(1).unwrap();
    assert_eq!(r.level(), 1);
    assert_eq!(r.levels().len(), 1);
    // The cells formed over level 1 (FA4 to FA7) are pending again; level 2's are gone.
    assert_eq!(r.pending().len(), 4);
    // Level 1: col 1 = f0 + s1 + s2; col 2 = 3 inputs + s3 + k1 + k2; col 3 = 2 + k3.
    assert_eq!(r.heights(), vec![5, 3, 6, 3, 1, 0, 0]);
    assert_eq!(r.go_to_level(1), Err(Rejection::NoSuchLevel { level: 1 }));
    assert_eq!(r.go_to_level(7), Err(Rejection::NoSuchLevel { level: 7 }));
    // Concluding again rebuilds the same level 2.
    r.conclude_level().unwrap();
    // Level 2: exactly the inputs of FA8 to HA11 plus what waits (spec 05 table).
    assert_eq!(r.heights(), vec![5, 1, 3, 3, 2, 0, 0]);
    assert!(r.undo());
    assert!(r.undo());
    assert_eq!(r.level(), 3);
}

// ---------------------------------------------------------------- CMP-03

/// Small deterministic generator (no external crate).
struct Lcg(u64);
impl Lcg {
    fn next(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % n
    }
}

/// One random successful action; returns false if none was possible.
fn random_action(r: &mut Reduction, g: &mut Lcg) -> bool {
    for _ in 0..50 {
        match g.next(10) {
            0 if r.level() > 0 => {
                let lvl = g.next(r.level());
                if r.go_to_level(lvl).is_ok() {
                    return true;
                }
            }
            1..=2 => {
                if r.conclude_level().is_ok() {
                    return true;
                }
            }
            _ => {
                let used: Vec<PointId> =
                    r.pending().iter().flat_map(|c| c.inputs.clone()).collect();
                let free: Vec<_> = r
                    .points()
                    .iter()
                    .filter(|p| !used.contains(&p.id))
                    .collect();
                if free.is_empty() {
                    continue;
                }
                let col = free[g.next(free.len())].column;
                let same: Vec<PointId> = free
                    .iter()
                    .filter(|p| p.column == col)
                    .map(|p| p.id)
                    .collect();
                let n = if same.len() >= 3 && g.next(2) == 0 {
                    3
                } else {
                    2
                };
                if same.len() >= n && r.form_cell(&same[..n]).is_ok() {
                    return true;
                }
            }
        }
    }
    false
}

#[test]
fn cmp03_a_thousand_actions_undo_and_redo_exactly() {
    let mut r = reduction("A*B", &[('A', 8), ('B', 8)]);
    let mut g = Lcg(2026);
    let mut states = vec![observe(&r)];
    while states.len() <= 1000 {
        assert!(
            random_action(&mut r, &mut g),
            "stuck after {} actions",
            states.len() - 1
        );
        states.push(observe(&r));
    }
    for expected in states.iter().rev().skip(1) {
        assert!(r.undo());
        assert_eq!(&observe(&r), expected);
    }
    assert!(!r.undo());
    for expected in states.iter().skip(1) {
        assert!(r.redo());
        assert_eq!(&observe(&r), expected);
    }
    assert!(!r.redo());
}

#[test]
fn a_new_action_clears_the_redo_branch() {
    let mut r = r1();
    form(&mut r, 1, &["a0_b1", "a1_b0", "c0_d1"]);
    assert!(r.undo());
    form(&mut r, 2, &["a0_b2", "a1_b1"]);
    assert!(!r.redo());
}

#[test]
fn origins_point_to_the_right_cells() {
    let mut r = r1();
    replay_r2_tree(&mut r);
    for p in r.points() {
        match p.origin {
            Origin::Input { dot } => assert!(dot < 20),
            Origin::Sum { cell } => assert_eq!(r.signal(p), format!("s{cell}")),
            Origin::Carry { cell } => assert_eq!(r.signal(p), format!("k{cell}")),
        }
    }
}
