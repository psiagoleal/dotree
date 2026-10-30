// Caminho relativo: app/core/src/expr/mod.rs
//! Expression language (spec 01, "Linguagem da expressão").
//!
//! Text such as `A*B + C*D + 3(E+F) + G` becomes a [`Polynomial`]: a sum of monomials,
//! fully expanded, with like terms combined and integer coefficients. Errors come out as
//! codes with parameters, never as natural-language text (IDI-05).

mod parser;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Narrowest and widest accepted variable width, in bits (EXP-06).
pub const MIN_WIDTH: u32 = 1;
pub const MAX_WIDTH: u32 = 16;
/// Largest accepted sum of the widths of the variables in use (EXP-08).
pub const MAX_INPUT_BITS: u32 = 32;

/// A product of distinct variables times a positive integer coefficient.
/// `vars` is sorted and has no repetitions; it is never empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Monomial {
    pub coefficient: u64,
    pub vars: Vec<char>,
}

/// Sum of monomials, in order of first appearance in the expanded expression.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Polynomial {
    pub terms: Vec<Monomial>,
}

impl fmt::Display for Polynomial {
    /// Compact form used by tests and by the VHDL header: `AB + CD + 3E + 3F + G`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, term) in self.terms.iter().enumerate() {
            if i > 0 {
                f.write_str(" + ")?;
            }
            if term.coefficient != 1 {
                write!(f, "{}", term.coefficient)?;
            }
            for v in &term.vars {
                write!(f, "{v}")?;
            }
        }
        Ok(())
    }
}

/// What the parser was ready to accept where it stopped (EXP-05).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expected {
    /// A variable, a coefficient or `(`.
    Operand,
    /// `+`, `*`, `×`, `·` or an implicit product.
    Operator,
    CloseParen,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxErrorKind {
    /// Nothing but blanks.
    Empty,
    /// A character that is not part of the language.
    UnexpectedChar(char),
    /// Variables are single upper-case letters.
    LowercaseVariable(char),
    /// A valid token in a place where it cannot go (e.g. `A2`, `A)`, `A**B`).
    UnexpectedToken(char),
    /// The text ended before the expression did.
    UnexpectedEnd,
    /// Coefficients start at 1.
    ZeroCoefficient,
    /// A number literal above `u32::MAX`.
    NumberTooLarge,
}

/// Syntax error with the 1-based character column where it was found (EXP-05).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxError {
    pub kind: SyntaxErrorKind,
    pub column: usize,
    pub expected: Vec<Expected>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprError {
    Syntax(SyntaxError),
    /// A variable used by the expression has no width (EXP-06).
    MissingWidth {
        var: char,
    },
    /// A width outside `MIN_WIDTH..=MAX_WIDTH` (EXP-06).
    WidthOutOfRange {
        var: char,
        width: u32,
    },
    /// The same variable twice in one monomial, i.e. a power (EXP-07).
    RepeatedVariable {
        var: char,
    },
    /// An additive constant such as `A + 5` (out of scope in v0.1, EXP-13).
    ConstantTerm,
    /// A coefficient that does not fit in 64 bits after expansion.
    CoefficientTooLarge,
    /// The widths of the variables in use add up to more than `MAX_INPUT_BITS` (EXP-08).
    TooManyInputBits {
        total: u32,
        limit: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Warning {
    /// A width was given for a variable the expression does not use (EXP-11).
    UnusedWidth { var: char },
}

/// A validated expression, ready to become a dot matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Analysis {
    pub polynomial: Polynomial,
    /// Widths of the variables in use only.
    pub widths: BTreeMap<char, u32>,
    pub warnings: Vec<Warning>,
}

/// Parses and expands an expression (EXP-01, EXP-05, EXP-07, EXP-13).
pub fn parse(src: &str) -> Result<Polynomial, ExprError> {
    parser::parse(src)
}

/// Parses, expands and validates the widths (EXP-06, EXP-08, EXP-11).
pub fn analyze(src: &str, widths: &BTreeMap<char, u32>) -> Result<Analysis, ExprError> {
    let polynomial = parse(src)?;
    let used: BTreeSet<char> = polynomial
        .terms
        .iter()
        .flat_map(|t| t.vars.iter().copied())
        .collect();

    // Checked in alphabetical order, so the reported variable is deterministic.
    let mut used_widths = BTreeMap::new();
    for &var in &used {
        let width = *widths.get(&var).ok_or(ExprError::MissingWidth { var })?;
        if !(MIN_WIDTH..=MAX_WIDTH).contains(&width) {
            return Err(ExprError::WidthOutOfRange { var, width });
        }
        used_widths.insert(var, width);
    }

    let total: u32 = used_widths.values().sum();
    if total > MAX_INPUT_BITS {
        return Err(ExprError::TooManyInputBits {
            total,
            limit: MAX_INPUT_BITS,
        });
    }

    let warnings = widths
        .keys()
        .filter(|v| !used.contains(v))
        .map(|&var| Warning::UnusedWidth { var })
        .collect();

    Ok(Analysis {
        polynomial,
        widths: used_widths,
        warnings,
    })
}
