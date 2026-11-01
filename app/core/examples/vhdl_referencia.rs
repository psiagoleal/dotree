// Caminho relativo: app/core/examples/vhdl_referencia.rs
//! Writes the VHDL of the reference cases for the GHDL job in CI (spec 06, VHD-01, VHD-04).
//!
//!   cargo run -p dotree-core --example vhdl_referencia -- <dir>
//!
//! Each case goes to its own subdirectory, and `<dir>/casos.txt` lists, one per line:
//! `<subdir> <testbench entity> <expect>`, where `<expect>` is `ok` (the testbench must
//! report all cases correct) or `falha` (it must fail: a deliberately broken circuit).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use dotree_core::expr::analyze;
use dotree_core::matrix::DotMatrix;
use dotree_core::netlist::{Netlist, Wire};
use dotree_core::reduce::Reduction;
use dotree_core::vhdl::{DEFAULT_NAME, Language, generate};

fn matrix(src: &str, widths: &[(char, u32)]) -> DotMatrix {
    let w: BTreeMap<char, u32> = widths.iter().copied().collect();
    DotMatrix::build(&analyze(src, &w).expect("valid expression")).expect("valid matrix")
}

/// Forms the cells of one level by `(column, signals)`, then concludes it.
fn level(r: &mut Reduction, cells: &[(u32, &[&str])]) {
    for &(column, signals) in cells {
        let ids: Vec<usize> = signals
            .iter()
            .map(|s| {
                r.points()
                    .iter()
                    .find(|p| p.column == column && r.signal(p) == *s)
                    .expect("point")
                    .id
            })
            .collect();
        r.form_cell(&ids).expect("valid cell");
    }
    r.conclude_level().expect("level with cells");
}

/// R2 (spec 00): the reference solution of R1.
fn r2() -> (DotMatrix, Netlist) {
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
    let mut r = Reduction::new(&m);
    level(
        &mut r,
        &[
            (1, &["a0_b1", "a1_b0", "c0_d1"]),
            (1, &["c1_d0", "e1", "e0"]),
            (2, &["a0_b2", "a1_b1"]),
        ],
    );
    level(
        &mut r,
        &[
            (1, &["f0", "s1", "s2"]),
            (2, &["a2_b0", "c1_d1", "e1"]),
            (2, &["s3", "k1", "k2"]),
            (3, &["a1_b2", "a2_b1", "k3"]),
        ],
    );
    level(
        &mut r,
        &[
            (0, &["a0_b0", "c0_d0", "e0"]),
            (2, &["s5", "k4", "s6"]),
            (3, &["k5", "s7", "k6"]),
            (4, &["a2_b2", "k7"]),
        ],
    );
    r.close_with_cpa().expect("CPA");
    let n = Netlist::from_reduction(&m, &r).expect("complete");
    (m, n)
}

/// A simple deterministic reduction: as many FAs as possible per level, an HA when a level
/// would otherwise be empty, then the CPA as soon as it is allowed.
fn greedy(m: &DotMatrix) -> Netlist {
    let mut r = Reduction::new(m);
    while !r.is_complete() {
        if r.cpa_status().is_ok() {
            r.close_with_cpa().expect("CPA");
            continue;
        }
        for column in 0..m.output_width {
            let ids: Vec<usize> = r
                .points()
                .iter()
                .filter(|p| p.column == column)
                .map(|p| p.id)
                .collect();
            for chunk in ids.chunks_exact(3) {
                r.form_cell(chunk).expect("FA");
            }
        }
        if r.pending().is_empty() {
            let column = r
                .heights()
                .iter()
                .position(|&h| h >= 2)
                .expect("a column to add") as u32;
            let ids: Vec<usize> = r
                .points()
                .iter()
                .filter(|p| p.column == column)
                .map(|p| p.id)
                .take(2)
                .collect();
            r.form_cell(&ids).expect("HA");
        }
        r.conclude_level().expect("level");
    }
    Netlist::from_reduction(m, &r).expect("complete")
}

fn write(
    dir: &Path,
    case: &str,
    m: &DotMatrix,
    n: &Netlist,
    lang: Language,
    expect: &str,
    list: &mut String,
) {
    let sub = dir.join(case);
    fs::create_dir_all(&sub).expect("create dir");
    let files = generate(m, n, DEFAULT_NAME, lang).expect("generate");
    let order: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
    for f in &files {
        fs::write(sub.join(&f.name), &f.contents).expect("write file");
    }
    // Analysis order: components, top, testbench (the order `generate` returns).
    fs::write(sub.join("ordem.txt"), order.join(" ") + "\n").expect("write order");
    list.push_str(&format!("{case} tb_{DEFAULT_NAME} {expect}\n"));
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .expect("usage: vhdl_referencia <dir>");
    let dir = Path::new(&dir);
    let mut list = String::new();

    let (m, n) = r2();
    write(dir, "r2", &m, &n, Language::PtBr, "ok", &mut list);

    // VER-03 / VHD-04: FA16 gets k14 (weight 2^3) where k15 (weight 2^4) belongs.
    let mut broken = n.clone();
    broken.cells[15].inputs[2] = Wire::Carry(14);
    write(
        dir,
        "r2_quebrado",
        &m,
        &broken,
        Language::PtBr,
        "falha",
        &mut list,
    );

    let m3 = matrix("A", &[('A', 4)]);
    write(dir, "r3", &m3, &greedy(&m3), Language::En, "ok", &mut list);

    let m4 = matrix("A*B", &[('A', 4), ('B', 4)]);
    write(
        dir,
        "r4",
        &m4,
        &greedy(&m4),
        Language::PtBr,
        "ok",
        &mut list,
    );

    // 21 input bits: the testbench uses the 4096-vector sample.
    let ms = matrix("A + 2048B", &[('A', 11), ('B', 10)]);
    write(
        dir,
        "amostra",
        &ms,
        &greedy(&ms),
        Language::En,
        "ok",
        &mut list,
    );

    fs::write(dir.join("casos.txt"), list).expect("write list");
}
