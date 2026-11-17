// Caminho relativo: app/wasm/tests/counters.rs
//! MT-32b (ADR 0005, CMP-20): the 7:3 and 15:4 counters through the bridge. Seven or fifteen
//! points form a counter; the snapshot, the tables, the file and the costs carry it.

use dotree_wasm::Session;
use serde_json::{Value, json};

fn seven() -> Session {
    Session::open(
        "A+B+C+D+E+F+G",
        r#"{"A":1,"B":1,"C":1,"D":1,"E":1,"F":1,"G":1}"#,
    )
    .expect("opens")
}

fn json(s: &str) -> Value {
    serde_json::from_str(s).unwrap()
}

fn all_points(s: &Session) -> String {
    let v = json(&s.snapshot());
    let ids: Vec<u64> = v["current"]["points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_u64().unwrap())
        .collect();
    json!(ids).to_string()
}

fn counted() -> Session {
    let mut s = seven();
    let out = json(&s.form_cell(&all_points(&s)));
    assert_eq!(out, json!({"ok": true, "kind": "c73"}));
    s.conclude_level();
    s
}

#[test]
fn seven_points_form_a_7_3_counter() {
    let s = counted();
    let v = json(&s.snapshot());
    assert_eq!(
        v["metrics"],
        json!({"fa": 0, "ha": 0, "c73": 1, "c154": 0, "andGates": 0, "area": 400, "delay": 300})
    );
    let cell = &v["levels"][0]["cells"][0];
    assert_eq!(cell["kind"], "c73");
    let upper = cell["upper"].as_array().unwrap();
    assert_eq!(upper.len(), 1);
    let top = v["current"]["points"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == upper[0])
        .unwrap();
    assert_eq!(
        (&top["signal"], &top["origin"], &top["column"]),
        (&json!("k1_2"), &json!("upper"), &json!(2))
    );
    assert!(v["result"].is_array(), "complete: one point per column");
}

#[test]
fn the_tables_carry_the_counter() {
    let s = counted();
    let rows = json(&s.wiring_table());
    assert_eq!(rows[0]["cell"], "C73_1");
    assert_eq!(rows[0]["inputs"].as_array().unwrap().len(), 7);
    assert_eq!(rows[0]["upper"], json!([{"name": "k1_2", "result": 2}]));
    let md = s.wiring_table_markdown(
        r#"{"cell":"Célula","level":"Nível","column":"Coluna","sum":"Soma","carry":"Vai-um","inputs":"Entradas","upper":"Bits 2 e 3"}"#,
    );
    assert!(
        md.starts_with("| Célula | Nível | Coluna | Entradas | Soma | Vai-um | Bits 2 e 3 |\n"),
        "{md}"
    );
    let heights = json(&s.height_table());
    assert_eq!(
        (&heights[0]["c73"], &heights[0]["c154"]),
        (&json!(1), &json!(0))
    );
}

#[test]
fn a_file_with_a_counter_opens_again() {
    let s = counted();
    let back = Session::restore(&s.exercise()).expect("opens");
    assert_eq!(
        json(&back.snapshot())["levels"],
        json(&s.snapshot())["levels"]
    );
}

#[test]
fn counters_take_costs_of_their_own() {
    let mut s = counted();
    let costs =
        r#"{"faArea":100,"faDelay":100,"haArea":50,"haDelay":50,"c73Area":250,"c73Delay":150}"#;
    assert!(s.set_costs(costs));
    assert_eq!(json(&s.costs()), json(costs));
    let m = &json(&s.snapshot())["metrics"];
    assert_eq!((&m["area"], &m["delay"]), (&json!(250), &json!(150)));
    // Without them, the counter follows the FA again.
    assert!(s.set_costs(r#"{"faArea":120,"faDelay":100,"haArea":50,"haDelay":50}"#));
    assert_eq!(json(&s.snapshot())["metrics"]["area"], 480);
    assert!(!s.set_costs(r#"{"faArea":100,"faDelay":100,"haArea":50,"haDelay":50,"c154Area":0}"#));
}
