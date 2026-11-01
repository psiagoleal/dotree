// Caminho relativo: app/core/tests/verify.rs
//! Spec 04 — verification by simulation (MT-7).

mod common;

use std::collections::BTreeMap;

use common::{Lcg, r1, reduce_at_random, reduction, replay_r2_tree};
use dotree_core::expr::analyze;
use dotree_core::matrix::DotMatrix;
use dotree_core::netlist::{Netlist, NetlistError, Wire};
use dotree_core::reduce::Reduction;
use dotree_core::verify::{Coverage, SAMPLE_SIZE, Verifier, sample_vectors};
use proptest::prelude::*;

const R1_SRC: &str = "A*B + C*D + 3(E+F) + G";
const R1_WIDTHS: [(char, u32); 7] = [
    ('A', 3),
    ('B', 3),
    ('C', 2),
    ('D', 2),
    ('E', 2),
    ('F', 1),
    ('G', 1),
];

fn r1_matrix() -> DotMatrix {
    let w: BTreeMap<char, u32> = R1_WIDTHS.into_iter().collect();
    DotMatrix::build(&analyze(R1_SRC, &w).unwrap()).unwrap()
}

fn r2_netlist() -> Netlist {
    let mut r = r1();
    replay_r2_tree(&mut r);
    r.close_with_cpa().unwrap();
    Netlist::from_reduction(&r1_matrix(), &r).unwrap()
}

/// The R1 expression evaluated directly, independent of the core.
fn r1_value(v: &BTreeMap<char, u64>) -> u128 {
    let x = |c: char| u128::from(v[&c]);
    x('A') * x('B') + x('C') * x('D') + 3 * (x('E') + x('F')) + x('G')
}

fn trivial(src: &str, widths: &[(char, u32)]) -> Netlist {
    let w: BTreeMap<char, u32> = widths.iter().copied().collect();
    let m = DotMatrix::build(&analyze(src, &w).unwrap()).unwrap();
    let r = Reduction::new(&m);
    Netlist::from_reduction(&m, &r).unwrap()
}

// ---------------------------------------------------------------- netlist

#[test]
fn r2_netlist_shape() {
    let n = r2_netlist();
    assert_eq!(n.cells.len(), 17);
    assert_eq!(n.input_bits(), 14);
    let bits: Vec<String> = n.result.iter().map(|w| n.ident(w.unwrap())).collect();
    assert_eq!(bits, ["s12", "s13", "s14", "s15", "s16", "s17", "k17"]);
    let fa16: Vec<String> = n.cells[15].inputs.iter().map(|&w| n.ident(w)).collect();
    assert_eq!(fa16, ["s11", "k10", "k15"]);
    assert!(n.cells.iter().enumerate().all(|(i, c)| c.number == i + 1));
}

#[test]
fn ver05_incomplete_circuit_has_no_netlist() {
    assert_eq!(
        Netlist::from_reduction(&r1_matrix(), &r1()),
        Err(NetlistError::NotComplete)
    );
}

#[test]
fn packing_is_alphabetical_with_the_first_variable_lowest() {
    let n = r2_netlist();
    // A=5, B=6, C=2, D=3, E=1, F=1, G=0
    let packed: u64 = 5 | 6 << 3 | 2 << 6 | 3 << 8 | 1 << 10 | 1 << 12;
    let v = n.unpack(packed);
    let expected: BTreeMap<char, u64> = [
        ('A', 5),
        ('B', 6),
        ('C', 2),
        ('D', 3),
        ('E', 1),
        ('F', 1),
        ('G', 0),
    ]
    .into_iter()
    .collect();
    assert_eq!(v, expected);
    assert_eq!(n.expected(packed), 42); // 30 + 6 + 3 * 2 + 0
}

// ---------------------------------------------------------------- VER-01

#[test]
fn ver01_r2_is_correct_for_all_16384_inputs() {
    let p = Verifier::new(r2_netlist()).run_to_end();
    assert_eq!(p.coverage, Coverage::Exhaustive);
    assert_eq!((p.checked, p.total), (16384, 16384));
    assert_eq!(p.counterexample, None);
    assert!(p.done());
}

#[test]
fn ver04_chunks_add_up_to_the_same_result() {
    let mut v = Verifier::new(r2_netlist());
    assert_eq!(v.progress().checked, 0);
    let mut last = 0;
    loop {
        let p = v.run(1000);
        assert!(p.checked > last && p.checked <= p.total);
        last = p.checked;
        if p.done() {
            break;
        }
    }
    assert_eq!(last, 16384);
    assert_eq!(v.progress().counterexample, None);
}

// ---------------------------------------------------------------- VER-03

#[test]
fn ver03_broken_circuit_gives_a_counterexample() {
    // FA16 gets k14 (weight 2^3) where k15 (weight 2^4) belongs.
    let mut n = r2_netlist();
    assert_eq!(n.cells[15].inputs[2], Wire::Carry(15));
    n.cells[15].inputs[2] = Wire::Carry(14);
    let p = Verifier::new(n).run_to_end();
    let ce = p.counterexample.expect("the broken circuit must be caught");
    assert!(p.checked <= p.total);
    assert_eq!(ce.expected, r1_value(&ce.inputs));
    assert_ne!(ce.obtained, ce.expected);
    assert_eq!(ce.differing_bits(), ce.expected ^ ce.obtained);
}

/// Scalar reference simulator, written independently of the bit-parallel verifier.
fn simulate_scalar(n: &Netlist, packed: u64) -> u128 {
    let v = n.unpack(packed);
    let input = |i: usize| -> u64 {
        n.signals[i]
            .factors
            .iter()
            .fold(1, |acc, f| acc & (v[&f.var] >> f.bit) & 1)
    };
    let mut sum = vec![0u64; n.cells.len() + 1];
    let mut carry = vec![0u64; n.cells.len() + 1];
    let get = |w: Wire, sum: &[u64], carry: &[u64]| match w {
        Wire::Input(i) => input(i),
        Wire::Sum(c) => sum[c],
        Wire::Carry(c) => carry[c],
    };
    for c in &n.cells {
        let total: u64 = c.inputs.iter().map(|&w| get(w, &sum, &carry)).sum();
        sum[c.number] = total & 1;
        carry[c.number] = total >> 1;
    }
    n.result
        .iter()
        .enumerate()
        .map(|(j, w)| u128::from(w.map_or(0, |w| get(w, &sum, &carry))) << j)
        .sum()
}

#[test]
fn ver03_reports_the_first_failing_vector_in_order() {
    let mut n = r2_netlist();
    n.cells[15].inputs[2] = Wire::Carry(14);
    let first_bad = (0..1u64 << n.input_bits())
        .find(|&v| simulate_scalar(&n, v) != n.expected(v))
        .expect("the broken circuit fails somewhere");
    let p = Verifier::new(n.clone()).run_to_end();
    let ce = p.counterexample.unwrap();
    assert_eq!(ce.inputs, n.unpack(first_bad));
    assert_eq!(ce.obtained, simulate_scalar(&n, first_bad));
    assert_eq!(p.checked, first_bad + 1);
}

#[test]
fn bit_parallel_and_scalar_simulation_agree_on_r2() {
    let n = r2_netlist();
    for v in [0, 1, 0x1234, 0x3fff] {
        assert_eq!(simulate_scalar(&n, v), n.expected(v));
    }
}

#[test]
fn ver03_swapped_result_bits_are_caught() {
    let mut n = r2_netlist();
    n.result.swap(0, 1);
    assert!(Verifier::new(n).run_to_end().counterexample.is_some());
}

// ---------------------------------------------------------------- VER-02 sample

#[test]
fn ver02_sample_sequence() {
    // Oracle: an independent Python implementation of the spec 04 definition.
    assert_eq!(
        sample_vectors(32, 8),
        [
            0x0,
            0xffff_ffff,
            0x66c2_07f2,
            0xd377_dc2a,
            0x5d21_a664,
            0x325b_e4b3,
            0x6f80_bfb8,
            0xdbd0_79f5
        ]
    );
    assert_eq!(
        sample_vectors(25, 8),
        [
            0x0, 0x1ff_ffff, 0xc2_07f2, 0x177_dc2a, 0x121_a664, 0x5b_e4b3, 0x180_bfb8, 0x1d0_79f5
        ]
    );
}

#[test]
fn ver02_exhaustive_up_to_24_bits_then_sample() {
    // A + 4096B with 12 + 12 bits is complete from the start (no column has two dots).
    let p = Verifier::new(trivial("A + 4096B", &[('A', 12), ('B', 12)])).progress();
    assert_eq!((p.coverage, p.total), (Coverage::Exhaustive, 1 << 24));
    let p = Verifier::new(trivial("A + 8192B", &[('A', 13), ('B', 12)])).progress();
    assert_eq!((p.coverage, p.total), (Coverage::Sample, SAMPLE_SIZE));
}

#[test]
fn ver02_sample_run_on_32_bits() {
    let p = Verifier::new(trivial("A + 65536B", &[('A', 16), ('B', 16)])).run_to_end();
    assert_eq!(p.coverage, Coverage::Sample);
    assert_eq!(p.checked, SAMPLE_SIZE);
    assert_eq!(p.counterexample, None);
}

// ---------------------------------------------------------------- VER-06 property

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// VER-06: any reduction the user can build verifies (correct by construction).
    #[test]
    fn ver06_random_reductions_verify(
        widths in prop::collection::vec(1u32..=3, 4),
        terms in prop::collection::vec((1u32..16, 1u64..=5), 1..=3),
        seed in any::<u64>(),
    ) {
        let names = ['A', 'B', 'C', 'D'];
        let src = terms
            .iter()
            .map(|&(mask, coef)| {
                let vars: String = (0..4).filter(|i| mask >> i & 1 == 1).map(|i| names[i]).collect();
                format!("{coef}{vars}")
            })
            .collect::<Vec<_>>()
            .join(" + ");
        let used: Vec<(char, u32)> = (0..4)
            .filter(|i| terms.iter().any(|&(m, _)| m >> i & 1 == 1))
            .map(|i| (names[i], widths[i]))
            .collect();
        let w: BTreeMap<char, u32> = used.iter().copied().collect();
        let m = DotMatrix::build(&analyze(&src, &w).unwrap()).unwrap();
        let mut r = Reduction::new(&m);
        reduce_at_random(&mut r, &mut Lcg(seed));
        let n = Netlist::from_reduction(&m, &r).unwrap();
        let bits = n.input_bits();
        let p = Verifier::new(n).run_to_end();
        prop_assert_eq!(p.counterexample, None, "{}", src);
        prop_assert_eq!(p.checked, 1u64 << bits);
    }
}

#[test]
fn reference_r1_reduction_helper_still_works() {
    // Guards the shared helper used above.
    let r = reduction("A", &[('A', 2)]);
    assert!(r.is_complete());
}

/// Measurement, not a check: `cargo test --release -p dotree-core --test verify -- --ignored
/// --nocapture`. A 12 x 12 multiplier has 24 input bits, the largest exhaustive case.
#[test]
#[ignore]
fn timing_of_the_largest_exhaustive_case() {
    let w: BTreeMap<char, u32> = [('A', 12), ('B', 12)].into_iter().collect();
    let m = DotMatrix::build(&analyze("A*B", &w).unwrap()).unwrap();
    let mut r = Reduction::new(&m);
    reduce_at_random(&mut r, &mut Lcg(7));
    let n = Netlist::from_reduction(&m, &r).unwrap();
    let cells = n.cells.len();
    let start = std::time::Instant::now();
    let p = Verifier::new(n).run_to_end();
    println!(
        "{} vectors, {cells} cells: {:?}",
        p.checked,
        start.elapsed()
    );
    assert_eq!(p.counterexample, None);
}
