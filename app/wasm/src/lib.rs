// Caminho relativo: app/wasm/src/lib.rs
//! Bridge between `dotree-core` and the browser. No domain rules live here (ADR 0001): this
//! crate only converts core results into JSON for the interface. Errors and warnings travel
//! as codes with parameters; the interface words them (IDI-05).

use std::collections::BTreeMap;

use dotree_core::expr::{self, Expected, ExprError, SyntaxErrorKind, Warning};
use dotree_core::matrix::{DotMatrix, MatrixError};
use serde::Serialize;
use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

/// Core version, used by the interface to prove the WASM module loaded.
#[wasm_bindgen(js_name = coreVersion)]
pub fn core_version() -> String {
    dotree_core::version().to_owned()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TermDto {
    /// As a string: coefficients can exceed what a JS number holds exactly.
    coefficient: String,
    vars: String,
    /// Value of the term with every input at its maximum.
    max: String,
}

#[derive(Serialize)]
struct InputDto {
    var: char,
    width: u32,
    max: String,
}

#[derive(Serialize)]
struct FactorDto {
    var: char,
    bit: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DotDto {
    id: usize,
    column: u32,
    term: usize,
    signal: String,
    factors: Vec<FactorDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MatrixDto {
    terms: Vec<TermDto>,
    inputs: Vec<InputDto>,
    dots: Vec<DotDto>,
    heights: Vec<u32>,
    output_width: u32,
    v_max: String,
    and_gates: usize,
    warnings: Vec<Value>,
}

fn expected_code(e: Expected) -> &'static str {
    match e {
        Expected::Operand => "operand",
        Expected::Operator => "operator",
        Expected::CloseParen => "closeParen",
        Expected::End => "end",
    }
}

fn expr_error(e: &ExprError) -> Value {
    match e {
        ExprError::Syntax(s) => {
            let (kind, ch) = match s.kind {
                SyntaxErrorKind::Empty => ("empty", None),
                SyntaxErrorKind::UnexpectedChar(c) => ("unexpectedChar", Some(c)),
                SyntaxErrorKind::LowercaseVariable(c) => ("lowercaseVariable", Some(c)),
                SyntaxErrorKind::UnexpectedToken(c) => ("unexpectedToken", Some(c)),
                SyntaxErrorKind::UnexpectedEnd => ("unexpectedEnd", None),
                SyntaxErrorKind::ZeroCoefficient => ("zeroCoefficient", None),
                SyntaxErrorKind::NumberTooLarge => ("numberTooLarge", None),
            };
            let expected: Vec<&str> = s.expected.iter().map(|&e| expected_code(e)).collect();
            json!({"code": "syntax", "kind": kind, "char": ch, "column": s.column, "expected": expected})
        }
        ExprError::MissingWidth { var } => json!({"code": "missingWidth", "var": var}),
        ExprError::WidthOutOfRange { var, width } => json!({"code": "widthOutOfRange", "var": var,
            "width": width, "min": expr::MIN_WIDTH, "max": expr::MAX_WIDTH}),
        ExprError::RepeatedVariable { var } => json!({"code": "repeatedVariable", "var": var}),
        ExprError::ConstantTerm => json!({"code": "constantTerm"}),
        ExprError::CoefficientTooLarge => json!({"code": "coefficientTooLarge"}),
        ExprError::TooManyInputBits { total, limit } => {
            json!({"code": "tooManyInputBits", "total": total, "limit": limit})
        }
    }
}

fn failure(error: Value) -> String {
    json!({"ok": false, "error": error}).to_string()
}

/// Builds the level-0 matrix of `src` with the widths in `widths_json` (an object such as
/// `{"A": 3, "B": 2}`). Returns `{"ok": true, "matrix": …}` or `{"ok": false, "error": …}`.
#[wasm_bindgen(js_name = buildMatrix)]
pub fn build_matrix(src: &str, widths_json: &str) -> String {
    let Ok(raw) = serde_json::from_str::<BTreeMap<String, u32>>(widths_json) else {
        return failure(json!({"code": "badWidths"}));
    };
    let mut widths = BTreeMap::new();
    for (k, w) in raw {
        let mut chars = k.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => {
                widths.insert(c, w);
            }
            _ => return failure(json!({"code": "badWidths"})),
        }
    }
    let analysis = match expr::analyze(src, &widths) {
        Ok(a) => a,
        Err(e) => return failure(expr_error(&e)),
    };
    let m = match DotMatrix::build(&analysis) {
        Ok(m) => m,
        Err(MatrixError::TooManyDots { count, limit }) => {
            return failure(json!({"code": "tooManyDots", "count": count, "limit": limit}));
        }
    };
    let dto = MatrixDto {
        terms: m
            .terms
            .iter()
            .enumerate()
            .map(|(i, t)| TermDto {
                coefficient: t.coefficient.to_string(),
                vars: t.vars.iter().collect(),
                max: m.term_max(i).to_string(),
            })
            .collect(),
        inputs: m
            .widths
            .iter()
            .map(|(&var, &width)| InputDto {
                var,
                width,
                max: m.input_max(var).to_string(),
            })
            .collect(),
        dots: m
            .dots
            .iter()
            .map(|d| {
                let s = &m.signals[d.signal];
                DotDto {
                    id: d.id,
                    column: d.column,
                    term: d.term,
                    signal: s.ident(),
                    factors: s
                        .factors
                        .iter()
                        .map(|f| FactorDto {
                            var: f.var,
                            bit: f.bit,
                        })
                        .collect(),
                }
            })
            .collect(),
        heights: m.heights(),
        output_width: m.output_width,
        v_max: m.v_max.to_string(),
        and_gates: m.and_gates(),
        warnings: analysis
            .warnings
            .iter()
            .map(|w| match w {
                Warning::UnusedWidth { var } => json!({"code": "unusedWidth", "var": var}),
            })
            .collect(),
    };
    json!({"ok": true, "matrix": dto}).to_string()
}
