// Caminho relativo: app/core/tests/counters.rs
//! MT-32a, CMP-20, VHD-11 and GER-12: one-column counters 7:3 and 15:4 (ADR 0005). Formed
//! like an FA, from 7 or 15 points of one column. Bit i of the count goes to column j + i,
//! every output ready together, and an output at or beyond W is dropped. The default cost
//! comes from the internal FA network of the VHDL: 4 FA and 3 FA delays for 7:3, 11 FA and
//! 5 FA delays for 15:4.

mod common;

use std::collections::BTreeMap;

use common::{Lcg, pick, r1, reduction};
use dotree_core::expr::analyze;
use dotree_core::help::HelpLog;
use dotree_core::matrix::DotMatrix;
use dotree_core::netlist::{Netlist, Wire};
use dotree_core::reduce::{CellKind, Costs, Origin, PointId, Reduction, Rejection, Time};
use dotree_core::state::Exercise;
use dotree_core::verify::Verifier;
use proptest::prelude::*;

const VARS: &str = "ABCDEFGHIJKLMNO";

/// `A + B + …` over `n` one-bit variables: n points in column 0.
fn ones(n: usize) -> (String, Vec<(char, u32)>) {
    let vars: Vec<char> = VARS.chars().take(n).collect();
    let src = vars
        .iter()
        .map(char::to_string)
        .collect::<Vec<_>>()
        .join("+");
    (src, vars.iter().map(|&v| (v, 1)).collect())
}

fn matrix(src: &str, widths: &[(char, u32)]) -> DotMatrix {
    let w: BTreeMap<char, u32> = widths.iter().copied().collect();
    DotMatrix::build(&analyze(src, &w).unwrap()).unwrap()
}

fn column(r: &Reduction, c: u32) -> Vec<PointId> {
    r.points()
        .iter()
        .filter(|p| p.column == c)
        .map(|p| p.id)
        .collect()
}

/// Every vector of the input space, checked by simulation (VER-01).
fn verified(m: &DotMatrix, r: &Reduction) -> u64 {
    let n = Netlist::from_reduction(m, r).expect("complete");
    let mut v = Verifier::new(n);
    let p = v.run_to_end();
    assert!(p.counterexample.is_none(), "{:?}", p.counterexample);
    p.checked
}

// ---------------------------------------------------------------- formation

#[test]
fn seven_points_of_one_column_form_a_7_3_counter() {
    let (src, w) = ones(7);
    let m = matrix(&src, &w);
    let mut r = Reduction::new(&m);
    let ids = column(&r, 0);
    assert_eq!(ids.len(), 7);
    assert_eq!(r.form_cell(&ids), Ok(CellKind::C73));
    assert_eq!(r.pending()[0].ready, Time(300));
    r.conclude_level().unwrap();

    let cell = &r.levels()[0].cells[0];
    assert_eq!(cell.kind, CellKind::C73);
    // W = 3: bits 0, 1 and 2 in columns 0, 1 and 2, all ready at 3 T_FA.
    assert_eq!(r.heights(), [1, 1, 1]);
    let names: Vec<(u32, String, Time)> = r
        .points()
        .iter()
        .map(|p| (p.column, r.signal(p), p.ready))
        .collect();
    assert_eq!(
        names,
        [
            (0, "s1".into(), Time(300)),
            (1, "k1".into(), Time(300)),
            (2, "k1_2".into(), Time(300)),
        ]
    );
    assert_eq!(cell.upper.len(), 1);
    assert_eq!(
        r.point(cell.upper[0]).unwrap().origin,
        Origin::Upper { cell: 1, bit: 2 }
    );
    assert!(r.is_complete());
    let mt = r.metrics();
    assert_eq!((mt.fa, mt.ha, mt.c73, mt.c154), (0, 0, 1, 0));
    assert_eq!((mt.area, mt.delay), (400, Time(300)));
    assert_eq!(verified(&m, &r), 128);
}

#[test]
fn fifteen_points_form_a_15_4_counter_checked_on_every_input() {
    let (src, w) = ones(15);
    let m = matrix(&src, &w);
    let mut r = Reduction::new(&m);
    assert_eq!(r.form_cell(&column(&r, 0)), Ok(CellKind::C154));
    r.conclude_level().unwrap();
    let signals: Vec<String> = r.points().iter().map(|p| r.signal(p)).collect();
    assert_eq!(signals, ["s1", "k1", "k1_2", "k1_3"]);
    let mt = r.metrics();
    assert_eq!((mt.c154, mt.area, mt.delay), (1, 1100, Time(500)));
    assert_eq!(verified(&m, &r), 1 << 15);
}

#[test]
fn other_counts_are_still_refused() {
    let (src, w) = ones(15);
    let mut r = reduction(&src, &w);
    let ids = column(&r, 0);
    for n in [0, 1, 4, 5, 6, 8, 14] {
        assert_eq!(
            r.form_cell(&ids[..n]),
            Err(Rejection::WrongCount { count: n })
        );
    }
    assert!(r.pending().is_empty());
}

#[test]
fn a_counter_takes_points_of_one_column_only() {
    // R1, column 1 has 7 points and column 2 has 5.
    let mut r = r1();
    let mut ids = column(&r, 1);
    ids[6] = column(&r, 2)[0];
    assert_eq!(r.form_cell(&ids), Err(Rejection::MixedColumns));
}

#[test]
fn outputs_at_or_beyond_w_are_dropped() {
    // 4(A + … + G): at most 28, so W = 5; the 7:3 on column 2 keeps bits 2, 3 and 4.
    let (src, w) = ones(7);
    let m = matrix(&format!("4({src})"), &w);
    let mut r = Reduction::new(&m);
    assert_eq!(m.output_width, 5);
    r.form_cell(&column(&r, 2)).unwrap();
    r.conclude_level().unwrap();
    let c = &r.levels()[0].cells[0];
    assert!(c.carry.is_some());
    assert_eq!(c.upper.len(), 1, "bit 2 lands on column 4 < W = 5");

    // W always fits the count of a column of the input, so a narrower W is set by hand
    // here, as the rule (like CMP-09's) is about any level: bit 2 would be column 4 = W.
    let mut m = matrix(&format!("4({src})"), &w);
    m.output_width = 4;
    let mut r = Reduction::new(&m);
    assert_eq!(r.heights(), [0, 0, 7, 0]);
    r.form_cell(&column(&r, 2)).unwrap();
    r.conclude_level().unwrap();
    let c = &r.levels()[0].cells[0];
    assert!(c.carry.is_some());
    assert!(c.upper.is_empty(), "bit 2 would be column 4 = W");
    assert_eq!(r.heights(), [0, 0, 1, 1]);
}

// ---------------------------------------------------------------- R1

/// Finishes a reduction with FA only, each level taking every 3 points of each column.
fn finish_with_fa(r: &mut Reduction) {
    while r.cpa_status().is_err() {
        for c in 0..r.heights().len() as u32 {
            let ids = column(r, c);
            for g in ids.chunks_exact(3) {
                r.form_cell(g).unwrap();
            }
        }
        r.conclude_level().unwrap();
    }
    r.close_with_cpa().unwrap();
}

#[test]
fn a_7_3_in_r1_column_1_and_the_circuit_still_adds() {
    let w: [(char, u32); 7] = [
        ('A', 3),
        ('B', 3),
        ('C', 2),
        ('D', 2),
        ('E', 2),
        ('F', 1),
        ('G', 1),
    ];
    let m = matrix("A*B + C*D + 3(E+F) + G", &w);
    let mut r = Reduction::new(&m);
    let col1 = ["a0_b1", "a1_b0", "c0_d1", "c1_d0", "e1", "e0", "f0"];
    let mut ids = pick(&r, 1, &col1);
    ids.reverse();
    assert_eq!(r.form_cell(&ids), Ok(CellKind::C73));
    // Inputs in canonical order (spec 02): every point is ready at 0, so generation order.
    let inputs: Vec<String> = r.pending()[0]
        .inputs
        .iter()
        .map(|&id| r.signal(r.point(id).unwrap()))
        .collect();
    assert_eq!(inputs, col1);
    r.conclude_level().unwrap();
    // Column 1 keeps s1, column 2 gets k1 and column 3 k1_2.
    assert_eq!(r.heights(), [5, 1, 6, 3, 1, 0, 0]);
    finish_with_fa(&mut r);
    assert_eq!(verified(&m, &r), 1 << 14);
    let mt = r.metrics();
    assert_eq!(mt.c73, 1);
    assert_eq!(mt.area, 400 + 100 * mt.fa as u32 + 50 * mt.ha as u32);
}

// ---------------------------------------------------------------- costs (GER-12)

#[test]
fn the_default_counter_cost_follows_the_fa() {
    let c = Costs::default();
    assert_eq!(c.area(CellKind::C73), 400);
    assert_eq!(c.delay(CellKind::C73), Time(300));
    assert_eq!(c.area(CellKind::C154), 1100);
    assert_eq!(c.delay(CellKind::C154), Time(500));
    let c = Costs {
        fa_area: 120,
        fa_delay: 40,
        ..Costs::default()
    };
    assert_eq!(c.area(CellKind::C73), 480);
    assert_eq!(c.delay(CellKind::C73), Time(120));
    assert_eq!(c.area(CellKind::C154), 1320);
    assert_eq!(c.delay(CellKind::C154), Time(200));
}

#[test]
fn a_counter_can_have_its_own_cost() {
    let c = Costs {
        c73_area: Some(250),
        c73_delay: Some(150),
        c154_delay: Some(325),
        ..Costs::default()
    };
    assert!(c.is_valid());
    assert_eq!(c.area(CellKind::C73), 250);
    assert_eq!(c.delay(CellKind::C73), Time(150));
    assert_eq!(c.area(CellKind::C154), 1100, "area still from the network");
    assert_eq!(c.delay(CellKind::C154), Time(325));
    for bad in [Some(0), Some(Costs::MAX + 1)] {
        assert!(
            !Costs {
                c154_area: bad,
                ..Costs::default()
            }
            .is_valid()
        );
    }

    let (src, w) = ones(7);
    let m = matrix(&src, &w);
    let mut r = Reduction::new(&m);
    r.form_cell(&column(&r, 0)).unwrap();
    r.conclude_level().unwrap();
    r.set_costs(c);
    let mt = r.metrics();
    assert_eq!((mt.area, mt.delay), (250, Time(150)));
    assert!(r.points().iter().all(|p| p.ready == Time(150)));
    let n = Netlist::from_reduction(&m, &r).unwrap();
    assert_eq!((n.area(), n.delay()), (250, Time(150)));
}

#[test]
fn the_critical_path_goes_through_the_counter() {
    // R1 with the 7:3: k1_2 (ready at 3) is the latest input of column 3.
    let mut r = r1();
    let ids = column(&r, 1);
    r.form_cell(&ids).unwrap();
    assert_eq!(r.critical().pending, [0]);
    r.conclude_level().unwrap();
    let k = pick(&r, 3, &["k1_2"]);
    let others: Vec<PointId> = column(&r, 3).into_iter().filter(|&i| i != k[0]).collect();
    let mut sel = k.clone();
    sel.extend(&others[..2]);
    r.form_cell(&sel).unwrap();
    r.conclude_level().unwrap();
    assert_eq!(r.metrics().delay, Time(400));
    assert!(r.critical().cells.contains(&1));
    let m = matrix(
        "A*B + C*D + 3(E+F) + G",
        &[
            ('A', 3),
            ('B', 3),
            ('C', 2),
            ('D', 2),
            ('E', 2),
            ('F', 1),
            ('G', 1),
        ],
    );
    let n = Netlist::partial(&m, &r);
    assert!(n.cells[0].inputs.len() == 7 && n.cells[0].kind == CellKind::C73);
    assert_eq!(n.delay(), Time(400));
    assert!(
        n.cells
            .iter()
            .any(|c| c.inputs.contains(&Wire::Upper(1, 2)))
    );
}

// ---------------------------------------------------------------- file (SAL)

#[test]
fn counters_go_to_the_file_and_come_back() {
    let mut r = r1();
    let ids = column(&r, 1);
    r.form_cell(&ids).unwrap();
    r.conclude_level().unwrap();
    let ids = column(&r, 2);
    r.form_cell(&ids[..3]).unwrap();
    r.set_costs(Costs {
        c73_delay: Some(200),
        ..Costs::default()
    });
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
    let ex = Exercise::capture(
        "A*B + C*D + 3(E+F) + G",
        &widths,
        &r,
        None,
        HelpLog::default(),
    );
    let json = ex.to_json();
    assert!(json.contains("\"kind\": \"c73\""), "{json}");
    assert!(json.contains("\"c73Delay\": 200"), "{json}");
    assert!(!json.contains("c73Area"), "unset costs stay out: {json}");
    let back = Exercise::parse(&json).unwrap();
    assert_eq!(back, ex);
    let (_, again) = back.restore().unwrap();
    assert_eq!(again.levels(), r.levels());
    assert_eq!(again.pending(), r.pending());
    assert_eq!(again.costs(), r.costs());

    // A kind that does not match the number of inputs is refused.
    let wrong = json.replacen("\"c73\"", "\"c154\"", 1);
    assert!(Exercise::parse(&wrong).unwrap().restore().is_err());
}

// ---------------------------------------------------------------- random

/// Reduces at random with FA, HA and, where a column has room, 7:3 and 15:4.
fn reduce_with_counters(r: &mut Reduction, g: &mut Lcg) {
    while !r.is_complete() {
        if r.cpa_status().is_ok() && g.next(2) == 0 {
            r.close_with_cpa().unwrap();
            continue;
        }
        for c in 0..r.heights().len() as u32 {
            let mut free = column(r, c);
            while free.len() >= 2 && g.next(4) != 0 {
                let sizes: Vec<usize> = [15, 7, 3, 2]
                    .into_iter()
                    .filter(|&s| s <= free.len())
                    .collect();
                let n = sizes[g.next(sizes.len())];
                r.form_cell(&free[..n]).unwrap();
                free.drain(..n);
            }
        }
        if r.pending().is_empty() {
            // Nothing drawn this time: one FA or HA on the tallest column.
            let c = (0..r.heights().len())
                .max_by_key(|&c| r.heights()[c])
                .unwrap() as u32;
            let free = column(r, c);
            r.form_cell(&free[..free.len().min(3)]).unwrap();
        }
        r.conclude_level().unwrap();
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 48, failure_persistence: None, ..ProptestConfig::default() })]

    /// VER-01 with counters: any reduction that uses them still computes the expression,
    /// and the netlist agrees with the reduction on area and delay.
    #[test]
    fn random_reductions_with_counters_add_correctly(seed in any::<u64>(), pick in 0usize..3) {
        let (src, w): (&str, &[(char, u32)]) = [
            ("A*B + C*D + 3(E+F) + G", &[('A', 3), ('B', 3), ('C', 2), ('D', 2), ('E', 2), ('F', 1), ('G', 1)][..]),
            ("A*B", &[('A', 4), ('B', 4)][..]),
            ("A+B+C+D+E+F+G+H+I", &[('A', 2), ('B', 2), ('C', 2), ('D', 2), ('E', 1), ('F', 1), ('G', 1), ('H', 1), ('I', 1)][..]),
        ][pick];
        let m = matrix(src, w);
        let mut r = Reduction::new(&m);
        reduce_with_counters(&mut r, &mut Lcg(seed));
        verified(&m, &r);
        let n = Netlist::from_reduction(&m, &r).unwrap();
        prop_assert_eq!(n.area(), r.metrics().area);
        prop_assert_eq!(n.delay(), r.metrics().delay);
    }
}

// ---------------------------------------------------------------- VHDL (VHD-11)

use dotree_core::diagram::Layout;
use dotree_core::vhdl::{DEFAULT_NAME, Language, VhdlError, VhdlFile, generate};

fn vhdl(m: &DotMatrix, r: &Reduction, lang: Language) -> Vec<VhdlFile> {
    generate(
        m,
        &Netlist::from_reduction(m, r).unwrap(),
        DEFAULT_NAME,
        lang,
    )
    .unwrap()
}

fn text<'a>(fs: &'a [VhdlFile], name: &str) -> &'a str {
    &fs.iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("no {name}"))
        .contents
}

fn names(fs: &[VhdlFile]) -> Vec<&str> {
    fs.iter()
        .map(|f| f.name.as_str())
        .filter(|n| n.ends_with(".vhd"))
        .collect()
}

/// One counter over `n` one-bit variables.
fn one_counter(n: usize) -> (DotMatrix, Reduction) {
    let (src, w) = ones(n);
    let m = matrix(&src, &w);
    let mut r = Reduction::new(&m);
    r.form_cell(&column(&r, 0)).unwrap();
    r.conclude_level().unwrap();
    (m, r)
}

#[test]
fn vhd11_a_7_3_gets_its_entity_built_of_full_adders() {
    let (m, r) = one_counter(7);
    let fs = vhdl(&m, &r, Language::PtBr);
    assert_eq!(
        names(&fs),
        [
            "full_adder.vhd",
            "counter_7_3.vhd",
            "dotree_circuito.vhd",
            "dotree_circuito_comportamental.vhd",
            "tb_dotree_circuito.vhd",
            "dotree_circuito_de2.vhd",
            "tb_dotree_circuito_de2.vhd",
        ]
    );
    let c = text(&fs, "counter_7_3.vhd");
    assert!(c.contains("entity counter_7_3 is"));
    assert_eq!(c.matches(": full_adder port map").count(), 4);
    assert!(c.contains("\nend counter_7_3;\n"));
    let top = text(&fs, "dotree_circuito.vhd");
    assert!(top.contains(
        "    C73_1 : counter_7_3 port map (x0 => a0, x1 => b0, x2 => c0, x3 => d0, \
         x4 => e0, x5 => f0, x6 => g0, y0 => s1, y1 => k1, y2 => k1_2);  -- 2^0\n"
    ));
    assert!(top.contains("    signal k1_2 : std_logic;\n"));
    assert!(top.contains("-- Metricas: 0 FA, 0 HA, 1 7:3, 0 AND; area 4 A_FA; atraso 3 T_FA\n"));
    assert!(!top.contains("component half_adder"));
}

#[test]
fn vhd11_a_15_4_brings_the_7_3_it_is_made_of() {
    let (m, r) = one_counter(15);
    let fs = vhdl(&m, &r, Language::En);
    let n = names(&fs);
    assert_eq!(
        n[..3],
        ["full_adder.vhd", "counter_7_3.vhd", "counter_15_4.vhd"]
    );
    let c = text(&fs, "counter_15_4.vhd");
    assert_eq!(c.matches(": counter_7_3 port map").count(), 2);
    assert_eq!(c.matches(": full_adder port map").count(), 3);
    let top = text(&fs, "dotree_circuito.vhd");
    // The top declares only what it instantiates.
    assert!(top.contains("component counter_15_4 is"));
    assert!(!top.contains("component counter_7_3 is"));
    assert!(top.contains("y3 => k1_3);  -- 2^0\n"));
    for f in &fs {
        assert!(f.contents.is_ascii(), "{}", f.name);
    }
}

#[test]
fn vhd11_dropped_outputs_are_left_open_and_names_are_reserved() {
    // The narrow W of `outputs_at_or_beyond_w_are_dropped`: bit 2 is gone.
    let (src, w) = ones(7);
    let mut m = matrix(&format!("4({src})"), &w);
    m.output_width = 4;
    let mut r = Reduction::new(&m);
    r.form_cell(&column(&r, 2)).unwrap();
    r.conclude_level().unwrap();
    assert!(r.is_complete());
    let fs = vhdl(&m, &r, Language::PtBr);
    let top = text(&fs, "dotree_circuito.vhd");
    assert!(top.contains("y0 => s1, y1 => k1, y2 => open);  -- 2^2\n"));
    assert!(!top.contains("k1_2"));

    let (m, r) = one_counter(7);
    let n = Netlist::from_reduction(&m, &r).unwrap();
    for name in ["C73_1", "k1_2", "counter_7_3", "Counter_15_4"] {
        assert_eq!(
            generate(&m, &n, name, Language::PtBr),
            Err(VhdlError::InvalidName),
            "{name}"
        );
    }
    assert!(generate(&m, &n, "c73_2", Language::PtBr).is_ok());
}

#[test]
fn vhd11_the_top_declares_the_diagram_identifiers_with_counters() {
    // DIA-03 and VHD-02 with the 7:3 of R1 and a 15:4 elsewhere.
    let mut circuits = vec![];
    let mut r = r1();
    r.form_cell(&column(&r, 1)).unwrap();
    r.conclude_level().unwrap();
    finish_with_fa(&mut r);
    circuits.push((
        matrix(
            "A*B + C*D + 3(E+F) + G",
            &[
                ('A', 3),
                ('B', 3),
                ('C', 2),
                ('D', 2),
                ('E', 2),
                ('F', 1),
                ('G', 1),
            ],
        ),
        r,
    ));
    circuits.push(one_counter(15));
    for (m, r) in circuits {
        let n = Netlist::from_reduction(&m, &r).unwrap();
        let fs = vhdl(&m, &r, Language::PtBr);
        let mut declared = std::collections::BTreeSet::new();
        for line in text(&fs, "dotree_circuito.vhd").lines().map(str::trim) {
            if let Some(rest) = line.strip_prefix("signal ") {
                declared.insert(rest.split(':').next().unwrap().trim().to_string());
            }
            if line.starts_with("R(") && line.contains("<=") {
                declared.insert(line.split("<=").next().unwrap().trim().to_string());
            }
        }
        assert_eq!(declared, Layout::new(&n).identifiers());
    }
}

#[test]
fn vhd03_the_costs_line_names_the_counters_in_use() {
    let (m, mut r) = one_counter(7);
    r.set_costs(Costs {
        c73_delay: Some(150),
        ..Costs::default()
    });
    let top = text(&vhdl(&m, &r, Language::PtBr), "dotree_circuito.vhd").to_string();
    assert!(
        top.contains("-- Custos: FA 1 A_FA, 1 T_FA; HA 0,5 A_FA, 0,5 T_FA; 7:3 4 A_FA, 1,5 T_FA\n"),
        "{top}"
    );
}

#[test]
fn cmp18_the_height_table_counts_the_counters() {
    use dotree_core::heights::height_table;
    let mut r = r1();
    r.form_cell(&column(&r, 1)).unwrap();
    r.form_cell(&column(&r, 0)[..3]).unwrap();
    let rows = height_table(&r);
    let now = rows.last().unwrap();
    assert_eq!((now.fa, now.ha, now.c73, now.c154), (1, 0, 1, 0));
    r.conclude_level().unwrap();
    let rows = height_table(&r);
    assert_eq!((rows[0].fa, rows[0].c73), (1, 1));
}

// ---------------------------------------------------------------- tables (MT-32b)

use dotree_core::diagram::{TableHeaders, wiring_table, wiring_table_markdown};
use dotree_core::heights::{HeightHeaders, height_table_markdown};

const HEADERS: TableHeaders = TableHeaders {
    cell: "Célula",
    level: "Nível",
    column: "Coluna",
    sum: "Soma",
    carry: "Vai-um",
    inputs: "Entradas",
    upper: "Bits 2 e 3",
};

#[test]
fn dia04_a_counter_row_lists_every_input_and_its_bits_2_and_3() {
    let (m, r) = one_counter(15);
    let n = Netlist::from_reduction(&m, &r).unwrap();
    let rows = wiring_table(&n);
    assert_eq!(rows[0].cell, "C154_1");
    assert_eq!(rows[0].inputs.len(), 15);
    assert_eq!(rows[0].inputs[14].as_deref(), Some("o0"));
    let upper: Vec<(&str, Option<u32>)> = rows[0]
        .upper
        .iter()
        .map(|o| (o.name.as_str(), o.result))
        .collect();
    assert_eq!(upper, [("k1_2", Some(2)), ("k1_3", Some(3))]);
    assert_eq!(
        wiring_table_markdown(&rows, HEADERS),
        "| Célula | Nível | Coluna | Entradas | Soma | Vai-um | Bits 2 e 3 |\n\
         |---|---|---|---|---|---|---|\n\
         | C154_1 | 0 | $2^0$ | `a0`, `b0`, `c0`, `d0`, `e0`, `f0`, `g0`, `h0`, `i0`, `j0`, `k0_in`, \
         `l0`, `m0`, `n0`, `o0` | `s1` → `R(0)` | `k1` → `R(1)` | `k1_2` → `R(2)`, `k1_3` → `R(3)` |\n"
    );
}

#[test]
fn dia04_with_a_counter_the_fa_rows_take_the_same_columns() {
    let mut r = r1();
    r.form_cell(&column(&r, 1)).unwrap();
    r.conclude_level().unwrap();
    finish_with_fa(&mut r);
    let m = matrix(
        "A*B + C*D + 3(E+F) + G",
        &[
            ('A', 3),
            ('B', 3),
            ('C', 2),
            ('D', 2),
            ('E', 2),
            ('F', 1),
            ('G', 1),
        ],
    );
    let rows = wiring_table(&Netlist::from_reduction(&m, &r).unwrap());
    // An FA keeps x, y, z, and an HA a missing z.
    let fa = rows.iter().find(|r| r.cell.starts_with("FA")).unwrap();
    assert_eq!(fa.inputs.len(), 3);
    assert!(fa.upper.is_empty());
    let md = wiring_table_markdown(&rows, HEADERS);
    let fa_line = md
        .lines()
        .find(|l| l.starts_with(&format!("| {} |", fa.cell)))
        .unwrap();
    assert_eq!(fa_line.matches(" | ").count(), 6, "{fa_line}");
    assert!(fa_line.ends_with(" | — |"), "{fa_line}");
}

#[test]
fn cmp18_the_height_table_shows_counter_columns_only_when_used() {
    let pt = HeightHeaders {
        level: "Nível",
        current: "atual",
        result: "Resultado",
    };
    let (_, r) = one_counter(7);
    let md = height_table_markdown(&dotree_core::heights::height_table(&r), pt);
    assert_eq!(
        md,
        "| Nível | $2^2$ | $2^1$ | $2^0$ | FA | HA | 7:3 | 15:4 |\n\
         |---|---|---|---|---|---|---|---|\n\
         | 0 | 0 | 0 | 7 | 0 | 0 | 1 | 0 |\n\
         | Resultado | 1 | 1 | 1 | — | — | — | — |\n"
    );
    let plain = height_table_markdown(&dotree_core::heights::height_table(&r1()), pt);
    assert!(
        plain.starts_with(
            "| Nível | $2^6$ | $2^5$ | $2^4$ | $2^3$ | $2^2$ | $2^1$ | $2^0$ | FA | HA |\n"
        ),
        "{plain}"
    );
}
