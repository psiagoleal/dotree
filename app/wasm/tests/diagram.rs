// Caminho relativo: app/wasm/tests/diagram.rs
//! Contract of the block diagram and wiring table the interface draws (spec 05; MT-14a).

mod common;

use common::{r1, r2, r2_levels};
use serde_json::{Value, json};

fn parse(s: String) -> Value {
    serde_json::from_str(&s).unwrap()
}

#[test]
fn r2_diagram_is_complete_with_its_blocks_outputs_and_critical_path() {
    let d = parse(r2().diagram());
    assert_eq!(d["complete"], true);
    assert_eq!(d["outputWidth"], 7);
    assert_eq!(d["rows"], 5);
    assert_eq!(d["andGates"], 13);
    let blocks = d["blocks"].as_array().unwrap();
    assert_eq!(blocks.len(), 17);
    assert_eq!(
        blocks[1],
        json!({"cell": 2, "kind": "fa", "row": 1, "column": 1, "slot": 1})
    );
    assert_eq!(blocks[16]["row"], 4);
    assert_eq!(
        d["outputs"],
        json!(["s12", "s13", "s14", "s15", "s16", "s17", "k17"])
    );
    assert_eq!(
        d["critical"],
        json!([1, 2, 4, 6, 8, 9, 12, 13, 14, 15, 16, 17])
    );
    // Edges: from the input block, from a cell, or to a result bit.
    let edges = d["edges"].as_array().unwrap();
    assert!(edges.contains(&json!({"from": "inputs", "to": 1, "label": "a0_b1"})));
    assert!(edges.contains(&json!({"from": 1, "to": 4, "label": "s1"})));
    assert!(edges.contains(&json!({"from": 17, "to": "R6", "label": "k17"})));
    // The AND block can be expanded into its gates (DIA-02).
    let ands = d["ands"].as_array().unwrap();
    assert_eq!(ands.len(), 13);
    assert!(ands.contains(&json!({"ident": "a1_b0", "factors": ["a1", "b0"]})));
}

#[test]
fn an_incomplete_diagram_goes_up_to_the_last_concluded_level() {
    // DIA-07: pending cells are left out; no result yet.
    let d = parse(r2_levels(1).diagram());
    assert_eq!(d["complete"], false);
    assert_eq!(d["blocks"].as_array().unwrap().len(), 3);
    assert_eq!(d["outputs"], json!([]));
    assert_eq!(parse(r1().diagram())["blocks"], json!([]));
}

#[test]
fn wiring_rows_follow_spec_05() {
    let rows = parse(r2().wiring_table());
    let rows = rows.as_array().unwrap();
    assert_eq!(rows.len(), 17);
    assert_eq!(
        rows[0],
        json!({"cell": "FA1", "level": 0, "column": 1, "inputs": ["a0_b1", "a1_b0", "c0_d1"],
               "sum": "s1", "sumResult": null, "carry": "k1", "carryResult": null})
    );
    assert_eq!(
        rows[16],
        json!({"cell": "HA17", "level": null, "column": 5, "inputs": ["k11", "k16", null],
               "sum": "s17", "sumResult": 5, "carry": "k17", "carryResult": 6})
    );
}

#[test]
fn wiring_markdown_uses_the_given_headers() {
    let headers =
        r#"{"cell":"Célula","level":"Nível","column":"Coluna","sum":"Soma","carry":"Vai-um"}"#;
    let md = r2().wiring_table_markdown(headers);
    let lines: Vec<&str> = md.lines().collect();
    assert_eq!(
        lines[0],
        "| Célula | Nível | Coluna | x | y | z | Soma | Vai-um |"
    );
    assert!(
        lines.contains(
            &"| HA17 | CPA | $2^5$ | `k11` | `k16` | — | `s17` → `R(5)` | `k17` → `R(6)` |"
        )
    );
}
