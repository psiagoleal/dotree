// Caminho relativo: app/wasm/src/netcodec.rs
//! JSON form of a [`Netlist`], so the finished circuit can travel to the verification
//! worker (VER-04), which runs its own copy of this module. Wires are `"i<signal>"`,
//! `"s<n>"` and `"k<n>"`; coefficients travel as strings, since they may not fit a JS number.

use dotree_core::expr::{self, Monomial};
use dotree_core::matrix::{Factor, Signal};
use dotree_core::netlist::{NetCell, Netlist, Wire};
use dotree_core::reduce::CellKind;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NetJson {
    variables: Vec<(char, u32)>,
    terms: Vec<TermJson>,
    signals: Vec<Vec<(char, u32)>>,
    cells: Vec<CellJson>,
    result: Vec<Option<String>>,
    output_width: u32,
    complete: bool,
}

#[derive(Serialize, Deserialize)]
struct TermJson {
    coefficient: String,
    vars: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CellJson {
    number: usize,
    kind: String,
    level: usize,
    cpa: bool,
    column: u32,
    inputs: Vec<String>,
    has_carry: bool,
}

fn wire_code(w: Wire) -> String {
    match w {
        Wire::Input(i) => format!("i{i}"),
        Wire::Sum(n) => format!("s{n}"),
        Wire::Carry(n) => format!("k{n}"),
    }
}

fn parse_wire(code: &str) -> Result<Wire, String> {
    let bad = || format!("bad wire {code:?}");
    let (tag, rest) = code.split_at_checked(1).ok_or_else(bad)?;
    let n: usize = rest.parse().map_err(|_| bad())?;
    match tag {
        "i" => Ok(Wire::Input(n)),
        "s" => Ok(Wire::Sum(n)),
        "k" => Ok(Wire::Carry(n)),
        _ => Err(bad()),
    }
}

pub fn encode(n: &Netlist) -> Value {
    let j = NetJson {
        variables: n.variables.clone(),
        terms: n
            .terms
            .iter()
            .map(|t| TermJson {
                coefficient: t.coefficient.to_string(),
                vars: t.vars.iter().collect(),
            })
            .collect(),
        signals: n
            .signals
            .iter()
            .map(|s| s.factors.iter().map(|f| (f.var, f.bit)).collect())
            .collect(),
        cells: n
            .cells
            .iter()
            .map(|c| CellJson {
                number: c.number,
                kind: match c.kind {
                    CellKind::Fa => "fa",
                    CellKind::Ha => "ha",
                }
                .into(),
                level: c.level,
                cpa: c.cpa,
                column: c.column,
                inputs: c.inputs.iter().map(|&w| wire_code(w)).collect(),
                has_carry: c.has_carry,
            })
            .collect(),
        result: n.result.iter().map(|w| w.map(wire_code)).collect(),
        output_width: n.output_width,
        complete: n.complete,
    };
    serde_json::to_value(j).expect("plain data always serializes")
}

/// Rebuilds a netlist and checks that every wire points to a known signal or to an earlier
/// cell, so a malformed message fails here instead of inside the simulation.
pub fn decode(json: &str) -> Result<Netlist, String> {
    let j: NetJson = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let cells = j
        .cells
        .iter()
        .map(|c| {
            let kind = match c.kind.as_str() {
                "fa" => CellKind::Fa,
                "ha" => CellKind::Ha,
                k => return Err(format!("bad cell kind {k:?}")),
            };
            Ok(NetCell {
                number: c.number,
                kind,
                level: c.level,
                cpa: c.cpa,
                column: c.column,
                inputs: c
                    .inputs
                    .iter()
                    .map(|w| parse_wire(w))
                    .collect::<Result<_, _>>()?,
                has_carry: c.has_carry,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let n = Netlist {
        variables: j.variables,
        terms: j
            .terms
            .into_iter()
            .map(|t| {
                Ok(Monomial {
                    coefficient: t.coefficient.parse().map_err(|_| "bad coefficient")?,
                    vars: t.vars.chars().collect(),
                })
            })
            .collect::<Result<_, String>>()?,
        signals: j
            .signals
            .into_iter()
            .map(|fs| Signal {
                factors: fs
                    .into_iter()
                    .map(|(var, bit)| Factor { var, bit })
                    .collect(),
            })
            .collect(),
        cells,
        result: j
            .result
            .iter()
            .map(|w| w.as_deref().map(parse_wire).transpose())
            .collect::<Result<_, _>>()?,
        output_width: j.output_width,
        complete: j.complete,
    };
    check(&n)?;
    Ok(n)
}

fn check(n: &Netlist) -> Result<(), String> {
    let vars: Vec<char> = n.variables.iter().map(|&(v, _)| v).collect();
    let width = |v: char| n.variables.iter().find(|&&(x, _)| x == v).map(|&(_, w)| w);
    let widths_ok = n
        .variables
        .iter()
        .all(|&(_, w)| (1..=expr::MAX_WIDTH).contains(&w));
    if !widths_ok || n.input_bits() > expr::MAX_INPUT_BITS {
        return Err("bad widths".into());
    }
    for s in &n.signals {
        if s.factors
            .iter()
            .any(|f| width(f.var).is_none_or(|w| f.bit >= w))
        {
            return Err("signal outside the variables".into());
        }
    }
    if n.terms
        .iter()
        .any(|t| t.vars.iter().any(|v| !vars.contains(v)))
    {
        return Err("term outside the variables".into());
    }
    let known = |w: Wire, before: usize| match w {
        Wire::Input(i) => i < n.signals.len(),
        Wire::Sum(c) | Wire::Carry(c) => c >= 1 && c < before,
    };
    for (i, c) in n.cells.iter().enumerate() {
        let arity = match c.kind {
            CellKind::Fa => 3,
            CellKind::Ha => 2,
        };
        if c.number != i + 1 || c.inputs.len() != arity {
            return Err(format!("bad cell {}", c.number));
        }
        if !c.inputs.iter().all(|&w| known(w, c.number)) {
            return Err(format!("bad input of cell {}", c.number));
        }
    }
    if !n
        .result
        .iter()
        .flatten()
        .all(|&w| known(w, n.cells.len() + 1))
    {
        return Err("bad result wire".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use dotree_core::{help, matrix::DotMatrix};

    #[test]
    fn round_trip_keeps_the_netlist() {
        let widths: BTreeMap<char, u32> = [
            ('A', 3),
            ('B', 3),
            ('C', 2),
            ('D', 2),
            ('E', 2),
            ('F', 1),
            ('G', 1),
        ]
        .into_iter()
        .collect();
        let a = expr::analyze("A*B + C*D + 3(E+F) + G", &widths).unwrap();
        let m = DotMatrix::build(&a).unwrap();
        let n = Netlist::from_reduction(&m, &help::dadda_solution(&m)).unwrap();
        assert_eq!(decode(&encode(&n).to_string()).unwrap(), n);
    }

    #[test]
    fn wires_out_of_order_are_refused() {
        let mut v = serde_json::to_value(NetJson {
            variables: vec![('A', 1)],
            terms: vec![TermJson {
                coefficient: "2".into(),
                vars: "A".into(),
            }],
            signals: vec![vec![('A', 0)]],
            cells: vec![],
            result: vec![None, Some("i0".into())],
            output_width: 2,
            complete: true,
        })
        .unwrap();
        assert!(decode(&v.to_string()).is_ok());
        v["result"][1] = "s1".into();
        assert!(decode(&v.to_string()).is_err());
        v["result"][1] = "x0".into();
        assert!(decode(&v.to_string()).is_err());
        v["result"][1] = "i1".into();
        assert!(decode(&v.to_string()).is_err());
    }
}
