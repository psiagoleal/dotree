// Caminho relativo: app/wasm/src/lib.rs
//! Bridge between `dotree-core` and the browser. No domain rules live here (ADR 0001): this
//! crate only converts core results into JSON for the interface. Errors and warnings travel
//! as codes with parameters; the interface words them (IDI-05).

use std::collections::BTreeMap;

use dotree_core::board::{self, BoardLimit};
use dotree_core::diagram::{self, Endpoint, Layout, TableHeaders};
use dotree_core::expr::{self, Analysis, Expected, ExprError, SyntaxErrorKind, Warning};
use dotree_core::help::{self, HelpLog, HelpStep, Hint, Method, NextMove};
use dotree_core::matrix::{DotMatrix, MatrixError};
use dotree_core::netlist::Netlist;
use dotree_core::reduce::{
    CellKind, CpaBlock, Origin, PendingCell, Point, PointId, Reduction, Rejection,
};
use dotree_core::state::{Exercise, StateError, WidthReveal};
use dotree_core::verify::{Coverage, Progress, Verifier};
use dotree_core::vhdl::{self, Language, VhdlError};
use serde::Serialize;
use serde_json::{Value, json};
use wasm_bindgen::prelude::*;

mod netcodec;

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
        ExprError::NoVariables => json!({"code": "noVariables"}),
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
fn parse_widths(widths_json: &str) -> Result<BTreeMap<char, u32>, Value> {
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
    Ok(widths)
}

fn analyze_and_build(src: &str, widths_json: &str) -> Result<(Analysis, DotMatrix), Value> {
    let widths = parse_widths(widths_json)?;
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

/// The method of an automatic solution (AJU-05, AJU-10).
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolutionMethod {
    Dadda,
    Wallace,
}

impl SolutionMethod {
    fn core(self) -> Method {
        match self {
            SolutionMethod::Dadda => Method::Dadda,
            SolutionMethod::Wallace => Method::Wallace,
        }
    }

    /// The three counted steps of this method: result, path, applied.
    fn steps(self) -> [HelpStep; 3] {
        match self {
            SolutionMethod::Dadda => [
                HelpStep::Solution,
                HelpStep::SolutionPath,
                HelpStep::SolutionApplied,
            ],
            SolutionMethod::Wallace => [
                HelpStep::Wallace,
                HelpStep::WallacePath,
                HelpStep::WallaceApplied,
            ],
        }
    }

    /// How many times result, path and applied were counted for this method.
    fn counts(self, l: &HelpLog) -> [u32; 3] {
        match self {
            SolutionMethod::Dadda => [l.solution, l.solution_path, l.solution_applied],
            SolutionMethod::Wallace => [l.wallace, l.wallace_path, l.wallace_applied],
        }
    }
}

/// A compression in progress (MT-12a). Every action returns JSON; `snapshot` returns the
/// whole state the interface draws. Times and areas are in half units (FA = 2, HA = 1).
#[wasm_bindgen]
pub struct Session {
    reduction: Reduction,
    /// Term of each input dot, for its colour (COR-01).
    dot_terms: Vec<usize>,
    /// The level-0 matrix: the automatic solution and the lower bound start from it.
    matrix: DotMatrix,
    /// Helps revealed so far (AJU-07); outside the undo history.
    log: HelpLog,
    /// What the exercise was opened with, for the exercise file (SAL-02).
    expression: String,
    widths: BTreeMap<char, u32>,
    /// The width challenge, once answered or given up (EXP-10); outside the undo history.
    width: Option<WidthReveal>,
}

impl Session {
    fn with(
        reduction: Reduction,
        matrix: DotMatrix,
        expression: &str,
        widths: &BTreeMap<char, u32>,
    ) -> Session {
        Session {
            reduction,
            dot_terms: matrix.dots.iter().map(|d| d.term).collect(),
            matrix,
            log: HelpLog::default(),
            expression: expression.into(),
            widths: widths.clone(),
            width: None,
        }
    }

    fn bound_json(&self) -> String {
        let b = help::lower_bound(&self.matrix);
        json!({"dots": b.dots, "outputWidth": b.output_width, "value": b.value}).to_string()
    }

    fn solution_session(&self, method: SolutionMethod) -> Session {
        Session::with(
            help::solution(&self.matrix, method.core()),
            self.matrix.clone(),
            &self.expression,
            &self.widths,
        )
    }

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

fn state_error(e: &StateError) -> Value {
    match e {
        StateError::Malformed(detail) => json!({"code": "malformed", "detail": detail}),
        StateError::NotAnExercise => json!({"code": "notExercise"}),
        StateError::OldSchema { found } => json!({"code": "oldSchema", "found": found}),
        StateError::FutureSchema { found } => json!({"code": "futureSchema", "found": found}),
        StateError::BadExpression => json!({"code": "badExpression"}),
        StateError::InconsistentCell { level, position } => {
            json!({"code": "inconsistentCell", "level": level, "position": position})
        }
        StateError::InconsistentLevel { level } => {
            json!({"code": "inconsistentLevel", "level": level})
        }
        StateError::InconsistentReveals => json!({"code": "inconsistentReveals"}),
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
        let widths = parse_widths(widths_json).expect("checked by analyze_and_build");
        Ok(Session::with(Reduction::new(&m), m, src, &widths))
    }

    // ------------------------------------------------------------ exercise file (spec 08)

    /// The exercise as a JSON file (SAL-02).
    pub fn exercise(&self) -> String {
        Exercise::capture(
            &self.expression,
            &self.widths,
            &self.reduction,
            self.width,
            self.log,
        )
        .to_json()
    }

    /// Opens an exercise file, or returns why not as JSON `{"code": …}` (SAL-03).
    pub fn restore(file: &str) -> Result<Session, String> {
        let open = || {
            let ex = Exercise::parse(file)?;
            let (m, r) = ex.restore()?;
            let mut s = Session::with(r, m, &ex.expression, &ex.widths);
            s.log = ex.help;
            s.width = ex.reveals.width;
            Ok(s)
        };
        open().map_err(|e: StateError| state_error(&e).to_string())
    }

    pub fn expression(&self) -> String {
        self.expression.clone()
    }

    /// The widths the exercise was opened with, as a JSON object.
    pub fn widths(&self) -> String {
        let w: BTreeMap<String, u32> = self
            .widths
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect();
        json!(w).to_string()
    }

    /// The width challenge was answered (`Some`) or given up (`None`); only the first
    /// call counts, as the challenge closes after it (EXP-10).
    #[wasm_bindgen(js_name = revealWidth)]
    pub fn reveal_width(&mut self, answer: Option<u32>) {
        self.width.get_or_insert(WidthReveal {
            answer,
            path: false,
        });
    }

    /// Step 2 of the width (GER-03); false while step 1 has not happened.
    #[wasm_bindgen(js_name = revealWidthPath)]
    pub fn reveal_width_path(&mut self) -> bool {
        self.width.as_mut().map(|w| w.path = true).is_some()
    }

    /// What was revealed so far, so a reopened exercise shows it again without asking.
    pub fn revealed(&self) -> String {
        let stage = |result: u32, path: u32| match (result, path) {
            (0, _) => "none",
            (_, 0) => "result",
            _ => "path",
        };
        let l = &self.log;
        json!({"width": self.width,
               "lowerBound": stage(l.lower_bound, l.lower_bound_path),
               "solution": stage(l.solution, l.solution_path),
               "wallace": stage(l.wallace, l.wallace_path)})
        .to_string()
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

    // ------------------------------------------------------------ help (spec 03, MT-13)
    // Each call is a reveal: the interface calls only after the user confirmed (AJU-02).

    /// The next hint that applies now, cycling through them on each request; `null` when
    /// none applies (complete circuit), which is not counted.
    pub fn hint(&mut self) -> String {
        let list = help::hints(&self.reduction);
        if list.is_empty() {
            return json!({"hint": null}).to_string();
        }
        let shown = list[self.log.hints as usize % list.len()];
        self.log.record(HelpStep::Hint);
        let hint = match shown {
            Hint::MinLevelsLeft { height, levels } => {
                json!({"code": "minLevelsLeft", "height": height, "levels": levels})
            }
            Hint::TallestColumn { column, height } => {
                json!({"code": "tallestColumn", "column": column, "height": height})
            }
            Hint::IdlePoints => json!({"code": "idlePoints"}),
            Hint::CpaReady => json!({"code": "cpaReady"}),
        };
        json!({ "hint": hint }).to_string()
    }

    /// What Dadda's method would do now (AJU-04). Forms nothing.
    #[wasm_bindgen(js_name = nextMove)]
    pub fn next_move(&mut self) -> String {
        let out = match help::next_move(&self.reduction) {
            NextMove::FormCell { kind, points } => {
                json!({"move": "formCell", "kind": kind_code(kind), "points": points})
            }
            NextMove::ConcludeLevel => json!({"move": "concludeLevel"}),
            NextMove::CloseWithCpa => json!({"move": "closeWithCpa"}),
            NextMove::Complete => return json!({"move": "complete"}).to_string(),
        };
        self.log.record(HelpStep::NextMove);
        out.to_string()
    }

    /// The lower bound and the numbers behind it (AJU-06); `path` counts step 2.
    #[wasm_bindgen(js_name = lowerBound)]
    pub fn lower_bound(&mut self, path: bool) -> String {
        self.log.record(if path {
            HelpStep::LowerBoundPath
        } else {
            HelpStep::LowerBound
        });
        self.bound_json()
    }

    /// The lower bound again, uncounted, once it was revealed; `None` before that (GER-01).
    #[wasm_bindgen(js_name = shownLowerBound)]
    pub fn shown_lower_bound(&self) -> Option<String> {
        (self.log.lower_bound > 0).then(|| self.bound_json())
    }

    /// The solution of `method` from level 0 as a separate session (AJU-03, AJU-05,
    /// AJU-10); `path` counts step 2. The student's circuit is untouched.
    pub fn solution(&mut self, path: bool, method: SolutionMethod) -> Session {
        let [result, step2, _] = method.steps();
        self.log.record(if path { step2 } else { result });
        self.solution_session(method)
    }

    /// The solution again, uncounted, once it was revealed; `None` before that (GER-01).
    #[wasm_bindgen(js_name = shownSolution)]
    pub fn shown_solution(&self, method: SolutionMethod) -> Option<Session> {
        (method.counts(&self.log)[0] > 0).then(|| self.solution_session(method))
    }

    /// Replaces the student's circuit with the solution of `method` as one undo step
    /// (AJU-09, AJU-10). False, with nothing changed or counted, before step 2 of that
    /// method's solution was revealed or when the circuit already is the solution.
    #[wasm_bindgen(js_name = applySolution)]
    pub fn apply_solution(&mut self, method: SolutionMethod) -> bool {
        if method.counts(&self.log)[1] == 0 {
            return false;
        }
        let applied = self
            .reduction
            .adopt(&help::solution(&self.matrix, method.core()));
        if applied {
            self.log.record(method.steps()[2]);
        }
        applied
    }

    // ------------------------------------------------------------ diagram (spec 05, MT-14a)

    /// Block diagram of the concluded levels (DIA-01 to DIA-03, DIA-05, DIA-07). Endpoints
    /// are `"inputs"`, a cell number or `"R<j>"`; `ands` lists the AND gates (DIA-02).
    pub fn diagram(&self) -> String {
        let netlist = Netlist::partial(&self.matrix, &self.reduction);
        let l = Layout::new(&netlist);
        let end = |e: Endpoint| match e {
            Endpoint::Inputs => json!("inputs"),
            Endpoint::Cell(n) => json!(n),
            Endpoint::Output(j) => json!(format!("R{j}")),
        };
        let blocks: Vec<Value> = l
            .blocks
            .iter()
            .map(|b| {
                json!({"cell": b.cell, "kind": kind_code(b.kind), "row": b.row,
                       "column": b.column, "slot": b.slot})
            })
            .collect();
        let edges: Vec<Value> = l
            .edges
            .iter()
            .map(|e| json!({"from": end(e.from), "to": end(e.to), "label": e.label}))
            .collect();
        let ands: Vec<Value> = netlist
            .signals
            .iter()
            .filter(|s| s.is_and())
            .map(|s| {
                let factors: Vec<String> = s
                    .factors
                    .iter()
                    .map(|f| format!("{}{}", f.var.to_ascii_lowercase(), f.bit))
                    .collect();
                json!({"ident": s.ident(), "factors": factors})
            })
            .collect();
        json!({
            "complete": l.complete,
            "outputWidth": l.output_width,
            "rows": l.rows,
            "andGates": l.and_gates,
            "blocks": blocks,
            "edges": edges,
            "outputs": l.outputs,
            "critical": l.critical,
            "ands": ands,
        })
        .to_string()
    }

    /// Wiring table rows of the concluded levels (DIA-04).
    #[wasm_bindgen(js_name = wiringTable)]
    pub fn wiring_table(&self) -> String {
        let netlist = Netlist::partial(&self.matrix, &self.reduction);
        let rows: Vec<Value> = diagram::wiring_table(&netlist)
            .into_iter()
            .map(|r| {
                json!({"cell": r.cell, "level": r.level, "column": r.column,
                       "inputs": r.inputs, "sum": r.sum, "sumResult": r.sum_result,
                       "carry": r.carry, "carryResult": r.carry_result})
            })
            .collect();
        Value::from(rows).to_string()
    }

    /// The wiring table as Markdown (DIA-06), with headers in the interface's language:
    /// `{"cell", "level", "column", "sum", "carry"}`.
    #[wasm_bindgen(js_name = wiringTableMarkdown)]
    pub fn wiring_table_markdown(&self, headers_json: &str) -> String {
        let h: Value = serde_json::from_str(headers_json).unwrap_or_default();
        let get = |k: &str| h[k].as_str().unwrap_or(k).to_owned();
        let (cell, level, column, sum, carry) = (
            get("cell"),
            get("level"),
            get("column"),
            get("sum"),
            get("carry"),
        );
        let netlist = Netlist::partial(&self.matrix, &self.reduction);
        diagram::wiring_table_markdown(
            &diagram::wiring_table(&netlist),
            TableHeaders {
                cell: &cell,
                level: &level,
                column: &column,
                sum: &sum,
                carry: &carry,
            },
        )
    }

    // ------------------------------------------------- verification and VHDL (MT-14b)

    /// The complete circuit as JSON for [`Verification`], or `undefined` while incomplete
    /// (VER-05).
    pub fn netlist(&self) -> Option<String> {
        Netlist::from_reduction(&self.matrix, &self.reduction)
            .ok()
            .map(|n| netcodec::encode(&n).to_string())
    }

    /// The VHDL files (spec 06) in `language` (`"pt-br"` or `"en"`):
    /// `{"ok": true, "files": [{"name", "contents"}], "board"}`, or `{"ok": false, "error"}`
    /// with `"notComplete"` (VHD-07) or `"invalidName"` (VHD-08). `board` says whether the
    /// DE2 files are among them (spec 10): `{"fits": true}`, or `{"fits": false, "reason":
    /// "tooManyInputs", "bits", "max"}` / `{"fits": false, "reason": "tooWide", "width",
    /// "max"}` (PLA-01).
    pub fn vhdl(&self, name: &str, language: &str) -> String {
        let language = if language == "en" {
            Language::En
        } else {
            Language::PtBr
        };
        let netlist = Netlist::partial(&self.matrix, &self.reduction);
        match vhdl::generate(&self.matrix, &netlist, name, language) {
            Ok(files) => {
                let files: Vec<Value> = files
                    .into_iter()
                    .map(|f| json!({"name": f.name, "contents": f.contents}))
                    .collect();
                let board = match board::fits(&netlist) {
                    Ok(()) => json!({"fits": true}),
                    Err(BoardLimit::TooManyInputs(bits)) => json!({"fits": false,
                        "reason": "tooManyInputs", "bits": bits, "max": board::MAX_INPUT_BITS}),
                    Err(BoardLimit::TooWide(width)) => json!({"fits": false,
                        "reason": "tooWide", "width": width, "max": board::MAX_OUTPUT_WIDTH}),
                };
                json!({"ok": true, "files": files, "board": board})
            }
            Err(e) => json!({"ok": false, "error": match e {
                VhdlError::NotComplete => "notComplete",
                VhdlError::InvalidName => "invalidName",
            }}),
        }
        .to_string()
    }

    #[wasm_bindgen(js_name = helpLog)]
    pub fn help_log(&self) -> String {
        let l = &self.log;
        json!({"hints": l.hints, "nextMoves": l.next_moves, "lowerBound": l.lower_bound,
               "lowerBoundPath": l.lower_bound_path, "solution": l.solution,
               "solutionPath": l.solution_path, "solutionApplied": l.solution_applied,
               "wallace": l.wallace, "wallacePath": l.wallace_path,
               "wallaceApplied": l.wallace_applied})
        .to_string()
    }

    pub fn undo(&mut self) -> bool {
        self.reduction.undo()
    }

    pub fn redo(&mut self) -> bool {
        self.reduction.redo()
    }
}

/// A verification run (spec 04), driven in chunks by the worker so it can report progress
/// and be cancelled (VER-04). Built from the JSON of [`Session::netlist`].
#[wasm_bindgen]
pub struct Verification {
    verifier: Verifier,
}

#[wasm_bindgen]
impl Verification {
    #[wasm_bindgen(constructor)]
    pub fn new(netlist_json: &str) -> Result<Verification, String> {
        Ok(Verification {
            verifier: Verifier::new(netcodec::decode(netlist_json)?),
        })
    }

    /// Checks up to `max_vectors` more vectors and returns the progress JSON:
    /// `{checked, total, coverage, done, counterexample}`.
    pub fn run(&mut self, max_vectors: f64) -> String {
        // `as` saturates: a huge or infinite budget runs to the end.
        progress_json(&self.verifier.run(max_vectors.max(1.0) as u64))
    }
}

fn progress_json(p: &Progress) -> String {
    let counterexample = p.counterexample.as_ref().map(|ce| {
        let inputs: Vec<Value> = ce
            .inputs
            .iter()
            .map(|(v, x)| json!({"var": v.to_string(), "value": x.to_string()}))
            .collect();
        let diff = ce.differing_bits();
        let differing: Vec<u32> = (0..128).filter(|b| diff >> b & 1 == 1).collect();
        json!({"inputs": inputs, "expected": ce.expected.to_string(),
               "obtained": ce.obtained.to_string(), "differing": differing})
    });
    json!({
        "checked": p.checked,
        "total": p.total,
        "coverage": match p.coverage {
            Coverage::Exhaustive => "exhaustive",
            Coverage::Sample => "sample",
        },
        "done": p.done(),
        "counterexample": counterexample,
    })
    .to_string()
}
