// Caminho relativo: app/core/tests/matrix.rs
//! Spec 01 — dot matrix (MT-4). Reference cases R1, R3, R4 from spec 00.

use std::collections::BTreeMap;

use dotree_core::expr::analyze;
use dotree_core::matrix::{DotMatrix, Factor, MAX_DOTS, MatrixError, Signal};

fn matrix(src: &str, widths: &[(char, u32)]) -> DotMatrix {
    let w: BTreeMap<char, u32> = widths.iter().copied().collect();
    let analysis = analyze(src, &w).unwrap_or_else(|e| panic!("{src:?}: {e:?}"));
    DotMatrix::build(&analysis).unwrap_or_else(|e| panic!("{src:?}: {e:?}"))
}

fn r1() -> DotMatrix {
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

/// Identifiers of the dots of one column, in canonical order.
fn column_idents(m: &DotMatrix, column: u32) -> Vec<String> {
    m.dots
        .iter()
        .filter(|d| d.column == column)
        .map(|d| m.signals[d.signal].ident())
        .collect()
}

// ---------------------------------------------------------------- R1 (spec 00)

#[test]
fn r1_counts_heights_and_width() {
    let m = r1();
    assert_eq!(m.dots.len(), 20);
    assert_eq!(m.heights(), vec![5, 7, 5, 2, 1, 0, 0]);
    assert_eq!(m.v_max, 71);
    assert_eq!(m.output_width, 7);
    assert_eq!(m.and_gates(), 13);
}

/// The path from inputs to v_max shown in step 2 of the width challenge (GER-03): each
/// input at its maximum, each term at its maximum, and their sum.
#[test]
fn r1_v_max_by_term_and_input() {
    let m = r1();
    let inputs: Vec<(char, u128)> = m.widths.keys().map(|&v| (v, m.input_max(v))).collect();
    assert_eq!(
        inputs,
        [
            ('A', 7),
            ('B', 7),
            ('C', 3),
            ('D', 3),
            ('E', 3),
            ('F', 1),
            ('G', 1)
        ]
    );
    let terms: Vec<u128> = (0..m.terms.len()).map(|i| m.term_max(i)).collect();
    assert_eq!(terms, [49, 9, 9, 3, 1]);
    assert_eq!(terms.iter().sum::<u128>(), m.v_max);
}

#[test]
fn term_max_with_large_coefficient_and_widths() {
    // Largest coefficient; 32 copies of 8 dots is exactly the 256-dot limit.
    let m = matrix("4294967295A", &[('A', 8)]);
    assert_eq!(m.input_max('A'), 255);
    assert_eq!(m.term_max(0), 4_294_967_295 * 255);
    assert_eq!(m.v_max, m.term_max(0));
}

#[test]
fn r1_terms_are_the_combined_terms() {
    let m = r1();
    let terms: Vec<String> = m
        .terms
        .iter()
        .map(|t| format!("{}{}", t.coefficient, t.vars.iter().collect::<String>()))
        .collect();
    assert_eq!(terms, ["1AB", "1CD", "3E", "3F", "1G"]);
}

#[test]
fn r1_canonical_order_of_each_column() {
    // This order fixes x, y, z of every cell in the R2 wiring table (spec 05).
    let m = r1();
    assert_eq!(column_idents(&m, 0), ["a0_b0", "c0_d0", "e0", "f0", "g0"]);
    assert_eq!(
        column_idents(&m, 1),
        ["a0_b1", "a1_b0", "c0_d1", "c1_d0", "e1", "e0", "f0"]
    );
    assert_eq!(
        column_idents(&m, 2),
        ["a0_b2", "a1_b1", "a2_b0", "c1_d1", "e1"]
    );
    assert_eq!(column_idents(&m, 3), ["a1_b2", "a2_b1"]);
    assert_eq!(column_idents(&m, 4), ["a2_b2"]);
}

#[test]
fn r1_ids_follow_canonical_order() {
    let m = r1();
    for (i, d) in m.dots.iter().enumerate() {
        assert_eq!(d.id, i);
    }
    // AB first (9 dots), then CD (4), then E, 2E, F, 2F, G.
    let shifts_and_terms: Vec<(usize, u32)> =
        m.dots[13..].iter().map(|d| (d.term, d.shift)).collect();
    assert_eq!(
        shifts_and_terms,
        [(2, 0), (2, 0), (2, 1), (2, 1), (3, 0), (3, 1), (4, 0)]
    );
}

#[test]
fn exp02_copies_share_signals() {
    // The four dots of E and 2E carry only the signals e0 and e1 (EXP-02).
    let m = r1();
    let mut e_signals: Vec<String> = m
        .dots
        .iter()
        .filter(|d| d.term == 2)
        .map(|d| m.signals[d.signal].ident())
        .collect();
    assert_eq!(e_signals.len(), 4);
    e_signals.sort();
    e_signals.dedup();
    assert_eq!(e_signals, ["e0", "e1"]);
    // Signals are distinct: 13 partial products + e0, e1, f0, g0.
    assert_eq!(m.signals.len(), 17);
}

// ---------------------------------------------------------------- R3, R4

#[test]
fn r3_single_variable() {
    let m = matrix("A", &[('A', 4)]);
    assert_eq!(m.heights(), vec![1, 1, 1, 1]);
    assert_eq!(m.output_width, 4);
    assert_eq!(m.v_max, 15);
    assert_eq!(m.and_gates(), 0);
}

#[test]
fn r4_four_by_four_multiplier() {
    let m = matrix("A*B", &[('A', 4), ('B', 4)]);
    assert_eq!(m.dots.len(), 16);
    assert_eq!(m.heights(), vec![1, 2, 3, 4, 3, 2, 1, 0]);
    assert_eq!(m.output_width, 8);
    assert_eq!(m.v_max, 225);
    assert_eq!(m.and_gates(), 16);
}

// ---------------------------------------------------------------- EXP-01 rules

#[test]
fn exp01_coefficient_becomes_shifted_copies() {
    // 3A = A + 2A
    let m = matrix("2A + A", &[('A', 2)]);
    assert_eq!(m.heights(), vec![1, 2, 1, 0]);
    assert_eq!(m.v_max, 9);
    assert_eq!(m.output_width, 4);
}

#[test]
fn exp01_power_of_two_coefficient_is_one_copy() {
    // 4D is a single copy shifted by 2, not four copies.
    let m = matrix("4D", &[('D', 2)]);
    // v_max = 4 x 3 = 12, so W = 4.
    assert_eq!(m.dots.len(), 2);
    assert_eq!(m.heights(), vec![0, 0, 1, 1]);
}

#[test]
fn exp01_empty_low_column_when_everything_is_shifted() {
    let m = matrix("2A", &[('A', 2)]);
    assert_eq!(m.heights(), vec![0, 1, 1]);
    assert_eq!(m.output_width, 3);
}

#[test]
fn exp01_like_terms_and_product_of_sums() {
    // AB + AB = 2AB: coefficient 2 is a single copy shifted by 1 (4 dots, not 8).
    let m = matrix("AB + AB", &[('A', 2), ('B', 2)]);
    assert_eq!(m.dots.len(), 4);
    assert_eq!(m.heights(), vec![0, 1, 2, 1, 0]);
    assert_eq!(m.v_max, 18);
    assert_eq!(m.output_width, 5);

    let m = matrix("(A+B)(C+D)", &[('A', 1), ('B', 1), ('C', 1), ('D', 1)]);
    assert_eq!(m.heights(), vec![4, 0, 0]);
    assert_eq!(m.and_gates(), 4);
}

#[test]
fn exp01_three_variable_product_is_one_and_gate_per_dot() {
    let m = matrix("A*B*C", &[('A', 1), ('B', 2), ('C', 1)]);
    assert_eq!(column_idents(&m, 0), ["a0_b0_c0"]);
    assert_eq!(column_idents(&m, 1), ["a0_b1_c0"]);
    assert_eq!(m.and_gates(), 2);
}

// ---------------------------------------------------------------- EXP-03 AND gates

#[test]
fn exp03_copies_do_not_duplicate_and_gates() {
    let m = matrix("3AB", &[('A', 2), ('B', 2)]);
    assert_eq!(m.dots.len(), 8);
    assert_eq!(m.and_gates(), 4);
}

// ---------------------------------------------------------------- EXP-08 dot limit

#[test]
fn exp08_dot_limit() {
    assert_eq!(MAX_DOTS, 256);
    // 16 x 16 = 256 dots, exactly at the limit.
    let m = matrix("A*B", &[('A', 16), ('B', 16)]);
    assert_eq!(m.dots.len(), 256);
    assert_eq!(m.output_width, 32);

    // 3AB with 16 x 8 bits = 2 x 128 dots, plus C: 257.
    let w: BTreeMap<char, u32> = [('A', 16), ('B', 8), ('C', 1)].into_iter().collect();
    let analysis = analyze("3AB + C", &w).unwrap();
    assert_eq!(
        DotMatrix::build(&analysis),
        Err(MatrixError::TooManyDots {
            count: 257,
            limit: 256
        })
    );
}

// ---------------------------------------------------------------- signals

#[test]
fn signal_identifiers() {
    let s = Signal {
        factors: vec![Factor { var: 'A', bit: 1 }, Factor { var: 'B', bit: 0 }],
    };
    assert_eq!(s.ident(), "a1_b0");
    assert!(s.is_and());
    let s = Signal {
        factors: vec![Factor { var: 'E', bit: 12 }],
    };
    assert_eq!(s.ident(), "e12");
    assert!(!s.is_and());
}
