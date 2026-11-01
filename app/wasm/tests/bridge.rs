// Caminho relativo: app/wasm/tests/bridge.rs
//! Contract of the JSON the interface receives from `buildMatrix` (MT-11).

use dotree_wasm::build_matrix;
use serde_json::{Value, json};

fn call(src: &str, widths: Value) -> Value {
    serde_json::from_str(&build_matrix(src, &widths.to_string())).expect("valid JSON")
}

#[test]
fn r1_matrix() {
    let v = call(
        "A*B + C*D + 3(E+F) + G",
        json!({"A": 3, "B": 3, "C": 2, "D": 2, "E": 2, "F": 1, "G": 1}),
    );
    assert_eq!(v["ok"], true);
    let m = &v["matrix"];
    assert_eq!(m["heights"], json!([5, 7, 5, 2, 1, 0, 0]));
    assert_eq!(m["outputWidth"], 7);
    assert_eq!(m["vMax"], "71");
    assert_eq!(m["andGates"], 13);
    assert_eq!(m["dots"].as_array().unwrap().len(), 20);
    assert_eq!(
        m["terms"],
        json!([
            {"coefficient": "1", "vars": "AB"},
            {"coefficient": "1", "vars": "CD"},
            {"coefficient": "3", "vars": "E"},
            {"coefficient": "3", "vars": "F"},
            {"coefficient": "1", "vars": "G"}
        ])
    );
    // First dot of column 1, in canonical order.
    let d = m["dots"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["column"] == 1)
        .unwrap();
    assert_eq!(
        *d,
        json!({"id": 1, "column": 1, "term": 0, "signal": "a0_b1",
        "factors": [{"var": "A", "bit": 0}, {"var": "B", "bit": 1}]})
    );
    assert_eq!(m["warnings"], json!([]));
}

#[test]
fn warnings_and_errors_are_codes() {
    let v = call("A", json!({"A": 2, "Z": 3}));
    assert_eq!(
        v["matrix"]["warnings"],
        json!([{"code": "unusedWidth", "var": "Z"}])
    );

    assert_eq!(
        call("A+", json!({"A": 1}))["error"],
        json!({"code": "syntax", "kind": "unexpectedEnd", "char": null, "column": 3,
               "expected": ["operand"]})
    );
    assert_eq!(
        call("a*b", json!({}))["error"],
        json!({"code": "syntax", "kind": "lowercaseVariable", "char": "a", "column": 1,
               "expected": ["operand"]})
    );
    assert_eq!(
        call("A*B", json!({"A": 1}))["error"],
        json!({"code": "missingWidth", "var": "B"})
    );
    assert_eq!(
        call("A", json!({"A": 17}))["error"],
        json!({"code": "widthOutOfRange", "var": "A", "width": 17, "min": 1, "max": 16})
    );
    assert_eq!(
        call("A*A", json!({"A": 1}))["error"],
        json!({"code": "repeatedVariable", "var": "A"})
    );
    assert_eq!(
        call("A + 5", json!({"A": 1}))["error"],
        json!({"code": "constantTerm"})
    );
    assert_eq!(
        call("A*B + C", json!({"A": 16, "B": 16, "C": 1}))["error"],
        json!({"code": "tooManyInputBits", "total": 33, "limit": 32})
    );
    assert_eq!(
        call("3AB + C", json!({"A": 16, "B": 8, "C": 1}))["error"],
        json!({"code": "tooManyDots", "count": 257, "limit": 256})
    );
    assert_eq!(
        call("A", json!("not an object"))["error"],
        json!({"code": "badWidths"})
    );
}
