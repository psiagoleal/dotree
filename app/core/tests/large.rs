// Caminho relativo: app/core/tests/large.rs
//! MT-25a: the 32 x 32 multiplier, at the limits of spec 01 (32 bits per variable, 64
//! input bits, 1024 dots). Matrix, the three solutions, verification by sample, netlist,
//! diagram, heights table and VHDL; the GHDL run is `nomes`' neighbour `mult32` in
//! `examples/vhdl_referencia.rs`.

use std::collections::BTreeMap;

use dotree_core::diagram::{Layout, wiring_table};
use dotree_core::expr::analyze;
use dotree_core::heights::height_table;
use dotree_core::help::{dadda_solution, parhami_solution, wallace_solution};
use dotree_core::matrix::DotMatrix;
use dotree_core::netlist::Netlist;
use dotree_core::verify::{Coverage, SAMPLE_SIZE, Verifier};
use dotree_core::vhdl::{DEFAULT_NAME, Language, generate};

fn mult32() -> DotMatrix {
    let w: BTreeMap<char, u32> = [('A', 32), ('B', 32)].into_iter().collect();
    DotMatrix::build(&analyze("A*B", &w).unwrap()).unwrap()
}

#[test]
fn mt25a_the_32_by_32_matrix() {
    let m = mult32();
    assert_eq!(m.dots.len(), 1024);
    assert_eq!(m.output_width, 64);
    assert_eq!(m.and_gates(), 1024);
    let h = m.heights();
    assert_eq!(h[31], 32);
    assert_eq!(h[0], 1);
    assert_eq!(h[63], 0);
}

#[test]
fn mt25a_every_solution_closes_the_32_by_32() {
    let m = mult32();
    for (name, r) in [
        ("dadda", dadda_solution(&m)),
        ("parhami", parhami_solution(&m)),
        ("wallace", wallace_solution(&m)),
    ] {
        assert!(r.is_complete(), "{name}");
        let n = Netlist::from_reduction(&m, &r).unwrap();
        assert!(n.complete, "{name}");
    }
}

#[test]
fn mt25a_dadda_on_32_by_32_is_verified_by_sample() {
    let m = mult32();
    let n = Netlist::from_reduction(&m, &dadda_solution(&m)).unwrap();
    // Dadda's counts for an n x n multiplier: n^2 - 4n + 3 FA and n - 1 HA before the CPA.
    let before_cpa = n.cells.iter().filter(|c| !c.cpa).count();
    assert_eq!(before_cpa, 899 + 31);
    let p = Verifier::new(n).run_to_end();
    assert_eq!(p.coverage, Coverage::Sample);
    assert_eq!(p.checked, SAMPLE_SIZE);
    assert_eq!(p.counterexample, None);
}

#[test]
fn mt25a_tables_diagram_and_vhdl_of_32_by_32() {
    let m = mult32();
    let r = dadda_solution(&m);
    let n = Netlist::from_reduction(&m, &r).unwrap();
    assert_eq!(wiring_table(&n).len(), n.cells.len());
    assert!(!height_table(&r).is_empty());
    let layout = Layout::new(&n);
    assert_eq!(layout.blocks.len(), n.cells.len());
    let fs = generate(&m, &n, DEFAULT_NAME, Language::PtBr).unwrap();
    let tb = &fs
        .iter()
        .find(|f| f.name == format!("tb_{DEFAULT_NAME}.vhd"))
        .unwrap()
        .contents;
    assert!(
        tb.contains("4096"),
        "the testbench takes the sample above 20 bits"
    );
}
