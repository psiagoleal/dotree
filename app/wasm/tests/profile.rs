// Caminho relativo: app/wasm/tests/profile.rs
//! Contract of the height profile (MT-31b): the input `{…; …}` (EXP-16, EXP-17), whether
//! the profile was reached (CMP-19), and the optimal circuit and the block name as counted
//! helps (AJU-12).

use dotree_wasm::{Session, SolutionMethod, build_matrix};
use serde_json::{Value, json};

const OPTIMAL: SolutionMethod = SolutionMethod::Optimal;

fn parse(s: String) -> Value {
    serde_json::from_str(&s).unwrap()
}

fn open(src: &str) -> Session {
    Session::open(src, "{}").expect("the profile opens")
}

#[test]
fn exp16_a_profile_builds_its_matrix_without_widths() {
    let v = parse(build_matrix("{2,2,3; 1,1,1,1}", "{}"));
    assert_eq!(v["ok"], true);
    assert_eq!(v["matrix"]["heights"], json!([3, 2, 2, 0]));
    assert_eq!(v["matrix"]["andGates"], 0);
    // Least significant column first, as everywhere in the bridge.
    assert_eq!(
        v["matrix"]["profile"],
        json!({"input": [3, 2, 2], "output": [1, 1, 1, 1]})
    );
    assert_eq!(
        parse(build_matrix("A*B", r#"{"A":2,"B":2}"#))["matrix"]["profile"],
        Value::Null
    );
}

#[test]
fn exp16_a_profile_keeps_no_widths_in_its_file() {
    // Widths left over from an earlier expression in the form are ignored, not saved.
    let s = Session::open("{3; 1,1}", r#"{"A":4,"B":2}"#).expect("the profile opens");
    assert_eq!(parse(s.widths()), json!({}));
    let file = parse(s.exercise());
    assert_eq!(file["widths"], json!({}));
}

#[test]
fn exp17_profile_errors_come_as_codes() {
    let error = |src: &str| parse(build_matrix(src, "{}"))["error"].clone();
    assert_eq!(
        error("{2,2 1}"),
        json!({"code": "profile", "kind": "syntax", "column": 6})
    );
    assert_eq!(
        error("{0; 1}"),
        json!({"code": "profile", "kind": "noDots"})
    );
    assert_eq!(
        error("{9,9,9; 1,1,1,1,1,1}"),
        json!({"code": "profile", "kind": "tooManyDots", "dots": 27, "limit": 26})
    );
    assert_eq!(
        error("{1; 1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1}"),
        json!({"code": "profile", "kind": "tooManyColumns", "limit": 16})
    );
    assert_eq!(
        error("{7; 1,1}"),
        json!({"code": "profile", "kind": "outputTooSmall", "column": 0})
    );
    assert_eq!(
        error("{2; 2,0}"),
        json!({"code": "profile", "kind": "emptyColumn", "column": 0})
    );
}

#[test]
fn cmp19_the_snapshot_says_whether_the_profile_was_reached() {
    let mut s = open("{3; 1,1}");
    let v = parse(s.snapshot());
    assert_eq!(v["profile"], json!({"output": [1, 1], "reached": false}));
    let ids: Vec<Value> = v["current"]["points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].clone())
        .collect();
    assert_eq!(parse(s.form_cell(&json!(ids).to_string()))["ok"], true);
    assert_eq!(parse(s.snapshot())["profile"]["reached"], false);
    assert_eq!(parse(s.conclude_level())["ok"], true);
    assert_eq!(parse(s.snapshot())["profile"]["reached"], true);
    // An expression has no profile.
    let r = Session::open("A*B", r#"{"A":2,"B":2}"#).unwrap();
    assert_eq!(parse(r.snapshot())["profile"], Value::Null);
}

#[test]
fn aju12_the_search_status_reveals_nothing_and_counts_nothing() {
    let s = open("{5,5; 1,1,1,1}");
    assert_eq!(s.optimal_search().as_deref(), Some("ok"));
    assert_eq!(
        open("{6,7,7,6; 1,1,1,1,1,1,1}").optimal_search().as_deref(),
        Some("tooLarge")
    );
    let e = Session::open("A", r#"{"A":2}"#).unwrap();
    assert_eq!(e.optimal_search(), None);
    assert_eq!(parse(s.help_log())["optimal"], 0);
}

#[test]
fn aju12_the_optimum_is_a_solution_counted_apart() {
    let mut s = open("{5,5; 1,1,1,1}");
    assert!(s.shown_solution(OPTIMAL).is_none());
    let best = parse(s.solution(false, OPTIMAL).snapshot());
    // d) of the reference set: 6 A_FA and 4 T_FA, in halves.
    assert_eq!(best["metrics"]["area"], 12);
    assert_eq!(best["metrics"]["delay"], 8);
    assert_eq!(best["profile"]["reached"], true);
    assert!(!s.apply_solution(OPTIMAL), "not before the path");
    let _ = s.solution(true, OPTIMAL);
    assert!(s.apply_solution(OPTIMAL));
    assert_eq!(parse(s.snapshot())["levels"], best["levels"]);
    let log = parse(s.help_log());
    assert_eq!(
        (
            log["optimal"].clone(),
            log["optimalPath"].clone(),
            log["optimalApplied"].clone(),
            log["solution"].clone()
        ),
        (json!(1), json!(1), json!(1), json!(0))
    );
    assert_eq!(parse(s.revealed())["optimal"], "path");
    // Shown again uncounted.
    assert!(s.shown_solution(OPTIMAL).is_some());
    assert_eq!(parse(s.help_log())["optimal"], 1);
}

#[test]
fn aju12_the_block_name_is_a_counted_reveal() {
    let mut s = open("{2,2,3; 1,1,1,1}");
    assert_eq!(s.shown_block(), None);
    assert_eq!(parse(s.revealed())["block"], false);
    assert_eq!(
        parse(s.block()),
        json!({"code": "adderWithCarryIn", "bits": 3})
    );
    assert_eq!(parse(s.help_log())["block"], 1);
    assert_eq!(parse(s.revealed())["block"], true);
    assert_eq!(
        s.shown_block().map(parse),
        Some(json!({"code": "adderWithCarryIn", "bits": 3}))
    );
    assert_eq!(parse(s.help_log())["block"], 1);
    for (src, block) in [
        ("{3; 1,1}", json!({"code": "fullAdder"})),
        ("{2; 1,1}", json!({"code": "halfAdder"})),
        ("{2,2; 1,1,1}", json!({"code": "adder", "bits": 2})),
        ("{3,3,3; 1,2,2,1}", json!({"code": "carrySave", "bits": 3})),
        ("{7; 1,1,1}", json!({"code": "counter"})),
        ("{5; 2,1}", json!({"code": "compressor"})),
    ] {
        assert_eq!(parse(open(src).block()), block, "{src}");
    }
}

#[test]
fn aju12_a_profile_exercise_reopens_with_its_profile_and_reveals() {
    let mut s = open("{5; 2,1}");
    s.block();
    let _ = s.solution(false, OPTIMAL);
    let back = Session::restore(&s.exercise()).expect("its own file opens");
    assert_eq!(back.expression(), "{5; 2,1}");
    assert_eq!(parse(back.widths()), json!({}));
    assert_eq!(
        parse(back.snapshot())["profile"],
        json!({"output": [1, 2], "reached": false})
    );
    assert_eq!(parse(back.revealed())["optimal"], "result");
    assert_eq!(parse(back.revealed())["block"], true);
}
