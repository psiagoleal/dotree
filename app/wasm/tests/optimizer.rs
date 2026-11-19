// Caminho relativo: app/wasm/tests/optimizer.rs
//! Contract of the optimizer (AJU-14, MT-46): searched in a worker and handed over, as the
//! optimal circuit of a profile (MT-42); revealed in two counted steps; each option applied
//! as one undo step and counted.

use dotree_wasm::{Session, search_optimizer};
use serde_json::{Value, json};

fn parse(s: &str) -> Value {
    serde_json::from_str(s).unwrap()
}

fn r1() -> Session {
    Session::open(
        "A*B + C*D + 3(E+F) + G",
        r#"{"A":3,"B":3,"C":2,"D":2,"E":2,"F":1,"G":1}"#,
    )
    .unwrap()
}

fn in_worker(s: &Session) -> String {
    search_optimizer(&s.optimizer_request())
}

fn log(s: &Session) -> (u64, u64, u64) {
    let l = parse(&s.help_log());
    let n = |k: &str| l[k].as_u64().unwrap();
    (n("optimizer"), n("optimizerPath"), n("optimizerApplied"))
}

fn metrics(s: &Session) -> (u64, u64) {
    let v = parse(&s.snapshot());
    (
        v["metrics"]["area"].as_u64().unwrap(),
        v["metrics"]["delay"].as_u64().unwrap(),
    )
}

#[test]
fn aju14_the_session_waits_for_the_worker_and_takes_its_options() {
    let mut s = r1();
    assert_eq!(s.optimizer_search(), "pending");
    // Nothing to reveal while the search runs, and nothing counted.
    assert_eq!(s.optimizer(false), None);
    assert_eq!(log(&s), (0, 0, 0));
    let result = in_worker(&s);
    assert_eq!(s.adopt_optimizer(&result).as_deref(), Some("ok"));
    assert_eq!(s.optimizer_search(), "ok");
    assert_eq!(log(&s), (0, 0, 0));
    assert_eq!(s.shown_optimizer(), None);
    assert!(s.optimizer_option(0).is_none(), "nothing before step 1");

    let shown = parse(&s.optimizer(false).unwrap());
    assert_eq!(
        shown,
        json!({"mode": "exactUpTo", "upTo": 1550, "count": 2})
    );
    assert_eq!(log(&s), (1, 0, 0));
    assert_eq!(s.shown_optimizer().map(|v| parse(&v)), Some(shown));
    assert_eq!(log(&s), (1, 0, 0), "shown again uncounted");
    let first = s.optimizer_option(0).unwrap();
    assert_eq!(metrics(&first), (1450, 600));
    assert_eq!(metrics(&s.optimizer_option(1).unwrap()), (1500, 550));
    assert!(s.optimizer_option(2).is_none());
}

#[test]
fn aju14_an_option_is_applied_after_step_2_as_one_undo_step() {
    let mut s = r1();
    s.adopt_optimizer(&in_worker(&s)).unwrap();
    assert_eq!(s.optimizer(true), None, "step 2 needs step 1");
    s.optimizer(false).unwrap();
    assert!(!s.apply_optimizer_option(1), "not before step 2");
    s.optimizer(true).unwrap();
    assert_eq!(log(&s), (1, 1, 0));
    assert!(!s.apply_optimizer_option(5));
    assert!(s.apply_optimizer_option(1));
    assert_eq!(metrics(&s), (1500, 550));
    assert!(parse(&s.snapshot())["result"].is_array());
    assert_eq!(log(&s), (1, 1, 1));
    assert!(!s.apply_optimizer_option(1), "already this circuit");
    assert_eq!(log(&s), (1, 1, 1));
    assert!(s.undo());
    assert_eq!(metrics(&s), (0, 0));
    assert!(s.redo());
    assert_eq!(metrics(&s), (1500, 550));
    // The counts travel in the exercise file and come back.
    let back = Session::restore(&s.exercise()).unwrap();
    assert_eq!(log(&back), (1, 1, 1));
    assert_eq!(parse(&back.revealed())["optimizer"], "path");
}

#[test]
fn aju14_a_result_for_other_costs_or_another_exercise_is_ignored() {
    let mut s = r1();
    let stale = in_worker(&s);
    assert!(s.set_costs(r#"{"faArea":100,"faDelay":100,"haArea":30,"haDelay":30}"#));
    assert_eq!(s.adopt_optimizer(&stale), None);
    assert_eq!(s.optimizer_search(), "pending");
    let other = Session::open("A*B", r#"{"A":2,"B":2}"#).unwrap();
    assert_eq!(s.adopt_optimizer(&in_worker(&other)), None);
    // The same expression with other widths is another exercise.
    let mut small = Session::open("A*B", r#"{"A":2,"B":2}"#).unwrap();
    let wide = Session::open("A*B", r#"{"A":3,"B":2}"#).unwrap();
    assert_eq!(small.adopt_optimizer(&in_worker(&wide)), None);
    assert_eq!(small.optimizer_search(), "pending");
    assert_eq!(s.adopt_optimizer("not json"), None);
    assert_eq!(s.optimizer_search(), "pending");
    // New costs drop the options found under the old ones.
    let fresh = in_worker(&s);
    assert_eq!(s.adopt_optimizer(&fresh).as_deref(), Some("ok"));
    s.optimizer(false).unwrap();
    assert!(s.set_costs(r#"{"faArea":100,"faDelay":100,"haArea":50,"haDelay":50}"#));
    assert_eq!(s.optimizer_search(), "pending");
    assert_eq!(
        s.shown_optimizer(),
        None,
        "nothing shown while searched again"
    );
    assert!(s.optimizer_option(0).is_none());
}

#[test]
fn aju14_an_option_that_breaks_the_rules_or_is_incomplete_is_refused() {
    let mut s = r1();
    let good = parse(&in_worker(&s));
    // The last level of the first option dropped: no longer complete.
    let mut v = good.clone();
    let levels = v["options"][0]["levels"].as_array_mut().unwrap();
    levels.pop();
    assert_eq!(s.adopt_optimizer(&v.to_string()), None);
    // A cell moved to another column breaks a rule.
    let mut v = good.clone();
    v["options"][1]["levels"][0]["cells"][0]["column"] = 5.into();
    assert_eq!(s.adopt_optimizer(&v.to_string()), None);
    // Costs or expression changed inside an option only.
    let mut v = good.clone();
    v["options"][0]["costs"]["haArea"] = 30.into();
    assert_eq!(s.adopt_optimizer(&v.to_string()), None);
    let mut v = good.clone();
    v["options"][0]["expression"] = "A*B".into();
    assert_eq!(s.adopt_optimizer(&v.to_string()), None);
    // No options, or a mode it does not know.
    for (key, bad) in [("options", json!([])), ("mode", json!("best"))] {
        let mut v = good.clone();
        v[key] = bad;
        assert_eq!(s.adopt_optimizer(&v.to_string()), None, "{key}");
    }
    assert_eq!(s.optimizer_search(), "pending");
    assert_eq!(s.adopt_optimizer(&good.to_string()).as_deref(), Some("ok"));
}

#[test]
fn aju14_a_profile_has_options_at_the_profile() {
    let mut s = Session::open("{5,5; 1,1,1,1}", "{}").unwrap();
    s.adopt_optimizer(&in_worker(&s)).unwrap();
    let shown = parse(&s.optimizer(false).unwrap());
    assert_eq!(shown["mode"], "exact");
    // The least area is the optimum of AJU-12: 6 FA (6 and 4).
    assert_eq!(metrics(&s.optimizer_option(0).unwrap()), (600, 400));
    let o = parse(&s.optimizer_option(0).unwrap().snapshot());
    assert_eq!(o["profile"]["reached"], true);
}

#[test]
fn aju14_the_fallback_searches_here() {
    let mut s = r1();
    assert_eq!(s.search_optimizer_here(), "ok");
    assert_eq!(parse(&s.optimizer(false).unwrap())["count"], 2);
}

#[test]
fn aju14_a_bad_request_has_no_options() {
    let v = parse(&search_optimizer("{}"));
    assert_eq!(v["options"], Value::Null);
    let v = parse(&search_optimizer(
        r#"{"expression":"A*B","widths":{"A":2,"B":2},"costs":{"faArea":0,"faDelay":1,"haArea":1,"haDelay":1}}"#,
    ));
    assert_eq!(v["options"], Value::Null);
}
