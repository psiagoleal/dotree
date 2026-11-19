// Caminho relativo: app/wasm/tests/profile.rs
//! Contract of the height profile (MT-31b): the input `{…; …}` (EXP-16, EXP-17), whether
//! the profile was reached (CMP-19), and the optimal circuit and the block name as counted
//! helps (AJU-12).

use dotree_wasm::{Session, SolutionMethod, build_matrix, search_optimal};
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
    let mut s = open("{5,5; 1,1,1,1}");
    assert_eq!(s.search_optimal().as_deref(), Some("ok"));
    assert_eq!(
        open("{6,7,7,6; 1,1,1,1,1,1,1}").search_optimal().as_deref(),
        Some("tooLarge")
    );
    let mut e = Session::open("A", r#"{"A":2}"#).unwrap();
    assert_eq!(e.optimal_search(), None);
    assert_eq!(e.search_optimal(), None);
    assert_eq!(e.optimal_request(), None);
    assert_eq!(parse(s.help_log())["optimal"], 0);
}

// ---------------------------------------------------------------- MT-42: off the main thread

/// What the worker does with the session's request.
fn in_worker(s: &Session) -> String {
    search_optimal(&s.optimal_request().expect("a profile"))
}

#[test]
fn mt42_the_session_waits_for_the_worker_and_takes_its_circuit() {
    let mut s = open("{5,5; 1,1,1,1}");
    // Opening a profile searches nothing: the page stays free (GER-06).
    assert_eq!(s.optimal_search().as_deref(), Some("pending"));
    let result = in_worker(&s);
    assert_eq!(s.optimal_search().as_deref(), Some("pending"));
    assert_eq!(s.adopt_optimal(&result).as_deref(), Some("ok"));
    assert_eq!(s.optimal_search().as_deref(), Some("ok"));
    let best = parse(s.solution(false, OPTIMAL).snapshot());
    assert_eq!(
        (
            best["metrics"]["area"].clone(),
            best["metrics"]["delay"].clone()
        ),
        (json!(600), json!(400))
    );
    assert_eq!(best["profile"]["reached"], true);
}

#[test]
fn mt42_a_search_too_large_comes_back_as_such() {
    let mut s = open("{6,7,7,6; 1,1,1,1,1,1,1}");
    let result = in_worker(&s);
    assert_eq!(s.adopt_optimal(&result).as_deref(), Some("tooLarge"));
    assert_eq!(s.optimal_search().as_deref(), Some("tooLarge"));
}

#[test]
fn mt42_a_result_for_other_costs_or_another_profile_is_ignored() {
    let mut s = Session::open("{2,2; 1,1,1}", "{}").unwrap();
    let old = in_worker(&s);
    assert!(s.set_costs(r#"{"faArea":100,"faDelay":100,"haArea":30,"haDelay":30}"#));
    assert_eq!(s.adopt_optimal(&old), None, "searched under the old costs");
    assert_eq!(s.optimal_search().as_deref(), Some("pending"));
    let other = search_optimal(&open("{3; 1,1}").optimal_request().unwrap());
    assert_eq!(s.adopt_optimal(&other), None, "another profile");
    assert_eq!(s.adopt_optimal("not json"), None);
    let fresh = in_worker(&s);
    assert_eq!(s.adopt_optimal(&fresh).as_deref(), Some("ok"));
    let sol = parse(s.solution(false, OPTIMAL).snapshot());
    assert_eq!(sol["metrics"]["ha"], 4);
}

#[test]
fn mt42_a_circuit_that_breaks_the_rules_or_misses_the_profile_is_refused() {
    let mut s = open("{5,5; 1,1,1,1}");
    let mut v = parse(in_worker(&s));
    // A cell over points that do not exist: the replay through the rules refuses it.
    v["circuit"]["levels"][0]["cells"][0]["inputs"] = json!([0, 1, 99]);
    assert_eq!(s.adopt_optimal(&v.to_string()), None);
    // The right envelope around a circuit of other costs, or of another profile.
    let mut v = parse(in_worker(&s));
    v["circuit"]["costs"] = json!({"faArea": 100, "faDelay": 100, "haArea": 30, "haDelay": 30});
    assert_eq!(s.adopt_optimal(&v.to_string()), None);
    let mut v = parse(in_worker(&s));
    v["circuit"]["expression"] = json!("{5,5; 1,1,2,1}");
    assert_eq!(s.adopt_optimal(&v.to_string()), None);
    // No cell at all: valid, but it does not reach the profile.
    let mut v = parse(in_worker(&s));
    v["circuit"]["levels"] = json!([]);
    assert_eq!(s.adopt_optimal(&v.to_string()), None);
    assert_eq!(s.optimal_search().as_deref(), Some("pending"));
}

#[test]
fn mt42_a_revealed_optimum_is_not_shown_while_the_search_runs_again() {
    let mut s = Session::open("{2,2; 1,1,1}", "{}").unwrap();
    s.search_optimal();
    let _ = s.solution(false, OPTIMAL);
    assert!(s.shown_solution(OPTIMAL).is_some());
    assert!(s.set_costs(r#"{"faArea":100,"faDelay":100,"haArea":30,"haDelay":30}"#));
    assert!(
        s.shown_solution(OPTIMAL).is_none(),
        "no empty circuit in its place"
    );
    assert!(!s.apply_solution(OPTIMAL));
    s.search_optimal();
    assert!(s.shown_solution(OPTIMAL).is_some());
}

#[test]
fn aju12_the_optimum_is_a_solution_counted_apart() {
    let mut s = open("{5,5; 1,1,1,1}");
    s.search_optimal();
    assert!(s.shown_solution(OPTIMAL).is_none());
    let best = parse(s.solution(false, OPTIMAL).snapshot());
    // d) of the reference set: 6 A_FA and 4 T_FA, in hundredths.
    assert_eq!(best["metrics"]["area"], 600);
    assert_eq!(best["metrics"]["delay"], 400);
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
    s.search_optimal();
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
