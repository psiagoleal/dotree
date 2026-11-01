// Caminho relativo: app/core/tests/diagram.rs
//! Spec 05 — block diagram layout and wiring table (MT-9).

mod common;

use std::collections::{BTreeMap, BTreeSet};

use common::{Lcg, r1, reduce_at_random, replay_r2_tree};
use dotree_core::diagram::{
    Block, Endpoint, Layout, TableHeaders, wiring_table, wiring_table_markdown,
};
use dotree_core::expr::analyze;
use dotree_core::matrix::DotMatrix;
use dotree_core::netlist::{Netlist, Wire};
use dotree_core::reduce::{CellKind, Reduction};
use proptest::prelude::*;

/// The spec itself: DIA-04 compares against the table written in spec 05.
const SPEC_05: &str = include_str!("../../../docs/specs/05-diagrama.md");

const PT_HEADERS: TableHeaders = TableHeaders {
    cell: "Célula",
    level: "Nível",
    column: "Coluna",
    sum: "Soma",
    carry: "Vai-um",
};

fn r1_matrix() -> DotMatrix {
    let w: BTreeMap<char, u32> = [
        ('A', 3),
        ('B', 3),
        ('C', 2),
        ('D', 2),
        ('E', 2),
        ('F', 1),
        ('G', 1),
    ]
    .into_iter()
    .collect();
    DotMatrix::build(&analyze("A*B + C*D + 3(E+F) + G", &w).unwrap()).unwrap()
}

fn r2() -> (Reduction, Netlist) {
    let mut r = r1();
    replay_r2_tree(&mut r);
    r.close_with_cpa().unwrap();
    let n = Netlist::from_reduction(&r1_matrix(), &r).unwrap();
    (r, n)
}

/// The reference table of spec 05: from its header line to the first blank line.
fn spec_table() -> String {
    let start = SPEC_05
        .find("| Célula |")
        .expect("spec 05 has the reference table");
    let rest = &SPEC_05[start..];
    let end = rest.find("\n\n").unwrap_or(rest.len());
    rest[..end].to_string()
}

// ---------------------------------------------------------------- DIA-04

#[test]
fn dia04_r2_wiring_table_is_the_one_in_spec_05() {
    let (_, n) = r2();
    let md = wiring_table_markdown(&wiring_table(&n), PT_HEADERS);
    assert_eq!(md.trim_end(), spec_table());
}

#[test]
fn dia04_rows_carry_kinds_levels_and_results() {
    let (_, n) = r2();
    let rows = wiring_table(&n);
    assert_eq!(rows.len(), 17);
    assert_eq!(rows[2].cell, "HA3");
    assert_eq!(rows[2].inputs[2], None);
    assert_eq!(rows[7].level, Some(2));
    assert_eq!(rows[11].level, None); // FA12 is in the CPA
    assert_eq!(rows[11].sum_result, Some(0));
    assert_eq!(rows[16].carry_result, Some(6)); // k17 is R(6)
    assert_eq!(rows[0].sum_result, None);
}

// ---------------------------------------------------------------- DIA-01

#[test]
fn dia01_r2_block_positions() {
    let (_, n) = r2();
    let l = Layout::new(&n);
    assert_eq!(l.rows, 5);
    assert_eq!(l.output_width, 7);
    let got: Vec<(usize, usize, u32, usize)> = l
        .blocks
        .iter()
        .map(|b| (b.cell, b.row, b.column, b.slot))
        .collect();
    assert_eq!(
        got,
        [
            (1, 1, 1, 0),
            (2, 1, 1, 1),
            (3, 1, 2, 0),
            (4, 2, 1, 0),
            (5, 2, 2, 0),
            (6, 2, 2, 1),
            (7, 2, 3, 0),
            (8, 3, 0, 0),
            (9, 3, 2, 0),
            (10, 3, 3, 0),
            (11, 3, 4, 0),
            (12, 4, 0, 0),
            (13, 4, 1, 0),
            (14, 4, 2, 0),
            (15, 4, 3, 0),
            (16, 4, 4, 0),
            (17, 4, 5, 0),
        ]
    );
    assert_eq!(
        l.blocks[2],
        Block {
            cell: 3,
            kind: CellKind::Ha,
            row: 1,
            column: 2,
            slot: 0
        }
    );
}

// ---------------------------------------------------------------- DIA-02, DIA-03

#[test]
fn dia03_every_wire_is_labeled_once_per_use() {
    let (_, n) = r2();
    let l = Layout::new(&n);
    assert_eq!(l.and_gates, 13);
    // 13 FA x 3 inputs + 4 HA x 2 inputs + 7 result bits.
    assert_eq!(l.edges.len(), 13 * 3 + 4 * 2 + 7);
    assert!(
        l.edges.iter().any(|e| e.from == Endpoint::Cell(15)
            && e.to == Endpoint::Cell(16)
            && e.label == "k15")
    );
    assert!(
        l.edges
            .iter()
            .any(|e| e.from == Endpoint::Inputs && e.to == Endpoint::Cell(1) && e.label == "a0_b1")
    );
    assert!(
        l.edges.iter().any(|e| e.from == Endpoint::Cell(17)
            && e.to == Endpoint::Output(6)
            && e.label == "k17")
    );
}

#[test]
fn dia03_identifiers_match_the_netlist() {
    // Built independently from the netlist: every cell input, every cell output that
    // exists, and R(0)..R(W-1). The VHDL generator (MT-10) must declare the same set.
    let (_, n) = r2();
    let mut expected = BTreeSet::new();
    for c in &n.cells {
        expected.extend(c.inputs.iter().map(|&w| n.ident(w)));
        expected.insert(n.ident(Wire::Sum(c.number)));
        if c.has_carry {
            expected.insert(n.ident(Wire::Carry(c.number)));
        }
    }
    expected.extend((0..n.output_width).map(|j| format!("R({j})")));
    assert_eq!(Layout::new(&n).identifiers(), expected);
}

#[test]
fn outputs_name_each_result_bit() {
    let (_, n) = r2();
    let outs: Vec<String> = Layout::new(&n)
        .outputs
        .into_iter()
        .map(|o| o.unwrap())
        .collect();
    assert_eq!(outs, ["s12", "s13", "s14", "s15", "s16", "s17", "k17"]);
}

#[test]
fn constant_zero_output_and_no_cells() {
    let w: BTreeMap<char, u32> = [('A', 2)].into_iter().collect();
    let m = DotMatrix::build(&analyze("2A", &w).unwrap()).unwrap();
    let n = Netlist::from_reduction(&m, &Reduction::new(&m)).unwrap();
    let l = Layout::new(&n);
    assert_eq!(l.outputs, [None, Some("a0".into()), Some("a1".into())]);
    assert!(l.blocks.is_empty());
    assert_eq!(l.rows, 1);
    assert_eq!(l.edges.len(), 2);
    assert!(l.critical.is_empty());
}

// ---------------------------------------------------------------- DIA-05

#[test]
fn dia05_r2_critical_cells_agree_with_the_reduction() {
    let (r, n) = r2();
    let l = Layout::new(&n);
    assert_eq!(l.critical, vec![1, 2, 4, 6, 8, 9, 12, 13, 14, 15, 16, 17]);
    assert_eq!(l.critical, r.critical().cells);
}

proptest! {
    // No regressions file: mutation runs used to leave cases that only fail on mutated code.
    // A real failure is printed, already shrunk, in the test output.
    #![proptest_config(ProptestConfig {
        cases: 64,
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    /// Two independent computations of the critical cells (reduction state vs. netlist
    /// timing) must always agree.
    #[test]
    fn dia05_critical_cells_agree_on_random_reductions(
        a in 1u32..=4, b in 1u32..=4, c in 1u32..=3, seed in any::<u64>(),
    ) {
        let w: BTreeMap<char, u32> = [('A', a), ('B', b), ('C', c)].into_iter().collect();
        let m = DotMatrix::build(&analyze("A*B + 3C", &w).unwrap()).unwrap();
        let mut r = Reduction::new(&m);
        reduce_at_random(&mut r, &mut Lcg(seed));
        let n = Netlist::from_reduction(&m, &r).unwrap();
        prop_assert_eq!(Layout::new(&n).critical, r.critical().cells);
    }
}

// ---------------------------------------------------------------- DIA-07

#[test]
fn dia07_partial_diagram_up_to_the_last_concluded_level() {
    let mut r = r1();
    replay_r2_tree(&mut r);
    let n = Netlist::partial(&r1_matrix(), &r);
    let l = Layout::new(&n);
    assert!(!l.complete);
    assert_eq!(l.blocks.len(), 11);
    assert_eq!(l.rows, 4);
    assert!(l.outputs.is_empty());
    assert!(l.edges.iter().all(|e| !matches!(e.to, Endpoint::Output(_))));
    assert_eq!(l.critical, r.critical().cells);
}
