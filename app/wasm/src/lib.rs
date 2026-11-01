// Caminho relativo: app/wasm/src/lib.rs
//! Bridge between `dotree-core` and the browser. No domain rules live here (ADR 0001): this
//! crate only converts core results into JSON for the interface. Errors and warnings travel
//! as codes with parameters; the interface words them (IDI-05).

use std::collections::BTreeMap;

use dotree_core::expr::{self, Analysis, Expected, ExprError, SyntaxErrorKind, Warning};
use dotree_core::matrix::{DotMatrix, MatrixError};
use dotree_core::reduce::{
    CellKind, CpaBlock, Origin, PendingCell, Point, PointId, Reduction, Rejection,
};
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

/// Parses the widths, analyses the expression and builds the level-0 matrix, or returns the
/// error object shared by `buildMatrix` and `Session.open`.
fn analyze_and_build(src: &str, widths_json: &str) -> Result<(Analysis, DotMatrix), Value> {
    let Ok(raw) = serde_json::from_str::<BTreeMap<String, u32>>(widths_json) else {
        return Err(json!({"code": "badWidths"}));
    };
    let mut widths = BTreeMap::new();
    for (k, w) in raw {
        let mut chars = k.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => {
                widths.insert(c, w);
            }
            _ => return Err(json!({"code": "badWidths"})),
        }
    }
    let analysis = expr::analyze(src, &widths).map_err(|e| expr_error(&e))?;
    let m = DotMatrix::build(&analysis).map_err(|e| match e {
        MatrixError::TooManyDots { count, limit } => {
            json!({"code": "tooManyDots", "count": count, "limit": limit})
        }
    })?;
    Ok((analysis, m))
}

/// Builds the level-0 matrix of `src` with the widths in `widths_json` (an object such as
/// `{"A": 3, "B": 2}`). Returns `{"ok": true, "matrix": …}` or `{"ok": false, "error": …}`.
#[wasm_bindgen(js_name = buildMatrix)]
pub fn build_matrix(src: &str, widths_json: &str) -> String {
    let (analysis, m) = match analyze_and_build(src, widths_json) {
        Ok(built) => built,
        Err(e) => return failure(e),
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

// ------------------------------------------------------------------ compression (MT-12a)

fn kind_code(k: CellKind) -> &'static str {
    match k {
        CellKind::Fa => "fa",
        CellKind::Ha => "ha",
    }
}

fn cpa_block(b: &CpaBlock) -> Value {
    match b {
        CpaBlock::ColumnTooTall { column, height } => {
            json!({"code": "columnTooTall", "column": column, "height": height})
        }
        CpaBlock::ThreeNotFirst { column } => json!({"code": "threeNotFirst", "column": column}),
        CpaBlock::NothingToAdd => json!({"code": "nothingToAdd"}),
        CpaBlock::PendingCells => json!({"code": "pendingCells"}),
    }
}

/// Refusals travel as rule codes, carrying only ids the user selected (GER-02).
fn rejection(r: &Rejection) -> Value {
    match r {
        Rejection::MixedColumns => json!({"code": "mixedColumns"}),
        Rejection::WrongCount { count } => json!({"code": "wrongCount", "count": count}),
        Rejection::AlreadyInCell { point } => json!({"code": "alreadyInCell", "point": point}),
        Rejection::NotInCurrentLevel { point } => {
            json!({"code": "notInCurrentLevel", "point": point})
        }
        Rejection::DuplicateSelection { point } => {
            json!({"code": "duplicateSelection", "point": point})
        }
        Rejection::EmptyLevel => json!({"code": "emptyLevel"}),
        Rejection::CpaUnavailable(b) => json!({"code": "cpaUnavailable", "block": cpa_block(b)}),
        Rejection::NoSuchLevel { level } => json!({"code": "noSuchLevel", "level": level}),
    }
}

fn outcome(r: Result<Value, Rejection>) -> String {
    match r {
        Ok(Value::Null) => json!({"ok": true}),
        Ok(v) => {
            let mut o = json!({"ok": true});
            o.as_object_mut()
                .unwrap()
                .extend(v.as_object().cloned().unwrap_or_default());
            o
        }
        Err(e) => json!({"ok": false, "rejection": rejection(&e)}),
    }
    .to_string()
}

/// A compression in progress (MT-12a). Every action returns JSON; `snapshot` returns the
/// whole state the interface draws. Times and areas are in half units (FA = 2, HA = 1).
#[wasm_bindgen]
pub struct Session {
    reduction: Reduction,
    /// Term of each input dot, for its colour (COR-01).
    dot_terms: Vec<usize>,
}

impl Session {
    fn point(&self, p: &Point) -> Value {
        let (origin, term) = match p.origin {
            Origin::Input { dot } => ("input", Some(self.dot_terms[dot])),
            Origin::Sum { .. } => ("sum", None),
            Origin::Carry { .. } => ("carry", None),
        };
        json!({"id": p.id, "column": p.column, "ready": p.ready.0,
               "signal": self.reduction.signal(p), "origin": origin, "term": term})
    }

    fn points(&self, ps: &[Point]) -> Vec<Value> {
        ps.iter().map(|p| self.point(p)).collect()
    }
}

fn pending(c: &PendingCell) -> Value {
    json!({"kind": kind_code(c.kind), "column": c.column, "inputs": c.inputs, "ready": c.ready.0})
}

#[wasm_bindgen]
impl Session {
    /// Opens a session on the level-0 matrix, or returns the error JSON of `buildMatrix`.
    pub fn open(src: &str, widths_json: &str) -> Result<Session, String> {
        let (_, m) = analyze_and_build(src, widths_json).map_err(|e| e.to_string())?;
        Ok(Session {
            reduction: Reduction::new(&m),
            dot_terms: m.dots.iter().map(|d| d.term).collect(),
        })
    }

    pub fn snapshot(&self) -> String {
        let r = &self.reduction;
        let levels: Vec<Value> = r
            .levels()
            .iter()
            .map(|l| {
                let cells: Vec<Value> = l
                    .cells
                    .iter()
                    .map(|c| {
                        json!({"number": c.number, "kind": kind_code(c.kind), "column": c.column,
                               "inputs": c.inputs, "sum": c.sum, "carry": c.carry,
                               "ready": c.ready.0})
                    })
                    .collect();
                json!({"points": self.points(&l.points), "cells": cells, "cpa": l.cpa})
            })
            .collect();
        let m = r.metrics();
        let critical = r.critical();
        json!({
            "level": r.level(),
            "levels": levels,
            "current": {
                "points": self.points(r.points()),
                "pending": r.pending().iter().map(pending).collect::<Vec<_>>(),
            },
            "heights": r.heights(),
            "metrics": {"fa": m.fa, "ha": m.ha, "andGates": m.and_gates,
                        "area": m.area_halves(), "delay": m.delay.0},
            "canUndo": r.can_undo(),
            "canRedo": r.can_redo(),
            // Why "close with CPA" is disabled, or null when it is available (CMP-10).
            "cpa": r.cpa_status().err().as_ref().map(cpa_block),
            // Result bits R_0..R_{W-1} once complete; null marks a constant 0 (CMP-12).
            "result": r.result(),
            // Shown only on request (CMP-15); pending cells by index into current.pending.
            "critical": {"cells": critical.cells, "pending": critical.pending},
        })
        .to_string()
    }

    /// Forms an FA or HA from the selected point ids (a JSON array).
    #[wasm_bindgen(js_name = formCell)]
    pub fn form_cell(&mut self, selection_json: &str) -> String {
        let Ok(selection) = serde_json::from_str::<Vec<PointId>>(selection_json) else {
            return json!({"ok": false, "rejection": {"code": "badSelection"}}).to_string();
        };
        outcome(
            self.reduction
                .form_cell(&selection)
                .map(|k| json!({"kind": kind_code(k)})),
        )
    }

    #[wasm_bindgen(js_name = concludeLevel)]
    pub fn conclude_level(&mut self) -> String {
        outcome(self.reduction.conclude_level().map(|()| Value::Null))
    }

    /// Closes the current level with a ripple-carry adder (CMP-11).
    #[wasm_bindgen(js_name = closeWithCpa)]
    pub fn close_with_cpa(&mut self) -> String {
        outcome(self.reduction.close_with_cpa().map(|()| Value::Null))
    }

    /// Goes back to concluded level `level` (CMP-04).
    #[wasm_bindgen(js_name = goToLevel)]
    pub fn go_to_level(&mut self, level: usize) -> String {
        outcome(self.reduction.go_to_level(level).map(|()| Value::Null))
    }

    pub fn undo(&mut self) -> bool {
        self.reduction.undo()
    }

    pub fn redo(&mut self) -> bool {
        self.reduction.redo()
    }
}
