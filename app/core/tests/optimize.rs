// Caminho relativo: app/core/tests/optimize.rs
//! MT-46: the optimizer (spec 03, AJU-14). Non-dominated circuits in area and delay, under
//! the costs in force, for an expression or a height profile: an exact search while it fits
//! the limit, a named heuristic beyond it.

use std::collections::BTreeMap;

use dotree_core::expr::analyze;
use dotree_core::help::{Method, solution_with_costs};
use dotree_core::matrix::DotMatrix;
use dotree_core::netlist::Netlist;
use dotree_core::optimize::{Mode, optimize};
use dotree_core::profile::{Profile, parse};
use dotree_core::reduce::{Costs, Reduction};
use dotree_core::verify::Verifier;
use proptest::prelude::*;

fn profile(src: &str) -> Profile {
    parse(src).unwrap().unwrap()
}

fn matrix(src: &str, widths: &[(char, u32)]) -> DotMatrix {
    let w: BTreeMap<char, u32> = widths.iter().copied().collect();
    DotMatrix::build(&analyze(src, &w).unwrap()).unwrap()
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

fn point(r: &Reduction) -> (u32, u32) {
    let m = r.metrics();
    (m.area, m.delay.0)
}

/// `a` dominates `b`: no worse in both, better in one.
fn dominates(a: (u32, u32), b: (u32, u32)) -> bool {
    a.0 <= b.0 && a.1 <= b.1 && a != b
}

fn classical(m: &DotMatrix, costs: Costs) -> Vec<(Method, (u32, u32))> {
    [Method::Dadda, Method::DaddaParhami, Method::Wallace]
        .into_iter()
        .map(|k| (k, point(&solution_with_costs(m, k, costs))))
        .collect()
}

/// Every option complete (or at the profile), verified, ascending in area and strictly
/// descending in delay, none dominated by a classical method.
fn check(m: &DotMatrix, p: Option<&Profile>, costs: Costs, circuits: &[Reduction]) {
    assert!(!circuits.is_empty());
    for r in circuits {
        match p {
            Some(p) => assert!(p.reached(r)),
            None => assert!(r.is_complete()),
        }
        assert_eq!(r.costs(), costs);
        if r.is_complete() {
            let n = Netlist::from_reduction(m, r).unwrap();
            assert_eq!(Verifier::new(n).run_to_end().counterexample, None);
        }
    }
    for w in circuits.windows(2) {
        let (a, b) = (point(&w[0]), point(&w[1]));
        assert!(a.0 < b.0 && a.1 > b.1, "{a:?} then {b:?}");
    }
    for (method, c) in classical(m, costs) {
        for r in circuits {
            assert!(
                !dominates(c, point(r)),
                "{method:?} {c:?} dominates {:?}",
                point(r)
            );
        }
    }
}

const PROBLEM_8_3: [&str; 11] = [
    "{2,2,3; 1,1,1,1}",
    "{3; 1,1}",
    "{1,4,3; 1,1,1,1}",
    "{5,5; 1,1,1,1}",
    "{2,2; 1,1,1}",
    "{5; 1,1,1}",
    "{7; 1,1,1}",
    "{3,3,3; 1,2,2,1}",
    "{4,7; 1,1,1,1}",
    "{2,5; 1,2,1}",
    "{5; 2,1}",
];

#[test]
fn aju14_on_the_reference_profiles_the_least_area_option_is_the_optimum() {
    for src in PROBLEM_8_3 {
        let p = profile(src);
        let m = p.matrix();
        let o = optimize(&m, Some(&p), Costs::default());
        assert_eq!(o.mode, Mode::Exact, "{src}");
        let best = p.optimal().unwrap();
        assert_eq!(point(&o.circuits[0]), point(&best), "{src}");
        check(&m, Some(&p), Costs::default(), &o.circuits);
    }
}

#[test]
fn aju14_the_reference_profiles_under_other_costs() {
    // e) by hand (AJU-12): with HA 0.3, 4 HA (1.2, 0.9) beat FA + HA (1.3, 1.3).
    let costs = Costs {
        ha_area: 30,
        ha_delay: 30,
        ..Costs::default()
    };
    for src in PROBLEM_8_3 {
        let p = profile(src);
        let m = p.matrix();
        let o = optimize(&m, Some(&p), costs);
        // The optimal search of AJU-12 gives up on some of them under these costs.
        if let Ok(best) = p.optimal_with_costs(costs) {
            assert_eq!(point(&o.circuits[0]), point(&best), "{src}");
        }
        check(&m, Some(&p), costs, &o.circuits);
    }
    let p = profile("{2,2; 1,1,1}");
    let o = optimize(&p.matrix(), Some(&p), costs);
    assert_eq!(point(&o.circuits[0]), (120, 90));
}

#[test]
fn aju14_r1_is_searched_exactly_up_to_the_limit() {
    let m = r1();
    let o = optimize(&m, None, Costs::default());
    // The search reaches its limit at 16 A_FA; up to 15.5 the front is exact: Wallace's
    // area and delay (14.5, 6, AJU-11), and one more HA buys half a T_FA. The circuits of
    // 16 A_FA and more are the heuristic's, all dominated by these two.
    assert_eq!(o.mode, Mode::ExactUpTo(1550));
    let points: Vec<(u32, u32)> = o.circuits.iter().map(point).collect();
    assert_eq!(points, vec![(1450, 600), (1500, 550)]);
    check(&m, None, Costs::default(), &o.circuits);
    // The lower bound (AJU-06): 13 FA at least, so no option is below 13 A_FA.
    assert!(o.circuits.iter().all(|r| r.metrics().area >= 1300));
}

#[test]
fn aju14_the_32_by_32_uses_the_heuristic() {
    let m = matrix("A*B", &[('A', 32), ('B', 32)]);
    let o = optimize(&m, None, Costs::default());
    assert_eq!(o.mode, Mode::Heuristic);
    for r in &o.circuits {
        assert!(r.is_complete());
    }
    for (method, c) in classical(&m, Costs::default()) {
        for r in &o.circuits {
            assert!(!dominates(c, point(r)), "{method:?}");
        }
    }
}

#[test]
fn aju14_the_largest_profile_uses_the_heuristic() {
    let p = profile("{6,7,7,6; 1,1,1,1,1,1,1}");
    let m = p.matrix();
    let o = optimize(&m, Some(&p), Costs::default());
    assert_eq!(o.mode, Mode::Heuristic);
    check(&m, Some(&p), Costs::default(), &o.circuits);
}

#[test]
fn aju14_cheap_counters_are_used() {
    // A 7:3 for the price of 2 FA: seven bits are counted by one counter.
    let costs = Costs {
        c73_area: Some(200),
        ..Costs::default()
    };
    let p = profile("{7; 1,1,1}");
    let m = p.matrix();
    let o = optimize(&m, Some(&p), costs);
    check(&m, Some(&p), costs, &o.circuits);
    assert_eq!(point(&o.circuits[0]), (200, 300));
    assert_eq!(o.circuits[0].metrics().c73, 1);
    // Under the default costs a counter costs what its FA network costs and is not tried.
    let o = optimize(&m, Some(&p), Costs::default());
    assert!(o.circuits.iter().all(|r| r.metrics().c73 == 0));
}

#[test]
fn aju14_a_profile_met_by_its_input_needs_no_cell() {
    let p = profile("{2; 2}");
    let o = optimize(&p.matrix(), Some(&p), Costs::default());
    assert_eq!(o.circuits.len(), 1);
    assert_eq!(point(&o.circuits[0]), (0, 0));
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 24, .. ProptestConfig::default() })]

    /// Small random profiles with a one-bit output: the least-area option is the optimum of
    /// AJU-12, a search with bounds of its own.
    #[test]
    fn aju14_the_least_area_option_is_the_optimum_of_aju12(
        heights in proptest::collection::vec(0u32..=4, 1..=3),
    ) {
        let value: u32 = heights.iter().enumerate().map(|(c, &h)| h << c).sum();
        prop_assume!(value > 0);
        let width = 32 - value.leading_zeros();
        let input: Vec<String> = heights.iter().rev().map(u32::to_string).collect();
        let output = vec!["1"; width as usize].join(",");
        let p = profile(&format!("{{{}; {output}}}", input.join(",")));
        let m = p.matrix();
        let o = optimize(&m, Some(&p), Costs::default());
        check(&m, Some(&p), Costs::default(), &o.circuits);
        if let (Ok(best), Mode::Exact) = (p.optimal(), o.mode) {
            prop_assert_eq!(point(&o.circuits[0]), point(&best));
        }
    }

    /// Small random expressions: every option verifies and none is dominated by a method.
    #[test]
    fn aju14_options_are_never_dominated_by_a_method(
        wa in 1u32..=3, wb in 1u32..=3, wc in 1u32..=3, k in 1u64..=5,
    ) {
        let m = matrix(&format!("A*B + {k}C"), &[('A', wa), ('B', wb), ('C', wc)]);
        let o = optimize(&m, None, Costs::default());
        check(&m, None, Costs::default(), &o.circuits);
    }
}
