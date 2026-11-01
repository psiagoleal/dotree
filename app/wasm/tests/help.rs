// Caminho relativo: app/wasm/tests/help.rs
//! Contract of the on-demand help the interface asks the `Session` for (spec 03; MT-13).
//! Each call is a reveal: the interface calls only after the user confirmed (AJU-02), and
//! the session counts it (AJU-07).

use dotree_wasm::Session;
use serde_json::{Value, json};

const R1: &str = "A*B + C*D + 3(E+F) + G";
const R1_WIDTHS: &str = r#"{"A":3,"B":3,"C":2,"D":2,"E":2,"F":1,"G":1}"#;

fn r1() -> Session {
    Session::open(R1, R1_WIDTHS).expect("R1 opens")
}

fn parse(s: String) -> Value {
    serde_json::from_str(&s).unwrap()
}

fn id_of(s: &Session, column: u64, signal: &str) -> Value {
    let v = parse(s.snapshot());
    v["current"]["points"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["column"] == column && p["signal"] == signal)
        .unwrap()["id"]
        .clone()
}

#[test]
fn nothing_is_counted_before_a_request() {
    assert_eq!(
        parse(r1().help_log()),
        json!({"hints": 0, "nextMoves": 0, "lowerBound": 0, "lowerBoundPath": 0,
               "solution": 0, "solutionPath": 0})
    );
}

#[test]
fn hints_cycle_through_what_applies_now() {
    let mut s = r1();
    let shown: Vec<Value> = (0..4).map(|_| parse(s.hint())["hint"].clone()).collect();
    assert_eq!(
        shown,
        [
            json!({"code": "minLevelsLeft", "height": 7, "levels": 4}),
            json!({"code": "tallestColumn", "column": 1, "height": 7}),
            json!({"code": "idlePoints"}),
            json!({"code": "minLevelsLeft", "height": 7, "levels": 4}),
        ]
    );
    assert_eq!(parse(s.help_log())["hints"], 4);
}

#[test]
fn the_next_move_names_points_and_changes_nothing() {
    let mut s = r1();
    let before = s.snapshot();
    assert_eq!(
        parse(s.next_move()),
        json!({"move": "formCell", "kind": "ha",
               "points": [id_of(&s, 1, "a0_b1"), id_of(&s, 1, "a1_b0")]})
    );
    assert_eq!(s.snapshot(), before);
    assert_eq!(parse(s.help_log())["nextMoves"], 1);
}

#[test]
fn lower_bound_with_its_numbers() {
    let mut s = r1();
    assert_eq!(
        parse(s.lower_bound(false)),
        json!({"dots": 20, "outputWidth": 7, "value": 13})
    );
    s.lower_bound(true);
    let log = parse(s.help_log());
    assert_eq!(
        (log["lowerBound"].clone(), log["lowerBoundPath"].clone()),
        (json!(1), json!(1))
    );
}

#[test]
fn the_solution_is_a_separate_session_from_level_0() {
    let mut s = r1();
    let ids = json!([id_of(&s, 2, "a0_b2"), id_of(&s, 2, "a1_b1")]).to_string();
    s.form_cell(&ids);
    let before = s.snapshot();

    let solution = s.solution(false);
    let v = parse(solution.snapshot());
    assert_eq!(v["metrics"]["fa"], 13);
    assert_eq!(v["metrics"]["ha"], 8);
    assert_eq!(v["metrics"]["area"], 34);
    let levels = v["levels"].as_array().unwrap();
    assert_eq!(levels.len(), 5);
    assert_eq!(levels[4]["cpa"], true);
    assert!(v["result"].is_array());

    // AJU-03: the student's circuit is untouched.
    assert_eq!(s.snapshot(), before);
    s.solution(true);
    let log = parse(s.help_log());
    assert_eq!(
        (log["solution"].clone(), log["solutionPath"].clone()),
        (json!(1), json!(1))
    );
}

#[test]
fn no_hint_once_complete_and_nothing_counted() {
    let mut s = Session::open("2A", r#"{"A":2}"#).unwrap();
    assert_eq!(parse(s.hint()), json!({"hint": null}));
    assert_eq!(parse(s.next_move()), json!({"move": "complete"}));
    assert_eq!(parse(s.help_log())["hints"], 0);
}
