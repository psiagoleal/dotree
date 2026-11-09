// Caminho relativo: app/core/tests/board.rs
//! Spec 10 — DE2 board top and Quartus project (MT-22). The proof that the files simulate
//! is the GHDL job in CI; the proof that the project compiles is the Quartus run recorded
//! in the spec.

mod common;

use std::collections::{BTreeMap, BTreeSet};

use common::{r1, replay_r2_tree};
use dotree_core::board::{BoardLimit, DE2_PINS, SEGMENTS, fits};
use dotree_core::expr::analyze;
use dotree_core::matrix::DotMatrix;
use dotree_core::netlist::Netlist;
use dotree_core::reduce::Reduction;
use dotree_core::vhdl::{DEFAULT_NAME, Language, VhdlError, VhdlFile, generate, port_name};

fn r2() -> (DotMatrix, Netlist) {
    let w: BTreeMap<char, u32> = [
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
    let m = DotMatrix::build(&analyze("A*B + C*D + 3(E+F) + G", &w).unwrap()).unwrap();
    let mut r = r1();
    replay_r2_tree(&mut r);
    r.close_with_cpa().unwrap();
    let n = Netlist::from_reduction(&m, &r).unwrap();
    (m, n)
}

/// A circuit that is complete from the start: one dot per column.
fn single_row(src: &str, widths: &[(char, u32)]) -> (DotMatrix, Netlist) {
    let w: BTreeMap<char, u32> = widths.iter().copied().collect();
    let m = DotMatrix::build(&analyze(src, &w).unwrap()).unwrap();
    let n = Netlist::from_reduction(&m, &Reduction::new(&m)).unwrap();
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

fn de2(fs: &[VhdlFile]) -> &str {
    file(fs, "dotree_circuito_de2.vhd")
}

const BOARD_FILES: [&str; 4] = [
    "dotree_circuito_de2.vhd",
    "tb_dotree_circuito_de2.vhd",
    "dotree_circuito.qpf",
    "dotree_circuito.qsf",
];

// ---------------------------------------------------------------- PLA-01

#[test]
fn pla01_r2_gets_the_board_files_after_the_others() {
    let (m, n) = r2();
    assert_eq!(fits(&n), Ok(()));
    let names: Vec<String> = files(&m, &n, Language::PtBr)
        .into_iter()
        .map(|f| f.name)
        .collect();
    assert_eq!(names[names.len() - 4..], BOARD_FILES);
}

#[test]
fn pla01_more_than_18_input_bits_do_not_fit() {
    let (m, n) = single_row("A + 1024B", &[('A', 10), ('B', 9)]);
    assert_eq!(fits(&n), Err(BoardLimit::TooManyInputs(19)));
    let fs = files(&m, &n, Language::PtBr);
    assert!(fs.iter().all(|f| !BOARD_FILES.contains(&f.name.as_str())));
    assert!(fs.iter().any(|f| f.name == "dotree_circuito.vhd"));
}

#[test]
fn pla01_more_than_32_output_bits_do_not_fit() {
    // 2^31 A with A of 2 bits: dots at columns 31 and 32.
    let (m, n) = single_row("2147483648A", &[('A', 2)]);
    assert_eq!(fits(&n), Err(BoardLimit::TooWide(33)));
    let fs = files(&m, &n, Language::PtBr);
    assert!(fs.iter().all(|f| !BOARD_FILES.contains(&f.name.as_str())));
}

// ---------------------------------------------------------------- PLA-02

#[test]
fn pla02_board_ports() {
    let (m, n) = r2();
    let fs = files(&m, &n, Language::PtBr);
    let t = de2(&fs);
    assert!(t.contains("entity dotree_circuito_de2 is"));
    assert!(t.contains("        SW   : in  std_logic_vector(17 downto 0);"));
    assert!(t.contains("        LEDR : out std_logic_vector(17 downto 0);"));
    for h in 0..7 {
        assert!(t.contains(&format!(
            "        HEX{h} : out std_logic_vector(6 downto 0);"
        )));
    }
    assert!(t.contains("        HEX7 : out std_logic_vector(6 downto 0)\n    );"));
}

#[test]
fn pla02_variables_take_the_switches_in_testbench_order() {
    for (m, n) in [
        r2(),
        single_row("R + 16S", &[('R', 4), ('S', 5)]),
        single_row("A + 512B", &[('A', 9), ('B', 9)]),
    ] {
        let fs = files(&m, &n, Language::PtBr);
        let t = de2(&fs);
        let offsets = n.offsets();
        for &(v, w) in &n.variables {
            let lo = offsets[&v];
            let switches = format!("SW({} downto {lo})", lo + w - 1);
            assert!(
                t.contains(&format!("{} => {switches}", port_name(v))),
                "{v}: {t}"
            );
            assert!(t.contains(&format!("{v} = {switches}")), "header for {v}");
        }
    }
}

// ---------------------------------------------------------------- PLA-03

#[test]
fn pla03_leds_show_the_result() {
    let (m, n) = r2();
    let t = files(&m, &n, Language::PtBr);
    assert!(de2(&t).contains("    LEDR(6 downto 0) <= R;"));
    assert!(de2(&t).contains("    LEDR(17 downto 7) <= (others => '0');"));

    let (m, n) = single_row("A + 512B", &[('A', 9), ('B', 9)]);
    let t = files(&m, &n, Language::PtBr);
    assert!(de2(&t).contains("    LEDR <= R;"));
    assert!(!de2(&t).contains("others => '0'"));

    // W = 21: only the 18 low bits fit the LEDs.
    let (m, n) = single_row("1048576A", &[('A', 1)]);
    let t = files(&m, &n, Language::PtBr);
    assert!(de2(&t).contains("    LEDR <= R(17 downto 0);"));
}

// ---------------------------------------------------------------- PLA-04

/// Lit segments of each hexadecimal digit, as drawn on a seven-segment display (a on top,
/// then clockwise b to f, g in the middle).
const LIT: [&str; 16] = [
    "abcdef", "bc", "abdeg", "abcdg", "bcfg", "acdfg", "acdefg", "abc", "abcdefg", "abcdfg",
    "abcefg", "cdefg", "adef", "bcdeg", "adefg", "aefg",
];

#[test]
fn pla04_segment_table_is_active_low_a_to_g() {
    for (d, lit) in LIT.iter().enumerate() {
        let mut expected = 0u8;
        for (i, seg) in "abcdefg".chars().enumerate() {
            if !lit.contains(seg) {
                expected |= 1 << i; // off is '1'
            }
        }
        assert_eq!(SEGMENTS[d], expected, "digit {d:x}");
    }
}

#[test]
fn pla04_displays_show_the_result_in_hex() {
    let (m, n) = r2();
    let fs = files(&m, &n, Language::PtBr);
    let t = de2(&fs);
    for (d, segs) in SEGMENTS.iter().enumerate() {
        assert!(t.contains(&format!("when \"{d:04b}\" => return \"{segs:07b}\";")));
    }
    assert!(t.contains("    HEX0 <= seg7(R(3 downto 0));"));
    assert!(t.contains("    HEX1 <= seg7(\"0\" & R(6 downto 4));"));
    for h in 2..8 {
        assert!(
            t.contains(&format!("    HEX{h} <= (others => '1');")),
            "HEX{h}"
        );
    }

    // W = 21: six displays, the last with a single bit.
    let (m, n) = single_row("1048576A", &[('A', 1)]);
    let fs = files(&m, &n, Language::PtBr);
    let t = de2(&fs);
    assert!(t.contains("    HEX4 <= seg7(R(19 downto 16));"));
    assert!(t.contains("    HEX5 <= seg7(\"000\" & R(20 downto 20));"));
    assert!(t.contains("    HEX6 <= (others => '1');"));
}

// ---------------------------------------------------------------- PLA-05

fn assignments(qsf: &str) -> Vec<(String, String)> {
    qsf.lines()
        .filter_map(|l| {
            let rest = l.strip_prefix("set_location_assignment ")?;
            let (pin, port) = rest.split_once(" -to ")?;
            Some((port.to_string(), pin.to_string()))
        })
        .collect()
}

#[test]
fn pla05_quartus_project() {
    let (m, n) = r2();
    let fs = files(&m, &n, Language::PtBr);
    let qsf = file(&fs, "dotree_circuito.qsf");
    for line in [
        "set_global_assignment -name FAMILY \"Cyclone II\"",
        "set_global_assignment -name DEVICE EP2C35F672C6",
        "set_global_assignment -name TOP_LEVEL_ENTITY dotree_circuito_de2",
        "set_global_assignment -name VHDL_INPUT_VERSION VHDL_1993",
    ] {
        assert!(qsf.lines().any(|l| l == line), "missing {line}");
    }
    let sources: Vec<&str> = qsf
        .lines()
        .filter_map(|l| l.strip_prefix("set_global_assignment -name VHDL_FILE "))
        .collect();
    assert_eq!(
        sources,
        [
            "full_adder.vhd",
            "half_adder.vhd",
            "dotree_circuito.vhd",
            "dotree_circuito_comportamental.vhd",
            "dotree_circuito_de2.vhd"
        ]
    );

    let a = assignments(qsf);
    assert_eq!(a.len(), 92);
    let ports: BTreeSet<&str> = a.iter().map(|(p, _)| p.as_str()).collect();
    let pins: BTreeSet<&str> = a.iter().map(|(_, p)| p.as_str()).collect();
    assert_eq!(
        (ports.len(), pins.len()),
        (92, 92),
        "distinct ports and pins"
    );
    let mut expected: BTreeSet<String> = BTreeSet::new();
    for i in 0..18 {
        expected.insert(format!("SW[{i}]"));
        expected.insert(format!("LEDR[{i}]"));
    }
    for h in 0..8 {
        for i in 0..7 {
            expected.insert(format!("HEX{h}[{i}]"));
        }
    }
    assert_eq!(ports, expected.iter().map(String::as_str).collect());
    // Two pins the DE2 manual states in its text.
    assert!(a.contains(&("SW[0]".into(), "PIN_N25".into())));
    assert!(a.contains(&("LEDR[0]".into(), "PIN_AE23".into())));

    let qpf = file(&fs, "dotree_circuito.qpf");
    assert!(
        qpf.lines()
            .any(|l| l == "PROJECT_REVISION = \"dotree_circuito\"")
    );
}

/// Compares the whole pin table with a DE2 pin file given in `DE2_PINS_FILE` (a `.qsf`
/// with `set_location_assignment` lines, or the `.csv` with `port,pin` lines). The file
/// stays outside the repository; without the variable the comparison is skipped.
#[test]
fn pla05_pins_match_the_board_pin_file() {
    let Ok(path) = std::env::var("DE2_PINS_FILE") else {
        eprintln!("DE2_PINS_FILE not set: skipping the comparison with the board pin file");
        return;
    };
    let text = std::fs::read_to_string(&path).expect("pin file");
    let mut theirs: BTreeMap<String, String> = BTreeMap::new();
    for line in text.lines() {
        let pair = if let Some(rest) = line.trim().strip_prefix("set_location_assignment ") {
            rest.split_once(" -to ")
                .map(|(pin, port)| (port.trim(), pin.trim()))
        } else {
            line.split_once(',')
                .map(|(port, pin)| (port.trim(), pin.trim()))
        };
        if let Some((port, pin)) = pair {
            theirs.insert(port.to_string(), pin.to_string());
        }
    }
    for (port, pin) in DE2_PINS {
        assert_eq!(theirs.get(port).map(String::as_str), Some(pin), "{port}");
    }
    eprintln!("{} pins match {path}", DE2_PINS.len());
}

// ---------------------------------------------------------------- PLA-06

#[test]
fn pla06_board_testbench() {
    let (m, n) = r2();
    for (lang, done) in [
        (Language::PtBr, "PLACA: TODOS OS 16384 CASOS CORRETOS"),
        (Language::En, "BOARD: ALL 16384 CASES CORRECT"),
    ] {
        let fs = files(&m, &n, lang);
        let tb = file(&fs, "tb_dotree_circuito_de2.vhd");
        assert!(tb.contains("entity tb_dotree_circuito_de2 is"));
        assert!(tb.contains(
            "dut : entity work.dotree_circuito_de2 port map (SW => SW, LEDR => LEDR, HEX0 => HEX0"
        ));
        assert!(tb.contains("for n in 0 to 16383 loop"));
        assert!(tb.contains("SW(13 downto 0) <= v;"));
        assert!(tb.contains(done), "{lang:?}");
    }
}

// ---------------------------------------------------------------- PLA-07

#[test]
fn pla07_names_used_by_the_board_top_are_refused() {
    let (m, n) = r2();
    for bad in ["dut", "seg7", "SW", "ledr", "Hex3"] {
        assert_eq!(
            generate(&m, &n, bad, Language::PtBr),
            Err(VhdlError::InvalidName),
            "{bad}"
        );
    }
    let fs = generate(&m, &n, "meu_somador", Language::PtBr).unwrap();
    assert!(fs.iter().any(|f| f.name == "meu_somador_de2.vhd"));
    assert!(fs.iter().any(|f| f.name == "meu_somador.qsf"));
}
