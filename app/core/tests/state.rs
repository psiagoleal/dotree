// Caminho relativo: app/core/tests/state.rs
//! Saving and opening an exercise (spec 08, MT-15a): export -> import gives the same state
//! (SAL-02); malformed files, unknown schemas and inconsistent cells are refused (SAL-03).

mod common;

use std::collections::BTreeMap;

use common::{Lcg, form, r1, reduce_at_random, replay_r2_tree};
use dotree_core::help::{HelpLog, HelpStep};
use dotree_core::reduce::Reduction;
use dotree_core::state::{Exercise, SCHEMA, StateError, WidthReveal};
use serde_json::{Value, json};

const R1: &str = "A*B + C*D + 3(E+F) + G";

fn r1_widths() -> BTreeMap<char, u32> {
    [
        ('A', 3),
        ('B', 3),
        ('C', 2),
        ('D', 2),
        ('E', 2),
        ('F', 1),
        ('G', 1),
    ]
    .into_iter()
    .collect()
}

fn capture(r: &Reduction, width: Option<WidthReveal>, help: HelpLog) -> Exercise {
    Exercise::capture(R1, &r1_widths(), r, width, help)
}

fn r2() -> Reduction {
    let mut r = r1();
    replay_r2_tree(&mut r);
    r.close_with_cpa().unwrap();
    r
}

/// Export, parse the text back and rebuild the reduction.
fn round_trip(ex: &Exercise) -> (Exercise, Reduction) {
    let back = Exercise::parse(&ex.to_json()).expect("its own export parses");
    let (_, r) = back.restore().expect("its own export restores");
    (back, r)
}

fn same_reduction(a: &Reduction, b: &Reduction) {
    assert_eq!(a.levels(), b.levels());
    assert_eq!(a.points(), b.points());
    assert_eq!(a.pending(), b.pending());
    assert_eq!(a.metrics(), b.metrics());
    assert_eq!(a.critical(), b.critical());
    assert_eq!(a.result(), b.result());
}

#[test]
fn sal02_complete_r2_round_trip() {
    let r = r2();
    let mut help = HelpLog::default();
    help.record(HelpStep::Hint);
    help.record(HelpStep::LowerBound);
    help.record(HelpStep::LowerBoundPath);
    let width = Some(WidthReveal {
        answer: Some(6),
        path: true,
    });
    let ex = capture(&r, width, help);
    let (back, restored) = round_trip(&ex);
    assert_eq!(back, ex);
    same_reduction(&r, &restored);
    assert!(restored.is_complete());
    assert_eq!(capture(&restored, width, help), ex);
}

#[test]
fn sal02_pending_cells_survive() {
    let mut r = r1();
    form(&mut r, 1, &["a0_b1", "a1_b0", "c0_d1"]);
    form(&mut r, 2, &["a0_b2", "a1_b1"]);
    let ex = capture(&r, None, HelpLog::default());
    assert_eq!(ex.pending.len(), 2);
    let (_, restored) = round_trip(&ex);
    same_reduction(&r, &restored);
}

#[test]
fn sal02_does_not_depend_on_internal_point_ids() {
    // Going back and concluding again allocates new point ids; the file must not care.
    let mut r = r1();
    replay_r2_tree(&mut r);
    r.go_to_level(1).unwrap();
    r.conclude_level().unwrap();
    r.go_to_level(0).unwrap();
    r.conclude_level().unwrap();
    let ex = capture(&r, None, HelpLog::default());
    let (_, restored) = round_trip(&ex);
    assert_eq!(capture(&restored, None, HelpLog::default()), ex);
    assert_eq!(r.metrics(), restored.metrics());
    assert_eq!(r.heights(), restored.heights());
    assert_eq!(r.levels().len(), restored.levels().len());
}

#[test]
fn sal02_random_reductions_round_trip() {
    for seed in 0..32 {
        let mut r = r1();
        reduce_at_random(&mut r, &mut Lcg(seed));
        let ex = capture(&r, None, HelpLog::default());
        let (back, restored) = round_trip(&ex);
        assert_eq!(back, ex, "seed {seed}");
        same_reduction(&r, &restored);
    }
}

#[test]
fn sal02_the_file_names_its_schema_and_contents() {
    let mut help = HelpLog::default();
    help.record(HelpStep::Solution);
    let ex = capture(
        &r2(),
        Some(WidthReveal {
            answer: None,
            path: false,
        }),
        help,
    );
    let v: Value = serde_json::from_str(&ex.to_json()).unwrap();
    assert_eq!(v["format"], "dotree-exercise");
    assert_eq!(v["schema"], SCHEMA);
    assert_eq!(v["expression"], R1);
    assert_eq!(v["widths"]["A"], 3);
    // Level 0 of R2: FA1 over a0_b1, a1_b0, c0_d1, as positions in the level's points.
    let first = &v["levels"][0]["cells"][0];
    assert_eq!(first["kind"], "fa");
    assert_eq!(first["column"], 1);
    assert_eq!(first["inputs"].as_array().unwrap().len(), 3);
    assert_eq!(v["levels"][3]["cpa"], true);
    assert_eq!(v["levels"][3]["cells"], json!([]));
    assert_eq!(v["pending"], json!([]));
    assert_eq!(
        v["reveals"]["width"],
        json!({"answer": null, "path": false})
    );
    assert_eq!(v["help"]["solution"], 1);
    assert_eq!(v["help"]["solutionPath"], 0);
}

// ------------------------------------------------------------------ SAL-03: refusals

fn r2_file() -> Value {
    serde_json::from_str(&capture(&r2(), None, HelpLog::default()).to_json()).unwrap()
}

fn open(v: &Value) -> Result<(), StateError> {
    Exercise::parse(&v.to_string())?.restore().map(|_| ())
}

#[test]
fn sal03_malformed_json_is_refused() {
    for bad in [
        "",
        "{",
        "[]",
        "null",
        "42",
        r#"{"format": "dotree-exercise"}"#,
    ] {
        assert!(
            matches!(Exercise::parse(bad), Err(StateError::Malformed(_))),
            "{bad:?}"
        );
    }
    let mut v = r2_file();
    v.as_object_mut().unwrap().remove("levels");
    assert!(matches!(open(&v), Err(StateError::Malformed(_))));
    let mut v = r2_file();
    v["widths"] = json!({"AB": 3});
    assert!(matches!(open(&v), Err(StateError::Malformed(_))));
}

#[test]
fn sal03_another_json_is_not_an_exercise() {
    assert_eq!(
        Exercise::parse(r#"{"name": "x"}"#),
        Err(StateError::NotAnExercise)
    );
    let mut v = r2_file();
    v["format"] = "dotree-netlist".into();
    assert_eq!(open(&v), Err(StateError::NotAnExercise));
}

#[test]
fn sal03_unknown_schemas_are_refused() {
    let mut v = r2_file();
    v["schema"] = (SCHEMA + 1).into();
    assert_eq!(
        open(&v),
        Err(StateError::FutureSchema {
            found: u64::from(SCHEMA) + 1
        })
    );
    v["schema"] = 0.into();
    assert_eq!(open(&v), Err(StateError::OldSchema { found: 0 }));
    v["schema"] = "1".into();
    assert!(matches!(open(&v), Err(StateError::Malformed(_))));
}

#[test]
fn sal03_a_bad_expression_is_refused() {
    let mut v = r2_file();
    v["expression"] = "A*".into();
    assert_eq!(open(&v), Err(StateError::BadExpression));
    let mut v = r2_file();
    v["widths"]["A"] = 99.into();
    assert_eq!(open(&v), Err(StateError::BadExpression));
}

#[test]
fn sal03_inconsistent_cells_are_refused() {
    let cell = |level: usize, position: usize| StateError::InconsistentCell { level, position };
    let base = r2_file();
    let first_inputs = base["levels"][0]["cells"][0]["inputs"].clone();

    // An input from another column.
    let mut v = base.clone();
    v["levels"][0]["cells"][0]["inputs"][0] = 0.into(); // a0_b0, column 0
    assert_eq!(open(&v), Err(cell(0, 1)));
    // A point that does not exist.
    let mut v = base.clone();
    v["levels"][1]["cells"][2]["inputs"][1] = 999.into();
    assert_eq!(open(&v), Err(cell(1, 3)));
    // The same point twice.
    let mut v = base.clone();
    v["levels"][0]["cells"][0]["inputs"][1] = first_inputs[0].clone();
    assert_eq!(open(&v), Err(cell(0, 1)));
    // A point already used by an earlier cell of the level.
    let mut v = base.clone();
    v["levels"][0]["cells"][1]["inputs"] = first_inputs.clone();
    assert_eq!(open(&v), Err(cell(0, 2)));
    // Kind or column that do not match the inputs.
    let mut v = base.clone();
    v["levels"][0]["cells"][0]["kind"] = "ha".into();
    assert_eq!(open(&v), Err(cell(0, 1)));
    let mut v = base.clone();
    v["levels"][2]["cells"][1]["column"] = 7.into();
    assert_eq!(open(&v), Err(cell(2, 2)));
    let mut v = base.clone();
    v["levels"][0]["cells"][0]["kind"] = "xa".into();
    assert!(matches!(open(&v), Err(StateError::Malformed(_))));
    // A pending cell over a point the level does not have.
    let mut v = r2_file();
    v["levels"].as_array_mut().unwrap().truncate(1);
    v["pending"] = json!([{"kind": "ha", "column": 0, "inputs": [0, 500]}]);
    assert_eq!(open(&v), Err(cell(1, 1)));
}

#[test]
fn sal03_inconsistent_levels_are_refused() {
    let level = |level: usize| StateError::InconsistentLevel { level };
    // An empty concluded level.
    let mut v = r2_file();
    v["levels"][1]["cells"] = json!([]);
    assert_eq!(open(&v), Err(level(1)));
    // A CPA where the matrix is still too tall (level 0 of R1 has height 5).
    let mut v = r2_file();
    v["levels"] = json!([{"cpa": true, "cells": []}]);
    assert_eq!(open(&v), Err(level(0)));
    // A CPA with cells of its own, or followed by more work.
    let mut v = r2_file();
    v["levels"][3]["cells"] = v["levels"][2]["cells"].clone();
    assert_eq!(open(&v), Err(level(3)));
    let mut v = r2_file();
    v["pending"] = json!([{"kind": "ha", "column": 0, "inputs": [0, 1]}]);
    assert_eq!(open(&v), Err(level(4)));
    let mut v = r2_file();
    let cpa = v["levels"][3].clone();
    v["levels"].as_array_mut().unwrap().push(cpa);
    assert_eq!(open(&v), Err(level(4)));
    let mut v = r2_file();
    let tree = v["levels"][2].clone();
    v["levels"].as_array_mut().unwrap().push(tree);
    assert_eq!(open(&v), Err(level(4)));
}

#[test]
fn sal03_impossible_reveals_are_refused() {
    let mut v = r2_file();
    v["reveals"]["width"] = json!({"answer": 0, "path": false});
    assert_eq!(open(&v), Err(StateError::InconsistentReveals));
    // Step 2 of a help without its step 1 (GER-03).
    let mut v = r2_file();
    v["help"]["lowerBoundPath"] = 1.into();
    assert_eq!(open(&v), Err(StateError::InconsistentReveals));
    let mut v = r2_file();
    v["help"]["solutionPath"] = 2.into();
    assert_eq!(open(&v), Err(StateError::InconsistentReveals));
}

// ------------------------------------------------------- AJU-09: the applied solution

#[test]
fn aju09_files_without_the_applied_count_still_open() {
    let mut help = HelpLog::default();
    help.record(HelpStep::Solution);
    let mut v: Value = serde_json::from_str(&capture(&r2(), None, help).to_json()).unwrap();
    assert_eq!(v["help"]["solutionApplied"], 0);
    v["help"].as_object_mut().unwrap().remove("solutionApplied");
    let back = Exercise::parse(&v.to_string()).expect("a schema-1 file from v0.1 opens");
    assert_eq!(back.help.solution_applied, 0);
}

#[test]
fn aju09_the_applied_count_round_trips() {
    let mut help = HelpLog::default();
    for step in [
        HelpStep::Solution,
        HelpStep::SolutionPath,
        HelpStep::SolutionApplied,
    ] {
        help.record(step);
    }
    let (back, _) = round_trip(&capture(&r2(), None, help));
    assert_eq!(back.help.solution_applied, 1);
}

#[test]
fn aju09_applying_without_seeing_the_path_is_inconsistent() {
    let mut v = r2_file();
    v["help"]["solution"] = 1.into();
    v["help"]["solutionApplied"] = 1.into();
    assert_eq!(open(&v), Err(StateError::InconsistentReveals));
}

// ------------------------------------------------------------ AJU-10: Wallace (MT-29)

#[test]
fn aju10_files_without_wallace_still_open() {
    let mut v: Value =
        serde_json::from_str(&capture(&r2(), None, HelpLog::default()).to_json()).unwrap();
    for key in ["wallace", "wallacePath", "wallaceApplied"] {
        assert_eq!(v["help"][key], 0, "{key}");
        v["help"].as_object_mut().unwrap().remove(key);
    }
    let back = Exercise::parse(&v.to_string()).expect("a schema-1 file from v0.1 opens");
    assert_eq!(
        (
            back.help.wallace,
            back.help.wallace_path,
            back.help.wallace_applied
        ),
        (0, 0, 0)
    );
}

#[test]
fn aju10_the_wallace_counts_round_trip() {
    let mut help = HelpLog::default();
    for step in [
        HelpStep::Wallace,
        HelpStep::WallacePath,
        HelpStep::WallaceApplied,
    ] {
        help.record(step);
    }
    let (back, _) = round_trip(&capture(&r2(), None, help));
    assert_eq!(
        (
            back.help.wallace,
            back.help.wallace_path,
            back.help.wallace_applied
        ),
        (1, 1, 1)
    );
}

#[test]
fn aju10_inconsistent_wallace_reveals_are_refused() {
    for (set, why) in [
        (&[("wallacePath", 1)][..], "the path without the result"),
        (
            &[("wallace", 1), ("wallaceApplied", 1)][..],
            "applied without the path",
        ),
    ] {
        let mut v = r2_file();
        for &(key, n) in set {
            v["help"][key] = n.into();
        }
        assert_eq!(open(&v), Err(StateError::InconsistentReveals), "{why}");
    }
}
