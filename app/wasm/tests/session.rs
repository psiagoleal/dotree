// Caminho relativo: app/wasm/tests/session.rs
//! Contract of the stateful `Session` the interface drives during compression (MT-12a).
//! Times and areas travel in half units (FA = 2, HA = 1), as integers.

use dotree_wasm::Session;
use serde_json::{Value, json};

const R1: &str = "A*B + C*D + 3(E+F) + G";
const R1_WIDTHS: &str = r#"{"A":3,"B":3,"C":2,"D":2,"E":2,"F":1,"G":1}"#;

fn r1() -> Session {
    Session::open(R1, R1_WIDTHS).expect("R1 opens")
}

fn snap(s: &Session) -> Value {
    serde_json::from_str(&s.snapshot()).unwrap()
}

/// Ids of current-level points with these signals in this column.
fn ids(s: &Session, column: u64, signals: &[&str]) -> Vec<u64> {
    let v = snap(s);
    let points = v["current"]["points"].as_array().unwrap();
    signals
        .iter()
        .map(|sig| {
            points
                .iter()
                .find(|p| p["column"] == column && p["signal"] == *sig)
                .unwrap_or_else(|| panic!("no {sig} in column {column}"))["id"]
                .as_u64()
                .unwrap()
        })
        .collect()
}

fn form(s: &mut Session, column: u64, signals: &[&str]) -> Value {
    let sel = json!(ids(s, column, signals)).to_string();
    serde_json::from_str(&s.form_cell(&sel)).unwrap()
}

fn act(out: String) -> Value {
    serde_json::from_str(&out).unwrap()
}

#[test]
fn opening_r1_gives_level_0() {
    let s = r1();
    let v = snap(&s);
    assert_eq!(v["level"], 0);
    assert_eq!(v["levels"], json!([]));
    assert_eq!(v["current"]["pending"], json!([]));
    let points = v["current"]["points"].as_array().unwrap();
    assert_eq!(points.len(), 20);
    assert_eq!(
        points[1],
        json!({"id": 1, "column": 1, "ready": 0, "signal": "a0_b1", "origin": "input", "term": 0})
    );
    assert_eq!(v["heights"], json!([5, 7, 5, 2, 1, 0, 0]));
    assert_eq!(
        v["metrics"],
        json!({"fa": 0, "ha": 0, "andGates": 13, "area": 0, "delay": 0})
    );
    assert_eq!(v["canUndo"], false);
    assert_eq!(v["canRedo"], false);
}

#[test]
fn an_invalid_expression_does_not_open() {
    let err: Value =
        serde_json::from_str(&Session::open("A+", r#"{"A":1}"#).err().unwrap()).unwrap();
    assert_eq!(err["code"], "syntax");
}

#[test]
fn forming_a_cell_makes_it_pending_and_counts_it() {
    let mut s = r1();
    assert_eq!(
        form(&mut s, 1, &["c0_d1", "a1_b0", "a0_b1"]),
        json!({"ok": true, "kind": "fa"})
    );
    let v = snap(&s);
    // Inputs in canonical order whatever the selection order.
    let expected = ids(&s, 1, &["a0_b1", "a1_b0", "c0_d1"]);
    assert_eq!(
        v["current"]["pending"],
        json!([{"kind": "fa", "column": 1, "inputs": expected, "ready": 2}])
    );
    assert_eq!(v["metrics"]["fa"], 1);
    assert_eq!(v["metrics"]["area"], 2);
    assert_eq!(v["metrics"]["delay"], 2);
    assert_eq!(v["canUndo"], true);
    assert_eq!(form(&mut s, 2, &["a0_b2", "a1_b1"])["kind"], "ha");
}

#[test]
fn refusals_are_codes_and_change_nothing() {
    let mut s = r1();
    form(&mut s, 1, &["a0_b1", "a1_b0", "c0_d1"]);
    let before = s.snapshot();
    let one = ids(&s, 1, &["c1_d0"]);
    assert_eq!(
        act(s.form_cell(&json!(one).to_string())),
        json!({"ok": false, "rejection": {"code": "wrongCount", "count": 1}})
    );
    let mixed = [ids(&s, 1, &["c1_d0"]), ids(&s, 2, &["a0_b2"])].concat();
    assert_eq!(
        act(s.form_cell(&json!(mixed).to_string()))["rejection"],
        json!({"code": "mixedColumns"})
    );
    let used = [ids(&s, 1, &["a0_b1", "c1_d0"])].concat();
    assert_eq!(
        act(s.form_cell(&json!(used).to_string()))["rejection"],
        json!({"code": "alreadyInCell", "point": used[0]})
    );
    assert_eq!(
        act(s.form_cell(&json!([999, 998]).to_string()))["rejection"],
        json!({"code": "notInCurrentLevel", "point": 999})
    );
    assert_eq!(
        act(s.form_cell(&json!([one[0], one[0]]).to_string()))["rejection"],
        json!({"code": "duplicateSelection", "point": one[0]})
    );
    assert_eq!(
        act(s.form_cell("not json"))["rejection"],
        json!({"code": "badSelection"})
    );
    assert_eq!(s.snapshot(), before);

    let mut fresh = r1();
    assert_eq!(
        act(fresh.conclude_level()),
        json!({"ok": false, "rejection": {"code": "emptyLevel"}})
    );
}

#[test]
fn r2_tree_through_the_session() {
    let mut s = r1();
    for (col, sigs) in [
        (1, &["a0_b1", "a1_b0", "c0_d1"][..]),
        (1, &["c1_d0", "e1", "e0"]),
        (2, &["a0_b2", "a1_b1"]),
    ] {
        assert_eq!(form(&mut s, col, sigs)["ok"], true);
    }
    assert_eq!(act(s.conclude_level()), json!({"ok": true}));
    for (col, sigs) in [
        (1, &["f0", "s1", "s2"][..]),
        (2, &["a2_b0", "c1_d1", "e1"]),
        (2, &["s3", "k1", "k2"]),
        (3, &["a1_b2", "a2_b1", "k3"]),
    ] {
        assert_eq!(form(&mut s, col, sigs)["ok"], true);
    }
    s.conclude_level();
    for (col, sigs) in [
        (0, &["a0_b0", "c0_d0", "e0"][..]),
        (2, &["s5", "k4", "s6"]),
        (3, &["k5", "s7", "k6"]),
        (4, &["a2_b2", "k7"]),
    ] {
        assert_eq!(form(&mut s, col, sigs)["ok"], true);
    }
    s.conclude_level();

    let v = snap(&s);
    assert_eq!(v["level"], 3);
    let levels = v["levels"].as_array().unwrap();
    assert_eq!(levels.len(), 3);
    let numbers: Vec<u64> = levels
        .iter()
        .flat_map(|l| {
            l["cells"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| c["number"].as_u64().unwrap())
        })
        .collect();
    assert_eq!(numbers, (1..=11).collect::<Vec<_>>());
    // Level 0 keeps its 20 input points; HA3 consumed a0_b2 and a1_b1.
    assert_eq!(levels[0]["points"].as_array().unwrap().len(), 20);
    let ha3 = &levels[0]["cells"][2];
    assert_eq!(ha3["kind"], "ha");
    assert_eq!(ha3["column"], 2);
    assert_eq!(ha3["ready"], 1);
    // Outputs are generated points: no term, origin sum or carry.
    let s1 = v["levels"][1]["points"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["signal"] == "s1")
        .unwrap()
        .clone();
    assert_eq!(s1["origin"], "sum");
    assert_eq!(s1["term"], Value::Null);
    assert_eq!(s1["ready"], 2);
    assert_eq!(v["heights"], json!([3, 2, 1, 2, 2, 1, 0])); // CPA: FA, FA, HA, FA, FA, HA
    assert_eq!(
        v["metrics"],
        json!({"fa": 9, "ha": 2, "andGates": 13, "area": 20, "delay": 6})
    );
}

#[test]
fn undo_and_redo() {
    let mut s = r1();
    assert!(!s.undo());
    form(&mut s, 2, &["a0_b2", "a1_b1"]);
    assert!(s.undo());
    let v = snap(&s);
    assert_eq!(v["current"]["pending"], json!([]));
    assert_eq!(v["canRedo"], true);
    assert!(s.redo());
    assert_eq!(snap(&s)["current"]["pending"].as_array().unwrap().len(), 1);
    assert!(!s.redo());
}
