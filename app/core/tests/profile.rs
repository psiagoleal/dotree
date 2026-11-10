// Caminho relativo: app/core/tests/profile.rs
//! Spec 01 (EXP-16, EXP-17), 02 (CMP-19) and 03 (AJU-12): input by column heights with a
//! target output profile, and its optimal circuit (MT-31a).

use dotree_core::profile::{Block, Profile, ProfileError, SearchError, parse};
use dotree_core::reduce::{CellKind, Time};

fn profile(src: &str) -> Profile {
    parse(src).unwrap().unwrap()
}

#[test]
fn exp16_heights_are_written_from_the_most_significant_column() {
    let p = profile("{2,2,3; 1,1,1,1}");
    assert_eq!(p.input, vec![3, 2, 2]);
    assert_eq!(p.output, vec![1, 1, 1, 1]);
    // Blanks are free, as in the expression.
    assert_eq!(profile("  { 3 ;1 , 1 }  "), profile("{3;1,1}"));
}

#[test]
fn exp16_an_expression_is_not_a_profile() {
    assert_eq!(parse("A*B + C"), Ok(None));
    assert_eq!(parse("  A"), Ok(None));
}

#[test]
fn exp16_each_dot_is_an_independent_one_bit_variable() {
    // The plan's example: {2,2,3; …} is A+B+C+2D+2E+4F+4G with every width 1.
    let (expression, widths) = profile("{2,2,3; 1,1,1,1}").expression();
    assert_eq!(expression, "A + B + C + 2D + 2E + 4F + 4G");
    assert_eq!(widths.len(), 7);
    assert!(widths.values().all(|&w| w == 1));
    let m = profile("{2,2,3; 1,1,1,1}").matrix();
    assert_eq!(m.heights(), vec![3, 2, 2, 0]);
    assert_eq!(m.output_width, 4);
    assert_eq!(m.and_gates(), 0);
}

#[test]
fn exp17_malformed_profiles_point_at_the_column() {
    use ProfileError::Syntax;
    assert_eq!(parse("{2,2,3 1,1}"), Err(Syntax { column: 8 }));
    assert_eq!(parse("{2,2,3; 1,1"), Err(Syntax { column: 12 }));
    assert_eq!(parse("{2,,3; 1}"), Err(Syntax { column: 4 }));
    assert_eq!(parse("{;1}"), Err(Syntax { column: 2 }));
    assert_eq!(parse("{3;1,1} A"), Err(Syntax { column: 9 }));
    assert_eq!(parse("{3;1,x}"), Err(Syntax { column: 6 }));
}

#[test]
fn exp17_limits_and_impossible_outputs_are_refused() {
    assert_eq!(parse("{0,0; 1}"), Err(ProfileError::NoDots));
    assert_eq!(
        parse("{9,9,9; 1,1,1,1,1,1}"),
        Err(ProfileError::TooManyDots {
            dots: 27,
            limit: 26
        })
    );
    assert_eq!(
        parse("{1; 1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1}"),
        Err(ProfileError::TooManyColumns { limit: 16 })
    );
    // 7 ones need 3 bits; {1,1} holds at most 3.
    assert_eq!(
        parse("{7; 1,1}"),
        Err(ProfileError::OutputTooSmall { column: 0 })
    );
    // Points never move to a lower column: column 1 starts with weight 2, and the output
    // has nothing from column 1 up.
    assert_eq!(
        parse("{1,1; 3}"),
        Err(ProfileError::OutputTooSmall { column: 1 })
    ); // A column with a point never empties: every cell leaves its sum in it.
    assert_eq!(
        parse("{2; 2,0}"),
        Err(ProfileError::EmptyColumn { column: 0 })
    );
    // Column 0 (4 dots, target 1) needs 2 cells, whose carries take column 1 to 4 points,
    // over its target 3: a carry reaches column 2, whose target is 0.
    assert_eq!(
        parse("{2,4; 1,0,3,1}"),
        Err(ProfileError::EmptyColumn { column: 2 })
    );
}

#[test]
fn cmp19_the_profile_is_reached_when_no_column_exceeds_its_target() {
    // h) {3,3,3; 1,2,2,1}: one FA per column on level 0 reaches it; nothing before.
    let p = profile("{3,3,3; 1,2,2,1}");
    let mut r = dotree_core::reduce::Reduction::new(&p.matrix());
    assert!(!p.reached(&r));
    for column in 0..3 {
        let ids: Vec<usize> = r
            .points()
            .iter()
            .filter(|q| q.column == column)
            .map(|q| q.id)
            .collect();
        r.form_cell(&ids).unwrap();
        // Formed but not concluded: not reached yet.
        assert!(!p.reached(&r));
    }
    r.conclude_level().unwrap();
    assert!(p.reached(&r));
    assert!(
        !r.is_complete(),
        "a redundant output is not a complete circuit"
    );
}

#[test]
fn cmp19_a_pending_cell_leaves_the_level_open() {
    // {2; 2} is met by the input itself; an HA formed over it, not concluded, would move a
    // carry to column 1, whose target is 0.
    let p = profile("{2; 2}");
    let mut r = dotree_core::reduce::Reduction::new(&p.matrix());
    assert!(p.reached(&r));
    let ids: Vec<usize> = r.points().iter().map(|q| q.id).collect();
    r.form_cell(&ids).unwrap();
    assert!(!p.reached(&r));
    r.conclude_level().unwrap();
    assert!(!p.reached(&r));
}

#[test]
fn cmp19_columns_beyond_the_profile_must_be_empty() {
    // k) {5; 2,1}: W = 3, but the profile has no column 2.
    let p = profile("{5; 2,1}");
    assert_eq!(p.matrix().output_width, 3);
    let mut r = dotree_core::reduce::Reduction::new(&p.matrix());
    // FA, then FA over its sum and the two dots left: column 0 = 1, column 1 = 2.
    let first: Vec<usize> = r.points().iter().take(3).map(|q| q.id).collect();
    r.form_cell(&first).unwrap();
    r.conclude_level().unwrap();
    let col0: Vec<usize> = r
        .points()
        .iter()
        .filter(|q| q.column == 0)
        .map(|q| q.id)
        .collect();
    r.form_cell(&col0).unwrap();
    r.conclude_level().unwrap();
    assert_eq!(r.heights(), vec![1, 2, 0]);
    assert!(p.reached(&r));
    // A full adder of the two carries would put a point in column 2: over the profile.
    let col1: Vec<usize> = r
        .points()
        .iter()
        .filter(|q| q.column == 1)
        .map(|q| q.id)
        .collect();
    r.form_cell(&col1).unwrap();
    r.conclude_level().unwrap();
    assert_eq!(r.heights(), vec![1, 1, 1]);
    assert!(!p.reached(&r));
}

/// The project's reference set of eleven profiles, a) to k), with area and delay in A_FA
/// and T_FA, worked out by hand and confirmed by an independent exhaustive search.
const PROBLEM_8_3: [(&str, f64, f64); 11] = [
    ("{2,2,3; 1,1,1,1}", 3.0, 3.0),
    ("{3; 1,1}", 1.0, 1.0),
    ("{1,4,3; 1,1,1,1}", 4.0, 3.0),
    ("{5,5; 1,1,1,1}", 6.0, 4.0),
    ("{2,2; 1,1,1}", 1.5, 1.5),
    ("{5; 1,1,1}", 2.5, 2.5),
    ("{7; 1,1,1}", 4.0, 3.0),
    ("{3,3,3; 1,2,2,1}", 3.0, 1.0),
    ("{4,7; 1,1,1,1}", 7.0, 4.0),
    ("{2,5; 1,2,1}", 3.0, 2.0),
    ("{5; 2,1}", 2.0, 2.0),
];

#[test]
fn aju12_the_optimal_circuit_reproduces_problem_8_3() {
    for (src, area, delay) in PROBLEM_8_3 {
        let p = profile(src);
        let r = p.optimal().unwrap_or_else(|e| panic!("{src}: {e:?}"));
        assert!(p.reached(&r), "{src}: the optimum reaches its profile");
        let m = r.metrics();
        assert_eq!((m.area_a_fa(), m.delay.t_fa()), (area, delay), "{src}");
        assert!(r.pending().is_empty(), "{src}: every level concluded");
        assert!(r.levels().iter().all(|l| !l.cpa), "{src}: no CPA level");
    }
}

#[test]
fn aju12_the_optimum_is_minimal_in_area_then_delay() {
    // f) {5; 1,1,1}: 2 FA and 1 HA, the HA last, on the two carries.
    let r = profile("{5; 1,1,1}").optimal().unwrap();
    let cells: Vec<(usize, CellKind, u32)> = r
        .levels()
        .iter()
        .flat_map(|l| l.cells.iter().map(|c| (c.level, c.kind, c.column)))
        .collect();
    assert_eq!(
        cells,
        vec![
            (0, CellKind::Fa, 0),
            (1, CellKind::Fa, 0),
            (2, CellKind::Ha, 1)
        ]
    );
    assert_eq!(r.metrics().delay, Time(5));
}

#[test]
fn aju12_a_profile_already_met_needs_no_cell() {
    let p = profile("{2; 2}");
    let r = p.optimal().unwrap();
    assert!(r.levels().is_empty());
    assert_eq!(r.metrics().area_halves(), 0);
    assert!(p.reached(&r));
}

#[test]
fn aju12_a_search_too_large_stops_instead_of_hanging() {
    // 26 dots in four columns: the search passes its limit and says so.
    assert_eq!(
        profile("{6,7,7,6; 1,1,1,1,1,1,1}").optimal().err(),
        Some(SearchError::TooLarge)
    );
}

#[test]
fn aju12_known_blocks_of_problem_8_3() {
    let block = |s: &str| profile(s).block();
    assert_eq!(block("{3; 1,1}"), Block::FullAdder);
    assert_eq!(block("{2; 1,1}"), Block::HalfAdder);
    assert_eq!(block("{2,2; 1,1,1}"), Block::Adder { bits: 2 });
    assert_eq!(
        block("{2,2,3; 1,1,1,1}"),
        Block::AdderWithCarryIn { bits: 3 }
    );
    assert_eq!(block("{3,3,3; 1,2,2,1}"), Block::CarrySave { bits: 3 });
    assert_eq!(block("{7; 1,1,1}"), Block::Counter);
    assert_eq!(block("{5,5; 1,1,1,1}"), Block::Counter);
    assert_eq!(block("{1,4,3; 1,1,1,1}"), Block::Counter);
    assert_eq!(block("{5; 2,1}"), Block::Compressor);
    assert_eq!(block("{2,5; 1,2,1}"), Block::Compressor);
    // Leading zeros change nothing.
    assert_eq!(block("{0,3; 0,1,1}"), Block::FullAdder);
}

/// (area, delay) in halves, compared in that order.
fn cost(r: &dotree_core::reduce::Reduction) -> (u32, u32) {
    let m = r.metrics();
    (m.area_halves(), m.delay.0)
}

proptest::proptest! {
    #![proptest_config(proptest::prelude::ProptestConfig::with_cases(64))]

    /// Every small profile the parser accepts has an optimal circuit: the parser already
    /// refuses the profiles no circuit reaches (EXP-17).
    #[test]
    fn aju12_every_accepted_profile_has_an_optimum(
        heights in proptest::collection::vec(0u32..=3, 1..=3),
        targets in proptest::collection::vec(0u32..=3, 1..=5),
    ) {
        let list = |v: &[u32]| v.iter().map(u32::to_string).collect::<Vec<_>>().join(",");
        if let Ok(Some(p)) = parse(&format!("{{{}; {}}}", list(&heights), list(&targets))) {
            let best = p.optimal();
            proptest::prop_assert!(best.is_ok(), "{:?}", best.err());
            proptest::prop_assert!(p.reached(&best.unwrap()));
        }
    }

    /// With one bit per output column, Dadda's and Wallace's complete circuits reach the
    /// profile too, so the optimum is never worse than either in (area, delay).
    #[test]
    fn aju12_the_optimum_never_loses_to_dadda_or_wallace(
        heights in proptest::collection::vec(0u32..=4, 1..=3)
    ) {
        proptest::prop_assume!(heights.iter().sum::<u32>() >= 1);
        let input: Vec<String> = heights.iter().rev().map(u32::to_string).collect();
        let value: u32 = heights.iter().enumerate().map(|(c, &h)| h << c).sum();
        let bits = 32 - value.leading_zeros();
        let output = vec!["1"; bits as usize].join(",");
        let p = profile(&format!("{{{}; {output}}}", input.join(",")));
        let best = p.optimal().unwrap();
        proptest::prop_assert!(p.reached(&best));
        for method in [dotree_core::help::Method::Dadda, dotree_core::help::Method::Wallace] {
            let other = dotree_core::help::solution(&p.matrix(), method);
            proptest::prop_assert!(p.reached(&other));
            proptest::prop_assert!(cost(&best) <= cost(&other), "{method:?}");
        }
    }
}

#[test]
fn exp16_the_input_is_an_expression_or_a_profile() {
    use dotree_core::profile::{InputError, analyze_input};
    use std::collections::BTreeMap;
    // A profile ignores the widths: its variables are its own, all of 1 bit.
    let widths: BTreeMap<char, u32> = [('A', 3)].into_iter().collect();
    let input = analyze_input("{2,2,3; 1,1,1,1}", &widths).unwrap();
    assert_eq!(input.profile, Some(profile("{2,2,3; 1,1,1,1}")));
    assert_eq!(input.analysis.widths.len(), 7);
    assert!(input.analysis.warnings.is_empty());
    // An expression goes through the expression rules.
    let input = analyze_input("A + 1", &widths).unwrap();
    assert_eq!(input.profile, None);
    assert_eq!(input.analysis.widths, widths);
    assert!(matches!(
        analyze_input("A +", &widths),
        Err(InputError::Expr(_))
    ));
    assert_eq!(
        analyze_input("{7; 1,1}", &widths).err(),
        Some(InputError::Profile(ProfileError::OutputTooSmall {
            column: 0
        }))
    );
}
