// Caminho relativo: app/core/tests/expr.rs
//! Spec 01 — expression language (MT-3). One group per criterion.

use std::collections::BTreeMap;

use dotree_core::expr::{
    Analysis, Expected, ExprError, MAX_INPUT_BITS, SyntaxError, SyntaxErrorKind, Warning, analyze,
    parse,
};

fn expanded(src: &str) -> String {
    parse(src)
        .unwrap_or_else(|e| panic!("{src:?} should parse, got {e:?}"))
        .to_string()
}

fn syntax_error(src: &str) -> SyntaxError {
    match parse(src) {
        Err(ExprError::Syntax(e)) => e,
        other => panic!("{src:?} should be a syntax error, got {other:?}"),
    }
}

fn widths(pairs: &[(char, u32)]) -> BTreeMap<char, u32> {
    pairs.iter().copied().collect()
}

/// Widths of reference case R1 (spec 00).
fn r1_widths() -> BTreeMap<char, u32> {
    widths(&[
        ('A', 3),
        ('B', 3),
        ('C', 2),
        ('D', 2),
        ('E', 2),
        ('F', 1),
        ('G', 1),
    ])
}

// ---------------------------------------------------------------- EXP-01: expansion

#[test]
fn exp01_r1_expands_into_its_five_terms() {
    assert_eq!(expanded("A*B + C*D + 3(E+F) + G"), "AB + CD + 3E + 3F + G");
}

#[test]
fn exp01_all_product_spellings_are_equivalent() {
    for src in ["A*B", "A×B", "A·B", "AB", "A * B", " A B "] {
        assert_eq!(expanded(src), "AB", "{src:?}");
    }
}

#[test]
fn exp01_blanks_are_ignored() {
    assert_eq!(expanded("3 (E + F)"), "3E + 3F");
}

#[test]
fn exp01_like_terms_combine() {
    assert_eq!(expanded("AB + AB"), "2AB");
    assert_eq!(expanded("2A + A"), "3A");
    assert_eq!(expanded("BA + AB"), "2AB");
}

#[test]
fn exp01_variables_inside_a_monomial_are_sorted() {
    assert_eq!(expanded("CBA"), "ABC");
}

#[test]
fn exp01_product_of_sums_distributes() {
    assert_eq!(expanded("(A+B)(C+D)"), "AC + AD + BC + BD");
    assert_eq!(expanded("2(A+B)*3C"), "6AC + 6BC");
}

#[test]
fn exp01_terms_keep_first_appearance_order() {
    assert_eq!(expanded("G + A*B + G"), "2G + AB");
}

#[test]
fn exp01_nested_parentheses() {
    assert_eq!(expanded("2((A))"), "2A");
    assert_eq!(expanded("(2(A+B))C"), "2AC + 2BC");
}

// ---------------------------------------------------------------- EXP-05: syntax errors

#[test]
fn exp05_dangling_plus() {
    let e = syntax_error("A+");
    assert_eq!(e.kind, SyntaxErrorKind::UnexpectedEnd);
    assert_eq!(e.column, 3);
    assert_eq!(e.expected, vec![Expected::Operand]);
}

#[test]
fn exp05_double_star() {
    let e = syntax_error("A**B");
    assert_eq!(e.kind, SyntaxErrorKind::UnexpectedToken('*'));
    assert_eq!(e.column, 3);
    assert_eq!(e.expected, vec![Expected::Operand]);
}

#[test]
fn exp05_unclosed_parenthesis() {
    let e = syntax_error("3(E+F");
    assert_eq!(e.kind, SyntaxErrorKind::UnexpectedEnd);
    assert_eq!(e.column, 6);
    assert_eq!(e.expected, vec![Expected::Operator, Expected::CloseParen]);
}

#[test]
fn exp05_stray_close_parenthesis() {
    let e = syntax_error("A)");
    assert_eq!(e.kind, SyntaxErrorKind::UnexpectedToken(')'));
    assert_eq!(e.column, 2);
    assert_eq!(e.expected, vec![Expected::Operator, Expected::End]);
}

#[test]
fn exp05_lowercase_variable() {
    let e = syntax_error("a*b");
    assert_eq!(e.kind, SyntaxErrorKind::LowercaseVariable('a'));
    assert_eq!(e.column, 1);
}

#[test]
fn exp05_digit_after_variable() {
    let e = syntax_error("A2");
    assert_eq!(e.kind, SyntaxErrorKind::UnexpectedToken('2'));
    assert_eq!(e.column, 2);
    assert_eq!(e.expected, vec![Expected::Operator, Expected::End]);
}

#[test]
fn exp05_zero_coefficient() {
    let e = syntax_error("0A");
    assert_eq!(e.kind, SyntaxErrorKind::ZeroCoefficient);
    assert_eq!(e.column, 1);
}

#[test]
fn exp05_empty_expression() {
    for src in ["", "   "] {
        let e = syntax_error(src);
        assert_eq!(e.kind, SyntaxErrorKind::Empty, "{src:?}");
        assert_eq!(e.column, 1);
        assert_eq!(e.expected, vec![Expected::Operand]);
    }
}

#[test]
fn exp05_unknown_character() {
    let e = syntax_error("A$B");
    assert_eq!(e.kind, SyntaxErrorKind::UnexpectedChar('$'));
    assert_eq!(e.column, 2);
}

#[test]
fn exp05_column_counts_characters_not_bytes() {
    // "×" and "·" take two bytes in UTF-8; the column must still count one each.
    let e = syntax_error("A×B·C+");
    assert_eq!(e.column, 7);
}

#[test]
fn exp05_huge_literal() {
    let e = syntax_error("99999999999A");
    assert_eq!(e.kind, SyntaxErrorKind::NumberTooLarge);
    assert_eq!(e.column, 1);
}

// ---------------------------------------------------------------- EXP-06: widths

#[test]
fn exp06_missing_width_names_the_variable() {
    let mut w = r1_widths();
    w.remove(&'G');
    assert_eq!(
        analyze("A*B + C*D + 3(E+F) + G", &w),
        Err(ExprError::MissingWidth { var: 'G' })
    );
}

#[test]
fn exp06_width_out_of_range() {
    assert_eq!(
        analyze("A", &widths(&[('A', 0)])),
        Err(ExprError::WidthOutOfRange { var: 'A', width: 0 })
    );
    assert_eq!(
        analyze("A", &widths(&[('A', 17)])),
        Err(ExprError::WidthOutOfRange {
            var: 'A',
            width: 17
        })
    );
    assert!(analyze("A", &widths(&[('A', 16)])).is_ok());
    assert!(analyze("A", &widths(&[('A', 1)])).is_ok());
}

#[test]
fn exp06_r1_is_valid() {
    let a: Analysis = analyze("A*B + C*D + 3(E+F) + G", &r1_widths()).unwrap();
    assert_eq!(a.polynomial.to_string(), "AB + CD + 3E + 3F + G");
    assert_eq!(a.widths, r1_widths());
    assert!(a.warnings.is_empty());
}

// ---------------------------------------------------------------- EXP-07: powers

#[test]
fn exp07_repeated_variable_in_a_monomial() {
    for src in ["A*A", "AA", "A(A+B)", "(A+B)(A+C)"] {
        assert_eq!(
            parse(src),
            Err(ExprError::RepeatedVariable { var: 'A' }),
            "{src:?}"
        );
    }
}

// ---------------------------------------------------------------- EXP-08: input bits

#[test]
fn exp08_input_bits_limit() {
    assert_eq!(MAX_INPUT_BITS, 32);
    assert!(analyze("A*B", &widths(&[('A', 16), ('B', 16)])).is_ok());
    assert_eq!(
        analyze("A*B + C", &widths(&[('A', 16), ('B', 16), ('C', 1)])),
        Err(ExprError::TooManyInputBits {
            total: 33,
            limit: 32
        })
    );
}

// ---------------------------------------------------------------- EXP-11: unused widths

#[test]
fn exp11_unused_width_is_a_warning() {
    let mut w = r1_widths();
    w.insert('Z', 4);
    let a = analyze("A*B + C*D + 3(E+F) + G", &w).unwrap();
    assert_eq!(a.warnings, vec![Warning::UnusedWidth { var: 'Z' }]);
    assert!(!a.widths.contains_key(&'Z'));
}

// ---------------------------------------------------------------- EXP-13: constants

#[test]
fn exp13_additive_constants_are_refused() {
    for src in ["A + 5", "2", "3(A + 1)"] {
        assert_eq!(parse(src), Err(ExprError::ConstantTerm), "{src:?}");
    }
}

#[test]
fn coefficient_overflow_is_reported_not_panicked() {
    assert_eq!(
        parse("4294967295(4294967295(4294967295A))"),
        Err(ExprError::CoefficientTooLarge)
    );
}
