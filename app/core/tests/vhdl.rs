// Caminho relativo: app/core/tests/vhdl.rs
//! Spec 06 — VHDL generation (MT-10). Structure only: the proof that the files simulate is
//! the GHDL job in CI (`examples/vhdl_referencia.rs`).

mod common;

use std::collections::{BTreeMap, BTreeSet};

use common::{r1, replay_r2_tree};
use dotree_core::diagram::Layout;
use dotree_core::expr::analyze;
use dotree_core::matrix::DotMatrix;
use dotree_core::netlist::Netlist;
use dotree_core::reduce::Reduction;
use dotree_core::verify::sample_vectors;
use dotree_core::vhdl::{DEFAULT_NAME, Language, VhdlError, VhdlFile, generate, port_name};

fn build(src: &str, widths: &[(char, u32)]) -> (DotMatrix, Reduction) {
    let w: BTreeMap<char, u32> = widths.iter().copied().collect();
    let m = DotMatrix::build(&analyze(src, &w).unwrap()).unwrap();
    let r = Reduction::new(&m);
    (m, r)
}

fn r1_matrix() -> DotMatrix {
    build(
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
    )
    .0
}

fn r2() -> (DotMatrix, Netlist) {
    let m = r1_matrix();
    let mut r = r1();
    replay_r2_tree(&mut r);
    r.close_with_cpa().unwrap();
    let n = Netlist::from_reduction(&m, &r).unwrap();
    (m, n)
}

fn files(m: &DotMatrix, n: &Netlist, lang: Language) -> Vec<VhdlFile> {
    generate(m, n, DEFAULT_NAME, lang).unwrap()
}

fn file<'a>(fs: &'a [VhdlFile], name: &str) -> &'a str {
    &fs.iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("no {name}"))
        .contents
}

fn top(fs: &[VhdlFile]) -> &str {
    file(fs, "dotree_circuito.vhd")
}

/// Removes `--` comments and string literals, to compare code across languages.
fn code_only(text: &str) -> String {
    text.lines()
        .map(|l| {
            let mut out = String::new();
            let mut in_str = false;
            let chars: Vec<char> = l.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                let c = chars[i];
                if !in_str && c == '-' && chars.get(i + 1) == Some(&'-') {
                    break;
                }
                if c == '"' {
                    in_str = !in_str;
                    out.push('"');
                } else if !in_str {
                    out.push(c);
                }
                i += 1;
            }
            out.trim_end().to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------- VHD-01

#[test]
fn vhd01_r2_files() {
    let (m, n) = r2();
    let names: Vec<String> = files(&m, &n, Language::PtBr)
        .into_iter()
        .map(|f| f.name)
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
}

#[test]
fn vhd01_no_half_adder_file_without_has() {
    // A + B + C, 1 bit each: the CPA closes column 0 with a single FA.
    let (m, mut r) = build("A + B + C", &[('A', 1), ('B', 1), ('C', 1)]);
    r.close_with_cpa().unwrap();
    let n = Netlist::from_reduction(&m, &r).unwrap();
    let names: Vec<String> = files(&m, &n, Language::PtBr)
        .into_iter()
        .map(|f| f.name)
        .collect();
    assert_eq!(
        names,
        [
            "full_adder.vhd",
            "dotree_circuito.vhd",
            "tb_dotree_circuito.vhd"
        ]
    );
}

#[test]
fn vhd01_only_vhdl93_libraries() {
    let (m, n) = r2();
    for f in files(&m, &n, Language::PtBr) {
        for line in f
            .contents
            .lines()
            .filter(|l| l.trim_start().starts_with("use "))
        {
            assert!(
                line.contains("ieee.std_logic_1164.all")
                    || line.contains("ieee.numeric_std.all")
                    || line.trim() == "use work.all;",
                "{}: {line}",
                f.name
            );
        }
    }
}

#[test]
fn vhd01_top_makes_the_cell_entities_visible() {
    // Without it GHDL leaves every FA/HA instance unbound (found by running the files).
    let (m, n) = r2();
    assert!(top(&files(&m, &n, Language::PtBr)).contains("\nuse work.all;\n"));
}

#[test]
fn vhd01_files_are_plain_ascii() {
    // GHDL refuses non-Latin-1 bytes even in comments (found by running the files).
    let (m, n) = r2();
    for lang in [Language::PtBr, Language::En] {
        for f in files(&m, &n, lang) {
            assert!(f.contents.is_ascii(), "{} ({lang:?}) is not ASCII", f.name);
        }
    }
}

// ---------------------------------------------------------------- VHD-07

#[test]
fn vhd07_incomplete_circuit() {
    let mut r = r1();
    replay_r2_tree(&mut r);
    let m = r1_matrix();
    let n = Netlist::partial(&m, &r);
    assert_eq!(
        generate(&m, &n, DEFAULT_NAME, Language::PtBr),
        Err(VhdlError::NotComplete)
    );
}

// ---------------------------------------------------------------- VHD-02

#[test]
fn vhd02_top_declares_exactly_the_diagram_identifiers() {
    let (m, n) = r2();
    let fs = files(&m, &n, Language::PtBr);
    let t = top(&fs);
    let mut declared = BTreeSet::new();
    for line in t.lines().map(str::trim) {
        if let Some(rest) = line.strip_prefix("signal ") {
            declared.insert(rest.split(':').next().unwrap().trim().to_string());
        }
        if line.starts_with("R(") && line.contains("<=") {
            declared.insert(line.split("<=").next().unwrap().trim().to_string());
        }
    }
    assert_eq!(declared, Layout::new(&n).identifiers());
}

#[test]
fn vhd02_one_instance_per_cell_with_table_names() {
    let (m, n) = r2();
    let fs = files(&m, &n, Language::PtBr);
    let labels: Vec<String> = top(&fs)
        .lines()
        .map(str::trim)
        .filter(|l| l.contains(" port map"))
        .map(|l| l.split(':').next().unwrap().trim().to_string())
        .collect();
    let expected: Vec<String> = (1..=17)
        .map(|i| {
            if [3, 11, 14, 17].contains(&i) {
                format!("HA{i}")
            } else {
                format!("FA{i}")
            }
        })
        .collect();
    assert_eq!(labels, expected);
    let fa16 = top(&fs)
        .lines()
        .find(|l| l.trim_start().starts_with("FA16 "))
        .unwrap();
    assert!(fa16.contains("x => s11") && fa16.contains("y => k10") && fa16.contains("z => k15"));
    assert!(fa16.contains("s => s16") && fa16.contains("k => k16"));
}

#[test]
fn vhd02_constant_zero_result_bit() {
    let (m, r) = build("2A", &[('A', 2)]);
    let n = Netlist::from_reduction(&m, &r).unwrap();
    let fs = files(&m, &n, Language::PtBr);
    assert!(top(&fs).contains("R(0) <= '0';"));
    assert!(top(&fs).contains("R(1) <= a0;"));
}

#[test]
fn vhd02_variable_named_r_gets_another_port_name() {
    assert_eq!(port_name('R'), "R_IN");
    assert_eq!(port_name('A'), "A");
    let (m, mut r) = build("R + S", &[('R', 1), ('S', 1)]);
    r.close_with_cpa().unwrap();
    let n = Netlist::from_reduction(&m, &r).unwrap();
    let fs = files(&m, &n, Language::PtBr);
    assert!(top(&fs).contains("R_IN : in  std_logic_vector(0 downto 0)"));
    assert!(top(&fs).contains("r0 <= R_IN(0);"));
}

// ---------------------------------------------------------------- VHD-03

#[test]
fn vhd03_header_documents_the_circuit() {
    let (m, n) = r2();
    let fs = files(&m, &n, Language::PtBr);
    let t = top(&fs);
    for needle in [
        "R = AB + CD + 3E + 3F + G",
        "A = 3, B = 3, C = 2, D = 2, E = 2, F = 1, G = 1",
        "--   2^1 : a0_b1 a1_b0 c0_d1 c1_d0 e1 e0 f0",
        "--   2^0 : a0_b0 c0_d0 e0 f0 g0",
        "13 FA, 4 HA, 13 AND",
        "15 A_FA",
        "6 T_FA",
    ] {
        assert!(t.contains(needle), "missing {needle:?}");
    }
}

// ---------------------------------------------------------------- VHD-06

#[test]
fn vhd06_languages_differ_only_in_comments_and_messages() {
    let (m, n) = r2();
    let pt = files(&m, &n, Language::PtBr);
    let en = files(&m, &n, Language::En);
    assert_eq!(pt.len(), en.len());
    for (a, b) in pt.iter().zip(&en) {
        assert_eq!(a.name, b.name);
        assert_eq!(code_only(&a.contents), code_only(&b.contents), "{}", a.name);
    }
    assert!(top(&pt).contains("Expressao"));
    assert!(top(&en).contains("Expression"));
    assert!(file(&pt, "tb_dotree_circuito.vhd").contains("CASOS CORRETOS"));
    assert!(file(&en, "tb_dotree_circuito.vhd").contains("CASES CORRECT"));
}

// ---------------------------------------------------------------- VHD-04

#[test]
fn vhd04_exhaustive_testbench_up_to_20_bits() {
    let (m, n) = r2();
    let fs = files(&m, &n, Language::PtBr);
    let tb = file(&fs, "tb_dotree_circuito.vhd");
    assert!(tb.contains("for n in 0 to 16383 loop"));
    assert!(tb.contains("A <= v(2 downto 0);"));
    assert!(tb.contains("G <= v(13 downto 13);"));
}

#[test]
fn vhd04_sampled_testbench_above_20_bits() {
    // A + 2048B with 11 + 10 bits: complete from the start, 21 input bits.
    let (m, r) = build("A + 2048B", &[('A', 11), ('B', 10)]);
    let n = Netlist::from_reduction(&m, &r).unwrap();
    let fs = files(&m, &n, Language::En);
    let tb = file(&fs, "tb_dotree_circuito.vhd");
    assert!(tb.contains("array (0 to 4095)"));
    for v in sample_vectors(21, 3) {
        assert!(tb.contains(&format!("\"{v:021b}\"")), "{v:021b}");
    }
    assert!(!tb.contains("for n in 0 to 2097151"));
}

// ---------------------------------------------------------------- entity name

#[test]
fn entity_name_validation() {
    let (m, n) = r2();
    for good in ["dotree_circuito", "Meu_Circuito2", "x"] {
        assert!(generate(&m, &n, good, Language::PtBr).is_ok(), "{good}");
    }
    for bad in [
        "",
        "1x",
        "a__b",
        "x_",
        "_x",
        "entity",
        "SIGNAL",
        "e0",
        "A0_B1",
        "s3",
        "FA1",
        "full_adder",
        "R",
        "A",
        "ç",
    ] {
        assert_eq!(
            generate(&m, &n, bad, Language::PtBr),
            Err(VhdlError::InvalidName),
            "{bad}"
        );
    }
    let fs = generate(&m, &n, "meu_somador", Language::PtBr).unwrap();
    assert!(fs.iter().any(|f| f.name == "meu_somador.vhd"));
    assert!(fs.iter().any(|f| f.name == "tb_meu_somador.vhd"));
}
