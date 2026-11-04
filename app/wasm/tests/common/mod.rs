// Caminho relativo: app/wasm/tests/common/mod.rs
//! Shared fixtures: R1 (spec 00) and the R2 tree driven through the `Session`.
#![allow(dead_code)]

use dotree_wasm::Session;
use serde_json::{Value, json};

pub const R1: &str = "A*B + C*D + 3(E+F) + G";
pub const R1_WIDTHS: &str = r#"{"A":3,"B":3,"C":2,"D":2,"E":2,"F":1,"G":1}"#;

pub fn r1() -> Session {
    Session::open(R1, R1_WIDTHS).expect("R1 opens")
}

pub fn snap(s: &Session) -> Value {
    serde_json::from_str(&s.snapshot()).unwrap()
}

pub fn form(s: &mut Session, column: u64, signals: &[&str]) {
    let v = snap(s);
    let points = v["current"]["points"].as_array().unwrap();
    let ids: Vec<Value> = signals
        .iter()
        .map(|sig| {
            points
                .iter()
                .find(|p| p["column"] == column && p["signal"] == *sig)
                .unwrap_or_else(|| panic!("no {sig} in column {column}"))["id"]
                .clone()
        })
        .collect();
    let out: Value = serde_json::from_str(&s.form_cell(&json!(ids).to_string())).unwrap();
    assert_eq!(out["ok"], true, "{signals:?}");
}

/// Levels 0 to `levels - 1` of the R2 tree (spec 00), concluded.
pub fn r2_levels(levels: usize) -> Session {
    let mut s = r1();
    let tree: [&[(u64, &[&str])]; 3] = [
        &[
            (1, &["a0_b1", "a1_b0", "c0_d1"]),
            (1, &["c1_d0", "e1", "e0"]),
            (2, &["a0_b2", "a1_b1"]),
        ],
        &[
            (1, &["f0", "s1", "s2"]),
            (2, &["a2_b0", "c1_d1", "e1"]),
            (2, &["s3", "k1", "k2"]),
            (3, &["a1_b2", "a2_b1", "k3"]),
        ],
        &[
            (0, &["a0_b0", "c0_d0", "e0"]),
            (2, &["s5", "k4", "s6"]),
            (3, &["k5", "s7", "k6"]),
            (4, &["a2_b2", "k7"]),
        ],
    ];
    for cells in &tree[..levels] {
        for (col, sigs) in *cells {
            form(&mut s, *col, sigs);
        }
        s.conclude_level();
    }
    s
}

/// R2 complete: the tree and the CPA.
pub fn r2() -> Session {
    let mut s = r2_levels(3);
    s.close_with_cpa();
    s
}
