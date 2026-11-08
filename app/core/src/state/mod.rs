// Caminho relativo: app/core/src/state/mod.rs
//! An exercise as a JSON file (spec 08): expression, widths, cells level by level, the
//! reveals already made and the help log (SAL-02).
//!
//! A cell names its inputs by their position in the level's points, in canonical order,
//! never by internal point id: going back to a level allocates new ids, so ids would not
//! survive a rebuild. Opening a file replays it through the same rules the student plays
//! by, so a cell the rules refuse is refused here too (SAL-03).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::expr;
use crate::help::HelpLog;
use crate::matrix::DotMatrix;
use crate::reduce::{CellKind, PointId, Reduction};

/// Tag that tells an exercise file from any other JSON.
pub const FORMAT: &str = "dotree-exercise";
/// Version of the file layout; any change to it bumps this number.
pub const SCHEMA: u32 = 1;

/// The width challenge (EXP-09, EXP-10): the answer given, or `None` when the user gave up
/// and asked for W; `path` once step 2 was revealed (GER-03).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WidthReveal {
    pub answer: Option<u32>,
    pub path: bool,
}

/// Reveals that are not in the help log. Lower bound and solution follow from the log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Reveals {
    pub width: Option<WidthReveal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedCell {
    #[serde(with = "kind_code")]
    pub kind: CellKind,
    pub column: u32,
    /// Positions in the level's points (canonical order).
    pub inputs: Vec<usize>,
}

/// A concluded level. The CPA's cells follow from the level's points, so a CPA level has
/// none of its own.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedLevel {
    pub cpa: bool,
    pub cells: Vec<SavedCell>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exercise {
    pub format: String,
    pub schema: u32,
    pub expression: String,
    pub widths: BTreeMap<char, u32>,
    pub levels: Vec<SavedLevel>,
    /// Cells formed on the current level, in formation order.
    pub pending: Vec<SavedCell>,
    pub reveals: Reveals,
    pub help: HelpLog,
}

/// Why a file cannot be opened (SAL-03).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    /// Not JSON, or JSON without the fields of an exercise; the detail is the parser's.
    Malformed(String),
    /// JSON of something else.
    NotAnExercise,
    OldSchema {
        found: u64,
    },
    FutureSchema {
        found: u64,
    },
    /// Expression and widths do not build a matrix.
    BadExpression,
    /// A cell the rules refuse; `position` counts from 1 within its level.
    InconsistentCell {
        level: usize,
        position: usize,
    },
    /// An empty level, a CPA where it cannot go, or work after the CPA.
    InconsistentLevel {
        level: usize,
    },
    /// A reveal that cannot happen, such as step 2 without step 1.
    InconsistentReveals,
}

mod kind_code {
    use super::CellKind;
    use serde::{Deserialize, Deserializer, Serializer, de::Error};

    pub fn serialize<S: Serializer>(k: &CellKind, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(match k {
            CellKind::Fa => "fa",
            CellKind::Ha => "ha",
        })
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<CellKind, D::Error> {
        match String::deserialize(d)?.as_str() {
            "fa" => Ok(CellKind::Fa),
            "ha" => Ok(CellKind::Ha),
            k => Err(D::Error::custom(format!("unknown cell kind {k:?}"))),
        }
    }
}

/// Position of each point id in `points`.
fn positions(points: &[crate::reduce::Point]) -> BTreeMap<PointId, usize> {
    points.iter().enumerate().map(|(i, p)| (p.id, i)).collect()
}

fn saved(
    kind: CellKind,
    column: u32,
    inputs: &[PointId],
    at: &BTreeMap<PointId, usize>,
) -> SavedCell {
    SavedCell {
        kind,
        column,
        inputs: inputs.iter().map(|id| at[id]).collect(),
    }
}

impl Exercise {
    pub fn capture(
        expression: &str,
        widths: &BTreeMap<char, u32>,
        reduction: &Reduction,
        width: Option<WidthReveal>,
        help: HelpLog,
    ) -> Exercise {
        let levels = reduction
            .levels()
            .iter()
            .map(|l| {
                let at = positions(&l.points);
                SavedLevel {
                    cpa: l.cpa,
                    cells: if l.cpa {
                        Vec::new()
                    } else {
                        l.cells
                            .iter()
                            .map(|c| saved(c.kind, c.column, &c.inputs, &at))
                            .collect()
                    },
                }
            })
            .collect();
        let at = positions(reduction.points());
        Exercise {
            format: FORMAT.into(),
            schema: SCHEMA,
            expression: expression.into(),
            widths: widths.clone(),
            levels,
            pending: reduction
                .pending()
                .iter()
                .map(|c| saved(c.kind, c.column, &c.inputs, &at))
                .collect(),
            reveals: Reveals { width },
            help,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("plain data always serializes")
    }

    /// Reads a file, checking its tag and schema before its fields.
    pub fn parse(json: &str) -> Result<Exercise, StateError> {
        let malformed = |e: serde_json::Error| StateError::Malformed(e.to_string());
        let v: Value = serde_json::from_str(json).map_err(malformed)?;
        let Some(o) = v.as_object() else {
            return Err(StateError::Malformed("not a JSON object".into()));
        };
        if o.get("format").and_then(Value::as_str) != Some(FORMAT) {
            return Err(StateError::NotAnExercise);
        }
        let Some(found) = o.get("schema").and_then(Value::as_u64) else {
            return Err(StateError::Malformed("missing or invalid schema".into()));
        };
        match found.cmp(&u64::from(SCHEMA)) {
            std::cmp::Ordering::Less => return Err(StateError::OldSchema { found }),
            std::cmp::Ordering::Greater => return Err(StateError::FutureSchema { found }),
            std::cmp::Ordering::Equal => {}
        }
        serde_json::from_value(v).map_err(malformed)
    }

    /// Rebuilds the matrix and replays every cell through the rules.
    pub fn restore(&self) -> Result<(DotMatrix, Reduction), StateError> {
        let h = &self.help;
        let width_ok = self
            .reveals
            .width
            .is_none_or(|w| w.answer.is_none_or(|a| a >= 1));
        if !width_ok
            || (h.lower_bound_path > 0 && h.lower_bound == 0)
            || (h.solution_path > 0 && h.solution == 0)
            || (h.solution_applied > 0 && h.solution_path == 0)
        {
            return Err(StateError::InconsistentReveals);
        }
        let analysis =
            expr::analyze(&self.expression, &self.widths).map_err(|_| StateError::BadExpression)?;
        let matrix = DotMatrix::build(&analysis).map_err(|_| StateError::BadExpression)?;
        let mut r = Reduction::new(&matrix);

        for (level, saved) in self.levels.iter().enumerate() {
            let bad_level = StateError::InconsistentLevel { level };
            if r.is_complete() {
                return Err(bad_level); // nothing goes after the CPA
            }
            if saved.cpa {
                if !saved.cells.is_empty() {
                    return Err(bad_level);
                }
                r.close_with_cpa().map_err(|_| bad_level)?;
            } else {
                replay(&mut r, level, &saved.cells)?;
                r.conclude_level().map_err(|_| bad_level)?;
            }
        }
        if !self.pending.is_empty() && r.is_complete() {
            return Err(StateError::InconsistentLevel {
                level: self.levels.len(),
            });
        }
        replay(&mut r, self.levels.len(), &self.pending)?;
        Ok((matrix, r))
    }
}

/// Forms `cells` on the current level, checking each against the rules and its own fields.
fn replay(r: &mut Reduction, level: usize, cells: &[SavedCell]) -> Result<(), StateError> {
    for (i, c) in cells.iter().enumerate() {
        let bad = StateError::InconsistentCell {
            level,
            position: i + 1,
        };
        let ids: Vec<PointId> = c
            .inputs
            .iter()
            .map(|&k| r.points().get(k).map(|p| p.id).ok_or(bad.clone()))
            .collect::<Result<_, _>>()?;
        let kind = r.form_cell(&ids).map_err(|_| bad.clone())?;
        let formed = r.pending().last().expect("a cell was just formed");
        if kind != c.kind || formed.column != c.column {
            return Err(bad);
        }
    }
    Ok(())
}
