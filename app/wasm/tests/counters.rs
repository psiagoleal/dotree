// Caminho relativo: app/wasm/tests/counters.rs
//! MT-32a (ADR 0005): the core has the 7:3 and 15:4 counters, but the interface only gets
//! them in MT-32b. Until then the bridge refuses 7 or 15 points with the count rejection
//! the interface already explains, and refuses files with counters.

use dotree_wasm::Session;
use serde_json::{Value, json};

fn seven() -> Session {
    Session::open(
        "A+B+C+D+E+F+G",
        r#"{"A":1,"B":1,"C":1,"D":1,"E":1,"F":1,"G":1}"#,
    )
    .expect("opens")
}

fn all_points(s: &Session) -> String {
    let v: Value = serde_json::from_str(&s.snapshot()).unwrap();
    let ids: Vec<u64> = v["current"]["points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_u64().unwrap())
        .collect();
    json!(ids).to_string()
}

#[test]
fn seven_points_are_refused_until_mt_32b() {
    let mut s = seven();
    let out: Value = serde_json::from_str(&s.form_cell(&all_points(&s))).unwrap();
    assert_eq!(
        out,
        json!({"ok": false, "rejection": {"code": "wrongCount", "count": 7}})
    );
    let v: Value = serde_json::from_str(&s.snapshot()).unwrap();
    assert_eq!(v["current"]["pending"], json!([]));
}

#[test]
fn a_file_with_a_counter_is_refused_until_mt_32b() {
    let s = seven();
    let mut file: Value = serde_json::from_str(&s.exercise()).unwrap();
    file["pending"] = json!([{"kind": "c73", "column": 0, "inputs": [0, 1, 2, 3, 4, 5, 6]}]);
    let err: Value = match Session::restore(&file.to_string()) {
        Ok(_) => panic!("accepted"),
        Err(e) => serde_json::from_str(&e).unwrap(),
    };
    assert_eq!(
        err,
        json!({"code": "inconsistentCell", "level": 0, "position": 1})
    );
}
