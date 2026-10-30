// Caminho relativo: app/core/src/expr/parser.rs
//! Lexer and recursive-descent parser that expands while it parses.
//!
//! Grammar (spaces ignored):
//!   expr   := term ( '+' term )*
//!   term   := factor ( ( '*' | '×' | '·' ) factor | factor-starting-with-letter-or-'(' )*
//!   factor := number | VAR | '(' expr ')'
//!
//! A number cannot follow a factor by juxtaposition (`A2` is an error), so `2A` and
//! `3(E+F)` read as coefficient times operand while `A2` stays unambiguous.

use super::{Expected, ExprError, Monomial, Polynomial, SyntaxError, SyntaxErrorKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tok {
    Var(char),
    Num(u32),
    Plus,
    Times(char),
    LParen,
    RParen,
    /// A lexical error, kept in place so the parser reports the earliest error in the text.
    Bad(SyntaxErrorKind),
}

/// A token, the first character of its text, and its 1-based character column.
#[derive(Debug, Clone, Copy)]
struct Token {
    tok: Tok,
    first: char,
    column: usize,
}

fn lex(src: &str) -> Vec<Token> {
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let column = i + 1;
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c.is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            let text: String = chars[start..i].iter().collect();
            let tok = match text.parse::<u32>() {
                Ok(0) => Tok::Bad(SyntaxErrorKind::ZeroCoefficient),
                Ok(n) => Tok::Num(n),
                Err(_) => Tok::Bad(SyntaxErrorKind::NumberTooLarge),
            };
            out.push(Token {
                tok,
                first: c,
                column,
            });
            continue;
        }
        let tok = match c {
            'A'..='Z' => Tok::Var(c),
            'a'..='z' => Tok::Bad(SyntaxErrorKind::LowercaseVariable(c)),
            '+' => Tok::Plus,
            '*' | '×' | '·' => Tok::Times(c),
            '(' => Tok::LParen,
            ')' => Tok::RParen,
            _ => Tok::Bad(SyntaxErrorKind::UnexpectedChar(c)),
        };
        out.push(Token {
            tok,
            first: c,
            column,
        });
        i += 1;
    }
    out
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    /// Column just past the last character, for errors at the end of the text.
    end_column: usize,
    depth: usize,
}

impl Parser {
    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).copied()
    }

    fn error_at(&self, tok: Option<Token>, expected: Vec<Expected>) -> ExprError {
        let (kind, column) = match tok {
            None => (SyntaxErrorKind::UnexpectedEnd, self.end_column),
            Some(Token {
                tok: Tok::Bad(k),
                column,
                ..
            }) => (k, column),
            Some(t) => (SyntaxErrorKind::UnexpectedToken(t.first), t.column),
        };
        ExprError::Syntax(SyntaxError {
            kind,
            column,
            expected,
        })
    }

    /// What may follow a complete factor at the current nesting depth.
    fn after_operand(&self) -> Vec<Expected> {
        if self.depth > 0 {
            vec![Expected::Operator, Expected::CloseParen]
        } else {
            vec![Expected::Operator, Expected::End]
        }
    }

    fn expr(&mut self) -> Result<Polynomial, ExprError> {
        let mut acc = self.term()?;
        while let Some(Token { tok: Tok::Plus, .. }) = self.peek() {
            self.pos += 1;
            let rhs = self.term()?;
            acc = add(acc, rhs)?;
        }
        Ok(acc)
    }

    fn term(&mut self) -> Result<Polynomial, ExprError> {
        let mut acc = self.factor()?;
        loop {
            match self.peek().map(|t| t.tok) {
                Some(Tok::Times(_)) => {
                    self.pos += 1;
                    let rhs = self.factor()?;
                    acc = mul(acc, rhs)?;
                }
                // Implicit product: only a variable or '(' may follow by juxtaposition.
                Some(Tok::Var(_)) | Some(Tok::LParen) => {
                    let rhs = self.factor()?;
                    acc = mul(acc, rhs)?;
                }
                Some(Tok::Plus) | None => return Ok(acc),
                Some(Tok::RParen) if self.depth > 0 => return Ok(acc),
                _ => return Err(self.error_at(self.peek(), self.after_operand())),
            }
        }
    }

    fn factor(&mut self) -> Result<Polynomial, ExprError> {
        let tok = self.peek();
        match tok.map(|t| t.tok) {
            Some(Tok::Num(n)) => {
                self.pos += 1;
                Ok(constant(u64::from(n)))
            }
            Some(Tok::Var(v)) => {
                self.pos += 1;
                Ok(Polynomial {
                    terms: vec![Monomial {
                        coefficient: 1,
                        vars: vec![v],
                    }],
                })
            }
            Some(Tok::LParen) => {
                self.pos += 1;
                self.depth += 1;
                let inner = self.expr()?;
                match self.peek().map(|t| t.tok) {
                    Some(Tok::RParen) => {
                        self.pos += 1;
                        self.depth -= 1;
                        Ok(inner)
                    }
                    _ => Err(self.error_at(self.peek(), self.after_operand())),
                }
            }
            _ => Err(self.error_at(tok, vec![Expected::Operand])),
        }
    }
}

fn constant(n: u64) -> Polynomial {
    Polynomial {
        terms: vec![Monomial {
            coefficient: n,
            vars: Vec::new(),
        }],
    }
}

/// Adds `term` into `acc`, combining like terms and keeping first-appearance order.
fn push_term(acc: &mut Vec<Monomial>, term: Monomial) -> Result<(), ExprError> {
    if let Some(existing) = acc.iter_mut().find(|t| t.vars == term.vars) {
        existing.coefficient = existing
            .coefficient
            .checked_add(term.coefficient)
            .ok_or(ExprError::CoefficientTooLarge)?;
    } else {
        acc.push(term);
    }
    Ok(())
}

fn add(lhs: Polynomial, rhs: Polynomial) -> Result<Polynomial, ExprError> {
    let mut terms = lhs.terms;
    for t in rhs.terms {
        push_term(&mut terms, t)?;
    }
    Ok(Polynomial { terms })
}

fn mul(lhs: Polynomial, rhs: Polynomial) -> Result<Polynomial, ExprError> {
    let mut terms = Vec::new();
    for a in &lhs.terms {
        for b in &rhs.terms {
            if let Some(&var) = a.vars.iter().find(|v| b.vars.contains(v)) {
                return Err(ExprError::RepeatedVariable { var });
            }
            let mut vars: Vec<char> = a.vars.iter().chain(&b.vars).copied().collect();
            vars.sort_unstable();
            let coefficient = a
                .coefficient
                .checked_mul(b.coefficient)
                .ok_or(ExprError::CoefficientTooLarge)?;
            push_term(&mut terms, Monomial { coefficient, vars })?;
        }
    }
    Ok(Polynomial { terms })
}

pub(super) fn parse(src: &str) -> Result<Polynomial, ExprError> {
    let tokens = lex(src);
    if tokens.is_empty() {
        return Err(ExprError::Syntax(SyntaxError {
            kind: SyntaxErrorKind::Empty,
            column: 1,
            expected: vec![Expected::Operand],
        }));
    }
    let mut p = Parser {
        tokens,
        pos: 0,
        end_column: src.chars().count() + 1,
        depth: 0,
    };
    let poly = p.expr()?;
    if let Some(tok) = p.peek() {
        // Only a stray ')' at depth 0 can stop expr() before the end.
        return Err(p.error_at(Some(tok), vec![Expected::Operator, Expected::End]));
    }
    if poly.terms.iter().any(|t| t.vars.is_empty()) {
        return Err(ExprError::ConstantTerm);
    }
    Ok(poly)
}
