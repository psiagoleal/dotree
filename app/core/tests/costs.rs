// Caminho relativo: app/core/tests/costs.rs
//! Configurable costs (GER-04, GER-12; MT-45): area and delay of FA and HA in hundredths of
//! `A_FA` and `T_FA`. The default reproduces the fixed convention of v0.2.

mod common;

use common::{Lcg, form, r1, reduce_at_random, reduction, replay_r2_tree};
use dotree_core::reduce::{CellKind, Costs, Reduction, Time};
use proptest::prelude::*;

/// FA 1 / 1, HA 0.5 / 0.5: the convention of spec 00 before MT-45.
#[test]
fn ger04_default_costs_are_the_old_convention() {
    let c = Costs::default();
    assert_eq!(
        (c.fa_area, c.fa_delay, c.ha_area, c.ha_delay),
        (100, 100, 50, 50)
    );
    assert!(c.is_valid());
    assert_eq!(c.area(CellKind::Fa), 100);
    assert_eq!(c.delay(CellKind::Ha), Time(50));
}

#[test]
fn ger12_each_cost_goes_from_0_01_to_100() {
    for (fa_area, fa_delay, ha_area, ha_delay, ok) in [
        (1, 1, 1, 1, true),
        (10_000, 10_000, 10_000, 10_000, true),
        (0, 100, 50, 50, false),
        (100, 0, 50, 50, false),
        (100, 100, 0, 50, false),
        (100, 100, 50, 0, false),
        (10_001, 100, 50, 50, false),
        (100, 100, 50, 10_001, false),
    ] {
        let c = Costs {
            fa_area,
            fa_delay,
            ha_area,
            ha_delay,
            ..Costs::default()
        };
        assert_eq!(c.is_valid(), ok, "{c:?}");
    }
}

#[test]
fn times_are_written_in_hundredths_without_trailing_zeros() {
    for (t, text, t_fa) in [
        (0, "0", 0.0),
        (1, "0.01", 0.01),
        (50, "0.5", 0.5),
        (125, "1.25", 1.25),
        (130, "1.3", 1.3),
        (600, "6", 6.0),
        (1205, "12.05", 12.05),
    ] {
        assert_eq!(Time(t).to_string(), text);
        assert_eq!(Time(t).t_fa(), t_fa);
    }
}

#[test]
fn ger04_r2_with_the_default_costs_keeps_its_numbers() {
    let mut r = r1();
    replay_r2_tree(&mut r);
    r.close_with_cpa().unwrap();
    let m = r.metrics();
    assert_eq!((m.fa, m.ha, m.area, m.delay), (13, 4, 1500, Time(600)));
    assert_eq!(m.area_a_fa(), 15.0);
}

/// `A + B + C + 2(D + E)`, widths 1: column 0 has a0 b0 c0, column 1 has d0 e0; W = 3.
/// Level 0: FA 1 over column 0 (s1, k1) and HA 2 over column 1 (s2, k2). The CPA adds
/// column 1 (s2, k1) with HA 3 and column 2 (k2 and the carry of 3) with HA 4, whose carry
/// falls at W and is dropped.
fn branch_case(costs: Costs) -> Reduction {
    let mut r = reduction(
        "A+B+C+2(D+E)",
        &[('A', 1), ('B', 1), ('C', 1), ('D', 1), ('E', 1)],
    );
    r.set_costs(costs);
    form(&mut r, 0, &["a0", "b0", "c0"]);
    form(&mut r, 1, &["d0", "e0"]);
    r.conclude_level().unwrap();
    r.close_with_cpa().unwrap();
    assert!(r.is_complete());
    r
}

/// By hand, with F and H the delays of FA and HA: k1 is ready at F, s2 and k2 at H.
/// HA 3 is ready at max(F, H) + H and HA 4 at max(H, HA 3) + H.
/// Default (F = 1, H = 0.5): HA 3 at 1.5, HA 4 at 2; k1 is the late input of 3, so the
/// critical cells are 1, 3 and 4.
/// FA faster (F = 0.4, H = 0.5): HA 3 at 0.5 + 0.5 = 1, HA 4 at 1.5; now s2 is the late
/// input of 3, so the critical path goes through HA 2 instead: 2, 3 and 4.
/// Area with FA 1.2 and HA 0.7: 1 FA + 3 HA = 1.2 + 2.1 = 3.3.
#[test]
fn ger12_custom_costs_move_the_critical_path_to_another_branch() {
    let r = branch_case(Costs::default());
    let m = r.metrics();
    assert_eq!((m.fa, m.ha, m.area, m.delay), (1, 3, 250, Time(200)));
    assert_eq!(r.critical().cells, vec![1, 3, 4]);

    let costs = Costs {
        fa_area: 120,
        fa_delay: 40,
        ha_area: 70,
        ha_delay: 50,
        ..Costs::default()
    };
    let r = branch_case(costs);
    let m = r.metrics();
    assert_eq!((m.fa, m.ha, m.area, m.delay), (1, 3, 330, Time(150)));
    assert_eq!(m.area_a_fa(), 3.3);
    assert_eq!(r.critical().cells, vec![2, 3, 4]);
    // k1 (0.4) now comes before s2 (0.5) in the canonical order of level 1's column 1.
    let col1: Vec<String> = r.levels()[1]
        .points
        .iter()
        .filter(|p| p.column == 1)
        .map(|p| r.signal(p))
        .collect();
    assert_eq!(col1, vec!["k1", "s2"]);
    assert_eq!(r.costs(), costs);
}

#[test]
fn ger12_changing_costs_recomputes_readiness_of_every_state() {
    let mut r = branch_case(Costs::default());
    let fast_fa = Costs {
        fa_area: 100,
        fa_delay: 40,
        ha_area: 50,
        ha_delay: 50,
        ..Costs::default()
    };
    r.set_costs(fast_fa);
    assert_eq!(r.metrics().delay, Time(150));
    assert_eq!(r.critical().cells, vec![2, 3, 4]);
    // Undo the CPA: level 1 is current again, with k1 at 0.4 before s2 at 0.5.
    assert!(r.undo());
    let col1: Vec<(String, Time)> = r
        .points()
        .iter()
        .filter(|p| p.column == 1)
        .map(|p| (r.signal(p), p.ready))
        .collect();
    assert_eq!(col1, vec![("k1".into(), Time(40)), ("s2".into(), Time(50))]);
    r.set_costs(Costs::default());
    assert!(r.redo());
    assert_eq!(r.metrics().delay, Time(200));
    assert_eq!(r.critical().cells, vec![1, 3, 4]);
}

fn costs_strategy() -> impl Strategy<Value = Costs> {
    (1u32..=300, 1u32..=300, 1u32..=300, 1u32..=300).prop_map(|(a, b, c, d)| Costs {
        fa_area: a,
        fa_delay: b,
        ha_area: c,
        ha_delay: d,
        ..Costs::default()
    })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    /// Building under some costs, or building under the default and switching afterwards,
    /// gives the same circuit, readiness, order, metrics and critical cells, at every step
    /// of the history.
    #[test]
    fn ger12_switching_costs_equals_building_with_them(seed in any::<u64>(), costs in costs_strategy()) {
        let mut built = r1();
        built.set_costs(costs);
        reduce_at_random(&mut built, &mut Lcg(seed));

        // The same cells, chosen by id, under the default costs.
        let mut switched = r1();
        let mut steps = 0;
        let mut replay = built.clone();
        while replay.undo() {
            steps += 1;
        }
        for _ in 0..steps {
            assert!(replay.redo());
            let before = switched.levels().len();
            let target = replay.levels().len();
            if target > before {
                let l = &replay.levels()[before];
                if l.cpa {
                    switched.close_with_cpa().unwrap();
                } else {
                    switched.conclude_level().unwrap();
                }
            } else {
                let c = replay.pending().last().expect("a step forms a cell or ends a level");
                switched.form_cell(&c.inputs).unwrap();
            }
        }
        switched.set_costs(costs);
        loop {
            prop_assert_eq!(switched.points(), built.points());
            prop_assert_eq!(switched.pending(), built.pending());
            prop_assert_eq!(switched.levels(), built.levels());
            prop_assert_eq!(switched.metrics(), built.metrics());
            prop_assert_eq!(switched.critical(), built.critical());
            let (a, b) = (switched.undo(), built.undo());
            prop_assert_eq!(a, b);
            if !a {
                break;
            }
        }
    }
}

/// `A + B + 2(C + D + E)`, widths 1: an FA over column 1 at level 0; the CPA then adds
/// column 0 (a0, b0) with an HA, and column 1 (s1 and that HA's carry) with another.
/// The incoming carry is the last input of a CPA cell (spec 02) whatever its time: under
/// FA 0.4 it comes after s1 (0.4 < 0.5), and switching to the default (s1 at 1, carry at
/// 0.5) must not move it ahead.
#[test]
fn ger12_the_cpa_keeps_its_incoming_carry_last() {
    let fast_fa = Costs {
        fa_area: 100,
        fa_delay: 40,
        ha_area: 50,
        ha_delay: 50,
        ..Costs::default()
    };
    let build = |costs: Costs| {
        let mut r = reduction(
            "A+B+2(C+D+E)",
            &[('A', 1), ('B', 1), ('C', 1), ('D', 1), ('E', 1)],
        );
        r.set_costs(costs);
        form(&mut r, 1, &["c0", "d0", "e0"]);
        r.conclude_level().unwrap();
        r.close_with_cpa().unwrap();
        r
    };
    let mut switched = build(fast_fa);
    switched.set_costs(Costs::default());
    let direct = build(Costs::default());
    assert_eq!(switched.levels(), direct.levels());
    let cpa = &direct.levels()[1];
    let second = &cpa.cells[1];
    let names: Vec<String> = second
        .inputs
        .iter()
        .map(|&i| direct.signal(direct.point(i).unwrap()))
        .collect();
    assert_eq!(names, vec!["s1", "k2"]);
}
