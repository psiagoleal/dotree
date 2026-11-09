// Caminho relativo: app/core/tests/heights.rs
//! Spec 02, CMP-16 — the height table, level by level (MT-30).

mod common;

use common::{form, r1, replay_r2_tree};
use dotree_core::heights::{
    HeightHeaders, HeightRow, RowKind, height_table, height_table_markdown,
};

/// The spec itself: the R2 table is compared with the one written in spec 02.
const SPEC_02: &str = include_str!("../../../docs/specs/02-compressao.md");

const PT: HeightHeaders = HeightHeaders {
    level: "Nível",
    current: "atual",
    result: "Resultado",
};

/// The table of spec 02 that starts with the header line of the height table.
fn spec_table() -> String {
    let start = SPEC_02
        .find("| Nível | $2^6$")
        .expect("spec 02 has the R2 height table");
    SPEC_02[start..]
        .lines()
        .take_while(|l| l.starts_with('|'))
        .map(|l| format!("{l}\n"))
        .collect()
}

fn row(level: usize, kind: RowKind, msb_first: &[u32], fa: usize, ha: usize) -> HeightRow {
    HeightRow {
        level,
        kind,
        heights: msb_first.iter().rev().copied().collect(),
        fa,
        ha,
    }
}

#[test]
fn cmp16_r2_complete_matches_the_spec() {
    let mut r = r1();
    replay_r2_tree(&mut r);
    r.close_with_cpa().unwrap();
    assert_eq!(
        height_table(&r),
        [
            row(0, RowKind::Level, &[0, 0, 1, 2, 5, 7, 5], 2, 1),
            row(1, RowKind::Level, &[0, 0, 1, 3, 6, 3, 5], 4, 0),
            row(2, RowKind::Level, &[0, 0, 2, 3, 3, 1, 5], 3, 1),
            row(3, RowKind::Cpa, &[0, 1, 2, 2, 1, 2, 3], 4, 2),
            row(4, RowKind::Result, &[1, 1, 1, 1, 1, 1, 1], 0, 0),
        ]
    );
    assert_eq!(height_table_markdown(&height_table(&r), PT), spec_table());
}

#[test]
fn cmp16_the_current_level_counts_its_pending_cells() {
    let mut r = r1();
    assert_eq!(
        height_table(&r),
        [row(0, RowKind::Current, &[0, 0, 1, 2, 5, 7, 5], 0, 0)]
    );
    form(&mut r, 1, &["a0_b1", "a1_b0", "c0_d1"]);
    form(&mut r, 2, &["a0_b2", "a1_b1"]);
    assert_eq!(
        height_table(&r),
        [row(0, RowKind::Current, &[0, 0, 1, 2, 5, 7, 5], 1, 1)]
    );
}

#[test]
fn cmp16_before_the_cpa_the_last_row_is_the_current_level() {
    let mut r = r1();
    replay_r2_tree(&mut r);
    let table = height_table(&r);
    assert_eq!(table.len(), 4);
    assert_eq!(
        table[3],
        row(3, RowKind::Current, &[0, 1, 2, 2, 1, 2, 3], 0, 0)
    );
    let md = height_table_markdown(&table, PT);
    assert!(
        md.ends_with("| 3 (atual) | 0 | 1 | 2 | 2 | 1 | 2 | 3 | 0 | 0 |\n"),
        "{md}"
    );
}

#[test]
fn cmp16_headers_come_from_the_interface() {
    let r = r1();
    let en = HeightHeaders {
        level: "Level",
        current: "current",
        result: "Result",
    };
    let md = height_table_markdown(&height_table(&r), en);
    assert!(md.starts_with("| Level | $2^6$ |"), "{md}");
    assert!(md.contains("| 0 (current) |"), "{md}");
}
