// Caminho relativo: app/wasm/tests/verification.rs
//! Verification and VHDL through the bridge (MT-14b): the netlist JSON that travels to the
//! worker, the chunked `Verification`, and the VHDL files.

mod common;

use common::{r1, r2, r2_levels};
use dotree_wasm::Verification;
use serde_json::Value;

fn run(v: &mut Verification, max: f64) -> Value {
    serde_json::from_str(&v.run(max)).unwrap()
}

#[test]
fn ver05_vhd07_nothing_while_incomplete() {
    assert_eq!(r1().netlist(), None);
    assert_eq!(r2_levels(3).netlist(), None);
    let out: Value = serde_json::from_str(&r2_levels(3).vhdl("dotree_circuito", "pt-br")).unwrap();
    assert_eq!(out["ok"], false);
    assert_eq!(out["error"], "notComplete");
}

#[test]
fn ver01_r2_is_exhaustive_with_no_errors() {
    let mut v = Verification::new(&r2().netlist().expect("R2 is complete")).unwrap();
    let p = run(&mut v, 1e12);
    assert_eq!(p["checked"], 16384);
    assert_eq!(p["total"], 16384);
    assert_eq!(p["coverage"], "exhaustive");
    assert_eq!(p["done"], true);
    assert_eq!(p["counterexample"], Value::Null);
}

#[test]
fn ver04_runs_in_chunks() {
    let mut v = Verification::new(&r2().netlist().unwrap()).unwrap();
    let p = run(&mut v, 1000.0);
    // Batches of 64 vectors: 1000 rounds up to 1024.
    assert_eq!(p["checked"], 1024);
    assert_eq!(p["done"], false);
    let p = run(&mut v, 1e12);
    assert_eq!(p["checked"], 16384);
    assert_eq!(p["done"], true);
}

#[test]
fn ver03_a_broken_netlist_gives_a_counterexample() {
    // FA16 gets k14 (weight 2^3) where k15 (weight 2^4) belongs, as in the core test.
    let mut n: Value = serde_json::from_str(&r2().netlist().unwrap()).unwrap();
    assert_eq!(n["cells"][15]["inputs"][2], "k15");
    n["cells"][15]["inputs"][2] = "k14".into();
    let mut v = Verification::new(&n.to_string()).unwrap();
    let p = run(&mut v, 1e12);
    assert_eq!(p["done"], true);
    let ce = &p["counterexample"];
    let inputs = ce["inputs"].as_array().unwrap();
    let vars: Vec<&str> = inputs.iter().map(|i| i["var"].as_str().unwrap()).collect();
    assert_eq!(vars, ["A", "B", "C", "D", "E", "F", "G"]);
    // Expected value from the inputs, computed here: A*B + C*D + 3(E+F) + G.
    let val = |k: &str| -> u128 {
        inputs.iter().find(|i| i["var"] == k).unwrap()["value"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap()
    };
    let expected = val("A") * val("B") + val("C") * val("D") + 3 * (val("E") + val("F")) + val("G");
    assert_eq!(ce["expected"], expected.to_string());
    let obtained: u128 = ce["obtained"].as_str().unwrap().parse().unwrap();
    assert_ne!(obtained, expected);
    let differing: Vec<u64> = ce["differing"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b.as_u64().unwrap())
        .collect();
    let mask = differing.iter().fold(0u128, |m, b| m | 1 << b);
    assert_eq!(mask, expected ^ obtained);
    assert!(p["checked"].as_u64().unwrap() <= 16384);
}

#[test]
fn a_bad_netlist_json_is_refused() {
    assert!(Verification::new("{").is_err());
    assert!(Verification::new(r#"{"cells": 3}"#).is_err());
}

fn vhdl(name: &str, lang: &str) -> Value {
    serde_json::from_str(&r2().vhdl(name, lang)).unwrap()
}

#[test]
fn vhd05_r2_files_in_both_languages() {
    let pt = vhdl("dotree_circuito", "pt-br");
    assert_eq!(pt["ok"], true);
    let names: Vec<&str> = pt["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "full_adder.vhd",
            "half_adder.vhd",
            "dotree_circuito.vhd",
            "tb_dotree_circuito.vhd"
        ]
    );
    let tb = |v: &Value| v["files"][3]["contents"].as_str().unwrap().to_owned();
    assert!(tb(&pt).contains("CASOS CORRETOS"));
    let en = vhdl("meu_somador", "en");
    assert_eq!(en["files"][2]["name"], "meu_somador.vhd");
    assert!(
        en["files"][2]["contents"]
            .as_str()
            .unwrap()
            .contains("entity meu_somador is")
    );
    assert!(tb(&en).contains("CASES CORRECT"));
}

#[test]
fn vhd08_invalid_names_are_refused() {
    for bad in [
        "",
        "9x",
        "a__b",
        "x_",
        "entity",
        "FA1",
        "s17",
        "a1_b0",
        "R",
        "half_adder",
    ] {
        let out = vhdl(bad, "pt-br");
        assert_eq!(out["ok"], false, "{bad}");
        assert_eq!(out["error"], "invalidName", "{bad}");
    }
}
