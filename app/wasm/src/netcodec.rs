// Caminho relativo: app/wasm/src/netcodec.rs
//! JSON form of a [`Netlist`], so the finished circuit can travel to the verification
//! worker (VER-04), which runs its own copy of this module. Wires are `"i<signal>"`,
//! `"s<n>"`, `"k<n>"` and `"u<n>_<bit>"` (a counter's bits 2 and 3); coefficients travel as strings, since they may not fit a JS number.

use dotree_core::expr::{self, Monomial};
use dotree_core::matrix::{Factor, Signal};
use dotree_core::netlist::{NetCell, Netlist, Wire};
use dotree_core::reduce::{CellKind, Costs};
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
    #[serde(default)]
    upper: u8,
}

fn kind_code(k: CellKind) -> &'static str {
    match k {
        CellKind::Fa => "fa",
        CellKind::Ha => "ha",
        CellKind::C73 => "c73",
        CellKind::C154 => "c154",
    }
}

fn wire_code(w: Wire) -> String {
    match w {
        Wire::Input(i) => format!("i{i}"),
        Wire::Sum(n) => format!("s{n}"),
        Wire::Carry(n) => format!("k{n}"),
        Wire::Upper(n, bit) => format!("u{n}_{bit}"),
    }
}

fn parse_wire(code: &str) -> Result<Wire, String> {
    let bad = || format!("bad wire {code:?}");
    let (tag, rest) = code.split_at_checked(1).ok_or_else(bad)?;
    if tag == "u" {
        let (n, bit) = rest.split_once('_').ok_or_else(bad)?;
        let n: usize = n.parse().map_err(|_| bad())?;
        let bit: u8 = bit.parse().map_err(|_| bad())?;
        return Ok(Wire::Upper(n, bit));
    }
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
                kind: kind_code(c.kind).into(),
                level: c.level,
                cpa: c.cpa,
                column: c.column,
                inputs: c.inputs.iter().map(|&w| wire_code(w)).collect(),
                has_carry: c.has_carry,
                upper: c.upper,
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
            let kind = [CellKind::Fa, CellKind::Ha, CellKind::C73, CellKind::C154]
                .into_iter()
                .find(|&k| kind_code(k) == c.kind)
                .ok_or_else(|| format!("bad cell kind {:?}", c.kind))?;
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
                upper: c.upper,
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
        // The verifier only evaluates the cells: their area and delay play no part.
        costs: Costs::default(),
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
        // Bit 2 or 3 of a counter that keeps it.
        Wire::Upper(c, bit) => c >= 1 && c < before && bit >= 2 && bit - 2 < n.cells[c - 1].upper,
    };
    for (i, c) in n.cells.iter().enumerate() {
        let room = c.kind.outputs() - 2;
        if c.number != i + 1
            || c.inputs.len() != c.kind.inputs()
            || u32::from(c.upper) > room
            || (c.upper > 0 && !c.has_carry)
        {
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

    /// A 7:3 over seven one-bit variables (ADR 0005).
    fn counter_netlist() -> Netlist {
        let widths: BTreeMap<char, u32> = ('A'..='G').map(|v| (v, 1)).collect();
        let a = expr::analyze("A+B+C+D+E+F+G", &widths).unwrap();
        let m = DotMatrix::build(&a).unwrap();
        let mut r = dotree_core::reduce::Reduction::new(&m);
        let ids: Vec<usize> = r.points().iter().map(|p| p.id).collect();
        r.form_cell(&ids).unwrap();
        r.conclude_level().unwrap();
        Netlist::from_reduction(&m, &r).unwrap()
    }

    #[test]
    fn counters_travel_too() {
        let n = counter_netlist();
        let v = encode(&n);
        assert_eq!(v["cells"][0]["kind"], "c73");
        assert_eq!(v["result"][2], "u1_2");
        assert_eq!(decode(&v.to_string()).unwrap(), n);
    }

    #[test]
    fn counter_wires_that_do_not_exist_are_refused() {
        let v = encode(&counter_netlist());
        for (path, bad) in [
            ("result2", "u1_3"), // a 7:3 has no bit 3
            ("result2", "u2_2"), // no cell 2
            ("result2", "u1_x"),
            ("upper", "2"),   // a 7:3 keeps at most one upper bit
            ("kind", "c154"), // 7 inputs are not a 15:4
        ] {
            let mut w = v.clone();
            match path {
                "result2" => w["result"][2] = bad.into(),
                "upper" => w["cells"][0]["upper"] = 2.into(),
                _ => w["cells"][0]["kind"] = bad.into(),
            }
            assert!(decode(&w.to_string()).is_err(), "{path} {bad}");
        }
    }
}
