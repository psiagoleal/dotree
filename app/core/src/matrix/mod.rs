// Caminho relativo: app/core/src/matrix/mod.rs
//! Dot matrix (spec 01, "Como a matriz é formada"; MT-4).
//!
//! Each combined term `k·X1·…·Xn` of the expanded expression becomes one copy per 1-bit of
//! `k` (binary decomposition), shifted by that bit's position. Each copy contributes one dot
//! per combination of variable bits. Copies of a term share their signals: in `3E`, the dot
//! of `e0` in column 0 and the one in column 1 are the same wire.

use std::collections::{BTreeMap, HashMap};

use crate::expr::{Analysis, Monomial};

/// Largest accepted number of dots in the initial matrix (EXP-08).
pub const MAX_DOTS: usize = 256;

/// One bit of one variable, e.g. `a1` = bit 1 of `A`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Factor {
    pub var: char,
    pub bit: u32,
}

/// A wire carried by input dots: a single variable bit (`e0`) or the AND of several
/// (`a1_b0`, a partial product). Factors are sorted by variable.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Signal {
    pub factors: Vec<Factor>,
}

impl Signal {
    /// VHDL-safe identifier shared by every artifact (spec 05, "Nomes"): `a1_b0`, `e0`.
    pub fn ident(&self) -> String {
        self.factors
            .iter()
            .map(|f| format!("{}{}", f.var.to_ascii_lowercase(), f.bit))
            .collect::<Vec<_>>()
            .join("_")
    }

    /// True for partial products, i.e. AND gates with 2 or more inputs.
    pub fn is_and(&self) -> bool {
        self.factors.len() > 1
    }
}

/// An input dot of the level-0 matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dot {
    /// Position in canonical generation order, starting at 0.
    pub id: usize,
    /// Weight `2^column`.
    pub column: u32,
    /// Index into [`DotMatrix::signals`].
    pub signal: usize,
    /// Index into [`DotMatrix::terms`]: the combined term, used for its color (COR-01).
    pub term: usize,
    /// Shift of the copy this dot belongs to (the `m` of `2^m`).
    pub shift: u32,
}

/// The level-0 dot matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DotMatrix {
    /// Combined terms, in expression order (one color each).
    pub terms: Vec<Monomial>,
    /// Widths of the variables in use.
    pub widths: BTreeMap<char, u32>,
    /// Distinct signals, in order of first appearance among the dots.
    pub signals: Vec<Signal>,
    /// Dots in canonical generation order: term, then copy (shift ascending), then bit
    /// combination in lexicographic order with the first variable varying slowest.
    pub dots: Vec<Dot>,
    /// Value of the expression with every input at its maximum.
    pub v_max: u128,
    /// Output width `W = ceil(log2(v_max + 1))`.
    pub output_width: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatrixError {
    /// More than `MAX_DOTS` dots (EXP-08).
    TooManyDots { count: usize, limit: usize },
}

impl DotMatrix {
    /// Builds the level-0 matrix from a validated expression (EXP-01 to EXP-03, EXP-08).
    pub fn build(analysis: &Analysis) -> Result<DotMatrix, MatrixError> {
        let terms = analysis.polynomial.terms.clone();
        let widths = analysis.widths.clone();
        let width_of = |v: &char| widths[v];

        // Count first, so a huge expression is refused before anything is allocated.
        let count = terms.iter().fold(0usize, |acc, t| {
            let per_copy = t
                .vars
                .iter()
                .fold(1usize, |p, v| p.saturating_mul(width_of(v) as usize));
            acc.saturating_add((t.coefficient.count_ones() as usize).saturating_mul(per_copy))
        });
        if count > MAX_DOTS {
            return Err(MatrixError::TooManyDots {
                count,
                limit: MAX_DOTS,
            });
        }

        let mut signals: Vec<Signal> = Vec::new();
        let mut signal_index: HashMap<Signal, usize> = HashMap::new();
        let mut dots = Vec::with_capacity(count);

        for (term_idx, term) in terms.iter().enumerate() {
            let sizes: Vec<u32> = term.vars.iter().map(width_of).collect();
            for shift in (0..u64::BITS).filter(|b| term.coefficient >> b & 1 == 1) {
                for bits in combinations(&sizes) {
                    let signal = Signal {
                        factors: term
                            .vars
                            .iter()
                            .zip(&bits)
                            .map(|(&var, &bit)| Factor { var, bit })
                            .collect(),
                    };
                    let next = signals.len();
                    let id = *signal_index.entry(signal.clone()).or_insert_with(|| {
                        signals.push(signal);
                        next
                    });
                    dots.push(Dot {
                        id: dots.len(),
                        column: bits.iter().sum::<u32>() + shift,
                        signal: id,
                        term: term_idx,
                        shift,
                    });
                }
            }
        }

        let v_max: u128 = terms
            .iter()
            .map(|t| {
                t.vars.iter().fold(u128::from(t.coefficient), |acc, v| {
                    acc * ((1u128 << width_of(v)) - 1)
                })
            })
            .sum();
        // v_max >= 1: every coefficient and every width is at least 1.
        let output_width = u128::BITS - v_max.leading_zeros();

        Ok(DotMatrix {
            terms,
            widths,
            signals,
            dots,
            v_max,
            output_width,
        })
    }

    /// Number of dots in each column `0..W`.
    pub fn heights(&self) -> Vec<u32> {
        let mut h = vec![0; self.output_width as usize];
        for d in &self.dots {
            // A dot's weight never exceeds v_max, so its column is always below W.
            h[d.column as usize] += 1;
        }
        h
    }

    /// Number of AND gates: distinct partial products (EXP-03).
    pub fn and_gates(&self) -> usize {
        self.signals.iter().filter(|s| s.is_and()).count()
    }
}

/// Every combination of bit indices `0..sizes[i]`, in lexicographic order with the first
/// position varying slowest: (0,0), (0,1), …, (1,0), …
fn combinations(sizes: &[u32]) -> Vec<Vec<u32>> {
    let mut out = vec![Vec::new()];
    for &size in sizes {
        out = out
            .into_iter()
            .flat_map(|prefix| {
                (0..size).map(move |b| {
                    let mut next = prefix.clone();
                    next.push(b);
                    next
                })
            })
            .collect();
    }
    out
}
