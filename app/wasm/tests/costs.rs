// Caminho relativo: app/wasm/tests/costs.rs
//! Configurable costs through the session (GER-12, MT-45): read and set them, see metrics,
//! solutions and the optimum follow, keep them in the exercise file.

use dotree_wasm::{Session, SolutionMethod};
use serde_json::{Value, json};

const R1: &str = "A*B + C*D + 3(E+F) + G";
const R1_WIDTHS: &str = r#"{"A":3,"B":3,"C":2,"D":2,"E":2,"F":1,"G":1}"#;

fn parse(s: String) -> Value {
    serde_json::from_str(&s).unwrap()
}

fn r1() -> Session {
    Session::open(R1, R1_WIDTHS).expect("R1 opens")
}

fn costs(fa_area: u32, fa_delay: u32, ha_area: u32, ha_delay: u32) -> String {
    json!({"faArea": fa_area, "faDelay": fa_delay, "haArea": ha_area, "haDelay": ha_delay})
        .to_string()
}

/// Reveals Dadda's levels and uses them: the circuit becomes Dadda's, in one undo step.
fn apply_dadda(s: &mut Session) {
    let _ = s.solution(false, SolutionMethod::Dadda);
    let _ = s.solution(true, SolutionMethod::Dadda);
    assert!(s.apply_solution(SolutionMethod::Dadda));
}

#[test]
fn ger12_a_session_starts_with_the_default_costs() {
    let s = r1();
    assert_eq!(parse(s.costs()), parse(costs(100, 100, 50, 50)));
    assert_eq!(parse(s.snapshot())["costs"], parse(s.costs()));
}

#[test]
fn ger12_new_costs_change_the_metrics_and_stay_out_of_the_history() {
    let mut s = r1();
    apply_dadda(&mut s);
    let before = parse(s.snapshot());
    // Dadda on R1: 13 FA and 8 HA, 17 A_FA, 7 T_FA (spec 03).
    assert_eq!(before["metrics"]["area"], 1700);
    assert_eq!(before["metrics"]["delay"], 700);

    assert!(s.set_costs(&costs(200, 100, 50, 50)));
    let after = parse(s.snapshot());
    assert_eq!(after["metrics"]["area"], 13 * 200 + 8 * 50);
    assert_eq!(after["costs"]["faArea"], 200);
    // The history is the same: one undo goes back to the empty circuit.
    assert_eq!(after["canUndo"], true);
    assert!(s.undo());
    assert_eq!(parse(s.snapshot())["metrics"]["area"], 0);
    assert!(!s.undo());
}

#[test]
fn ger12_invalid_costs_change_nothing() {
    let mut s = r1();
    for bad in [
        costs(0, 100, 50, 50),
        costs(100, 100, 50, 10_001),
        r#"{"faArea": 100}"#.into(),
        r#"{"faArea": 1.5, "faDelay": 100, "haArea": 50, "haDelay": 50}"#.into(),
        "not json".into(),
    ] {
        assert!(!s.set_costs(&bad), "{bad}");
        assert_eq!(parse(s.costs()), parse(costs(100, 100, 50, 50)));
    }
}

#[test]
fn ger12_solutions_are_measured_with_the_session_costs() {
    let mut s = r1();
    assert!(s.set_costs(&costs(100, 100, 100, 100)));
    let sol = s.solution(false, SolutionMethod::Wallace);
    let m = parse(sol.snapshot())["metrics"].clone();
    let (fa, ha) = (m["fa"].as_u64().unwrap(), m["ha"].as_u64().unwrap());
    assert_eq!(m["area"].as_u64().unwrap(), 100 * (fa + ha));
    assert_eq!(parse(sol.costs()), parse(s.costs()));
}

#[test]
fn ger12_the_optimum_is_searched_again_under_new_costs() {
    // e) {2,2; 1,1,1}: 1 FA + 1 HA by default, 4 HA once an HA costs 0.3 (core test).
    let mut s = Session::open("{2,2; 1,1,1}", "{}").unwrap();
    assert_eq!(s.optimal_search().as_deref(), Some("ok"));
    let before = parse(s.solution(false, SolutionMethod::Optimal).snapshot());
    assert_eq!(before["metrics"]["fa"], 1);
    assert_eq!(before["metrics"]["ha"], 1);

    assert!(s.set_costs(&costs(100, 100, 30, 30)));
    let after = parse(
        s.shown_solution(SolutionMethod::Optimal)
            .expect("already revealed")
            .snapshot(),
    );
    assert_eq!(after["metrics"]["fa"], 0);
    assert_eq!(after["metrics"]["ha"], 4);
    assert_eq!(after["metrics"]["area"], 120);
    assert_eq!(after["metrics"]["delay"], 90);
}

#[test]
fn ger12_costs_travel_in_the_exercise_file() {
    let mut s = r1();
    apply_dadda(&mut s);
    assert!(s.set_costs(&costs(120, 40, 70, 50)));
    let file = s.exercise();
    assert_eq!(
        parse(file.clone())["costs"],
        json!({"faArea": 120, "faDelay": 40, "haArea": 70, "haDelay": 50})
    );
    let back = Session::restore(&file).expect("its own file opens");
    assert_eq!(parse(back.costs()), parse(s.costs()));
    assert_eq!(
        parse(back.snapshot())["metrics"],
        parse(s.snapshot())["metrics"]
    );
}
