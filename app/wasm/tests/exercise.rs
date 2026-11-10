// Caminho relativo: app/wasm/tests/exercise.rs
//! The exercise file through the bridge (spec 08, MT-15a): `exercise` exports, `restore`
//! opens, and the reveals come back without being counted again.

mod common;

use common::{R1, r1, r2_levels, snap};
use dotree_wasm::{Session, SolutionMethod};
use serde_json::{Value, json};

fn r2() -> Session {
    let mut s = r2_levels(3);
    s.close_with_cpa();
    s
}

fn json(s: &str) -> Value {
    serde_json::from_str(s).unwrap()
}

fn refusal(file: &str) -> Value {
    match Session::restore(file) {
        Ok(_) => panic!("{file} was accepted"),
        Err(e) => json(&e),
    }
}

#[test]
fn sal02_restore_gives_back_the_same_exercise() {
    let mut s = r2();
    s.reveal_width(Some(6));
    assert!(s.reveal_width_path());
    s.lower_bound(false);
    s.hint();
    let file = s.exercise();
    let back = Session::restore(&file).expect("its own export opens");
    assert_eq!(snap(&back), snap(&s));
    assert_eq!(json(&back.help_log()), json(&s.help_log()));
    assert_eq!(json(&back.revealed()), json(&s.revealed()));
    assert_eq!(back.exercise(), file);
    assert_eq!(back.expression(), R1);
    assert_eq!(json(&back.widths())["E"], 2);
}

#[test]
fn reveals_start_hidden_and_are_kept() {
    let mut s = r1();
    assert_eq!(
        json(&s.revealed()),
        json!({"width": null, "lowerBound": "none", "solution": "none", "wallace": "none",
               "optimal": "none", "block": false})
    );
    // Step 2 of the width needs step 1 first (GER-03).
    assert!(!s.reveal_width_path());
    s.reveal_width(None);
    s.reveal_width(Some(9)); // the first answer stands
    assert!(s.reveal_width_path());
    s.lower_bound(false);
    s.lower_bound(true);
    let _ = s.solution(false, SolutionMethod::Dadda);
    assert_eq!(
        json(&s.revealed()),
        json!({"width": {"answer": null, "path": true}, "lowerBound": "path",
               "solution": "result", "wallace": "none", "optimal": "none", "block": false})
    );
}

#[test]
fn shown_helps_do_not_count_again() {
    let mut s = r1();
    assert_eq!(s.shown_lower_bound(), None);
    assert!(s.shown_solution(SolutionMethod::Dadda).is_none());
    assert!(s.shown_solution(SolutionMethod::Wallace).is_none());
    let bound = s.lower_bound(false);
    let solution = snap(&s.solution(false, SolutionMethod::Dadda));
    assert!(
        s.shown_solution(SolutionMethod::Wallace).is_none(),
        "seeing Dadda shows nothing of Wallace"
    );
    let wallace = snap(&s.solution(false, SolutionMethod::Wallace));
    let log = s.help_log();
    assert_eq!(s.shown_lower_bound(), Some(bound));
    assert_eq!(
        snap(&s.shown_solution(SolutionMethod::Dadda).unwrap()),
        solution
    );
    assert_eq!(
        snap(&s.shown_solution(SolutionMethod::Wallace).unwrap()),
        wallace
    );
    assert_ne!(wallace, solution);
    assert_eq!(json(&s.revealed())["wallace"], "result");
    assert_eq!(s.help_log(), log);
}

#[test]
fn sal03_refusals_carry_a_code() {
    let file = json(&r2().exercise());
    assert_eq!(refusal("{")["code"], "malformed");
    assert!(refusal("{")["detail"].as_str().is_some());
    assert_eq!(refusal(r#"{"a": 1}"#), json!({"code": "notExercise"}));
    let with = |key: &str, value: Value| {
        let mut v = file.clone();
        v[key] = value;
        refusal(&v.to_string())
    };
    assert_eq!(
        with("schema", 2.into()),
        json!({"code": "futureSchema", "found": 2})
    );
    assert_eq!(
        with("schema", 0.into()),
        json!({"code": "oldSchema", "found": 0})
    );
    assert_eq!(
        with("expression", "A*".into()),
        json!({"code": "badExpression"})
    );
    assert_eq!(
        with(
            "pending",
            json!([{"kind": "ha", "column": 0, "inputs": [0, 1]}])
        ),
        json!({"code": "inconsistentLevel", "level": 4})
    );
    let mut v = file.clone();
    v["levels"][0]["cells"][0]["kind"] = "ha".into();
    assert_eq!(
        refusal(&v.to_string()),
        json!({"code": "inconsistentCell", "level": 0, "position": 1})
    );
    let mut v = file.clone();
    v["help"]["solutionPath"] = 1.into();
    assert_eq!(
        refusal(&v.to_string()),
        json!({"code": "inconsistentReveals"})
    );
}
