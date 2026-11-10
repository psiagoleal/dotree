// Caminho relativo: app/core/src/profile/mod.rs
//! Input by column heights with a target output profile (MT-31a; spec 01 EXP-16 and EXP-17,
//! spec 02 CMP-19, spec 03 AJU-12).
//!
//! `{2,2,3; 1,1,1,1}` gives the heights of the input columns and of the output columns,
//! from the most significant to the least: the notation of generalized parallel counters
//! (Parhami, *Computer Arithmetic*, ch. 8), with the output written as heights too. Each
//! dot is an independent 1-bit variable, so the profile is an ordinary expression
//! (`A + B + C + 2D + 2E + 4F + 4G`) plus a place to stop: the reduction reaches the profile
//! when no column is taller than its target.

use std::collections::{BTreeMap, HashMap};

use crate::expr::{Analysis, ExprError, analyze};
use crate::matrix::DotMatrix;
use crate::reduce::{CellKind, PointId, Reduction};

/// One variable per dot, named `A` to `Z`.
pub const MAX_DOTS: u32 = 26;
/// Longest list of heights, input or output.
pub const MAX_COLUMNS: usize = 16;
/// Levels the optimal search may try before it gives up (AJU-12).
pub const SEARCH_LIMIT: usize = 1_000_000;

/// Heights per column, least significant first (index = column).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    pub input: Vec<u32>,
    pub output: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileError {
    /// Something other than `{h, …, h; t, …, t}`, at this 1-based character column.
    Syntax { column: usize },
    /// Every input height is 0.
    NoDots,
    /// More dots than variables `A` to `Z`.
    TooManyDots { dots: u32, limit: u32 },
    /// A list with more than `MAX_COLUMNS` heights.
    TooManyColumns { limit: usize },
    /// The output cannot hold the input from this column up: points never move to a lower
    /// column, so the value of the input from column `c` up must fit in the output from `c`
    /// up. The highest such column.
    OutputTooSmall { column: u32 },
    /// The output asks for an empty column that will always hold a point: a column with a
    /// point never empties, because every cell leaves its sum in it. The lowest such column.
    EmptyColumn { column: u32 },
}

/// Why the optimal search has no circuit to give (AJU-12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchError {
    /// No circuit of FA and HA reaches the profile, within an area bound well above any
    /// useful circuit.
    NoCircuit,
    /// The search tried more than `SEARCH_LIMIT` levels.
    TooLarge,
}

/// The known block a profile describes (AJU-12), to name on request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block {
    /// `{3; 1,1}`.
    FullAdder,
    /// `{2; 1,1}`.
    HalfAdder,
    /// `{2,…,2; 1,…,1}`: a `bits`-bit adder.
    Adder { bits: u32 },
    /// `{2,…,2,3; 1,…,1}`: a `bits`-bit adder with carry in.
    AdderWithCarryIn { bits: u32 },
    /// `{3,…,3; 1,2,…,2,1}`: `bits` full adders side by side (carry-save adder).
    CarrySave { bits: u32 },
    /// Output with one bit per column: a (generalized) parallel counter.
    Counter,
    /// Output with some column of 2 or more: a compressor to a redundant form.
    Compressor,
}

/// What the user typed, analysed: an expression, or a height profile and its equivalent
/// expression (EXP-16).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub analysis: Analysis,
    pub profile: Option<Profile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputError {
    Expr(ExprError),
    Profile(ProfileError),
}

/// Reads `src` as a height profile when it starts with `{`, or else as an expression with
/// `widths`. A profile ignores the widths: its variables are its own, all of 1 bit.
pub fn analyze_input(src: &str, widths: &BTreeMap<char, u32>) -> Result<Input, InputError> {
    match parse(src).map_err(InputError::Profile)? {
        Some(profile) => {
            let (expression, own) = profile.expression();
            let analysis = analyze(&expression, &own).expect("a parsed profile is valid");
            Ok(Input {
                analysis,
                profile: Some(profile),
            })
        }
        None => Ok(Input {
            analysis: analyze(src, widths).map_err(InputError::Expr)?,
            profile: None,
        }),
    }
}

/// The profile in `src`, or `None` when `src` is an expression (it does not start with `{`).
pub fn parse(src: &str) -> Result<Option<Profile>, ProfileError> {
    let chars: Vec<char> = src.chars().collect();
    let mut at = 0;
    let skip = |at: &mut usize| {
        while *at < chars.len() && chars[*at].is_whitespace() {
            *at += 1;
        }
    };
    skip(&mut at);
    if chars.get(at) != Some(&'{') {
        return Ok(None);
    }
    at += 1;
    let input = list(&chars, &mut at, ';')?;
    let output = list(&chars, &mut at, '}')?;
    skip(&mut at);
    if at < chars.len() {
        return Err(ProfileError::Syntax { column: at + 1 });
    }

    let mut input: Vec<u32> = input.into_iter().rev().collect();
    let output: Vec<u32> = output.into_iter().rev().collect();
    let dots: u32 = input.iter().fold(0, |a, &h| a.saturating_add(h));
    if dots == 0 {
        return Err(ProfileError::NoDots);
    }
    if dots > MAX_DOTS {
        return Err(ProfileError::TooManyDots {
            dots,
            limit: MAX_DOTS,
        });
    }
    while input.last() == Some(&0) {
        input.pop();
    }
    // From the top down, the value of the input from column c up must fit in the output
    // from c up (both at most 16 columns, so u128 holds every sum).
    let weight = |hs: &[u32], from: usize| -> u128 {
        hs.iter()
            .enumerate()
            .skip(from)
            .map(|(c, &h)| u128::from(h) << c)
            .sum()
    };
    for column in (0..input.len()).rev() {
        if weight(&input, column) > weight(&output, column) {
            return Err(ProfileError::OutputTooSmall {
                column: column as u32,
            });
        }
    }
    let profile = Profile { input, output };
    if let Some(column) = profile.always_occupied().find(|&c| profile.target(c) == 0) {
        return Err(ProfileError::EmptyColumn {
            column: column as u32,
        });
    }
    Ok(Some(profile))
}

/// Heights separated by commas up to `end`, which it consumes. At least one height.
fn list(chars: &[char], at: &mut usize, end: char) -> Result<Vec<u32>, ProfileError> {
    let mut out = Vec::new();
    loop {
        while *at < chars.len() && chars[*at].is_whitespace() {
            *at += 1;
        }
        let start = *at;
        let mut value: u32 = 0;
        while let Some(d) = chars.get(*at).and_then(|c| c.to_digit(10)) {
            value = value.saturating_mul(10).saturating_add(d);
            *at += 1;
        }
        if *at == start {
            return Err(ProfileError::Syntax { column: *at + 1 });
        }
        out.push(value);
        if out.len() > MAX_COLUMNS {
            return Err(ProfileError::TooManyColumns { limit: MAX_COLUMNS });
        }
        while *at < chars.len() && chars[*at].is_whitespace() {
            *at += 1;
        }
        match chars.get(*at) {
            Some(',') => *at += 1,
            Some(&c) if c == end => {
                *at += 1;
                return Ok(out);
            }
            _ => return Err(ProfileError::Syntax { column: *at + 1 }),
        }
    }
}

impl Profile {
    /// The equivalent expression, one 1-bit variable per dot from `A`, column by column
    /// from the least significant, and its widths.
    pub fn expression(&self) -> (String, BTreeMap<char, u32>) {
        let mut terms = Vec::new();
        let mut widths = BTreeMap::new();
        let mut var = b'A';
        for (column, &h) in self.input.iter().enumerate() {
            for _ in 0..h {
                let v = char::from(var);
                var += 1;
                widths.insert(v, 1);
                terms.push(if column == 0 {
                    v.to_string()
                } else {
                    format!("{}{v}", 1u32 << column)
                });
            }
        }
        (terms.join(" + "), widths)
    }

    /// The dot matrix of the input.
    pub fn matrix(&self) -> DotMatrix {
        let (src, widths) = self.expression();
        let analysis = analyze(&src, &widths).expect("a parsed profile is a valid expression");
        DotMatrix::build(&analysis).expect("at most 26 dots")
    }

    /// Target height of `column`; 0 beyond the listed columns.
    pub fn target(&self, column: usize) -> u32 {
        self.output.get(column).copied().unwrap_or(0)
    }

    /// Columns below W that hold a point in every circuit. At least `n` points enter a
    /// column: its dots and the carries of the cells the column below needs. A cell lowers
    /// its column by at most 2 (an FA: 3 in, 1 out), so a column entered by `n` points with
    /// target `t` needs at least `ceil((n - t) / 2)` cells, each with a carry up.
    fn always_occupied(&self) -> impl Iterator<Item = usize> + '_ {
        let width = self.matrix().output_width as usize;
        let mut carries = 0;
        (0..width).filter(move |&c| {
            let n = self.input.get(c).copied().unwrap_or(0) + carries;
            carries = n.saturating_sub(self.target(c)).div_ceil(2);
            n > 0
        })
    }

    /// No cell pending and no column above its target (CMP-19).
    pub fn reached(&self, r: &Reduction) -> bool {
        r.pending().is_empty()
            && r.heights()
                .iter()
                .enumerate()
                .all(|(c, &h)| h <= self.target(c))
    }

    /// The circuit of FA and HA that reaches the profile with the least area and, among
    /// those, the least delay, then the fewest levels (AJU-12).
    ///
    /// Iterative deepening on the area: each level picks, per column, how many FA and HA it
    /// forms, always over the earliest points of the column. Points are told apart only by
    /// column and readiness, so equal states found by different paths are searched once.
    pub fn optimal(&self) -> Result<Reduction, SearchError> {
        let matrix = self.matrix();
        let width = matrix.output_width as usize;
        let mut start: State = vec![Vec::new(); width];
        for (c, &h) in matrix.heights().iter().enumerate() {
            start[c] = vec![0; h as usize];
        }
        let targets: Vec<u32> = (0..width).map(|c| self.target(c)).collect();
        // Every useful circuit is far below this: each FA lowers the number of points by 1
        // and each HA moves one up a column.
        let bound = 4 * matrix.dots.len() * (width + 1) + 4;
        let mut search = Search {
            targets: &targets,
            room: targets.iter().map(|&t| t as usize).sum(),
            tried: 0,
            seen: HashMap::new(),
            best: None,
            path: Vec::new(),
            budget: 0,
        };
        for budget in 0..=bound {
            search.budget = budget;
            search.seen.clear();
            search.dfs(&start, 0)?;
            if let Some((_, plan)) = search.best.take() {
                return Ok(replay(&matrix, &plan));
            }
        }
        Err(SearchError::NoCircuit)
    }

    /// The known block this profile describes (AJU-12).
    pub fn block(&self) -> Block {
        let input = &self.input;
        let width = self.matrix().output_width as usize;
        let mut output: Vec<u32> = (0..width).map(|c| self.target(c)).collect();
        while output.last() == Some(&0) {
            output.pop();
        }
        let n = input.len();
        let ones = output.iter().all(|&t| t == 1);
        if input == &[3] && output == [1, 1] {
            Block::FullAdder
        } else if input == &[2] && output == [1, 1] {
            Block::HalfAdder
        } else if input.iter().all(|&h| h == 3)
            && output.len() == n + 1
            && output[0] == 1
            && output[n] == 1
            && output[1..n].iter().all(|&t| t == 2)
        {
            Block::CarrySave { bits: n as u32 }
        } else if ones && output.len() == n + 1 && input[1..].iter().all(|&h| h == 2) {
            match input[0] {
                2 => Block::Adder { bits: n as u32 },
                3 => Block::AdderWithCarryIn { bits: n as u32 },
                _ => Block::Counter,
            }
        } else if ones {
            Block::Counter
        } else {
            Block::Compressor
        }
    }
}

/// Readiness times (half `T_FA`) of the points of each column, ascending.
type State = Vec<Vec<u32>>;
/// FA and HA per column on one level.
type Choice = Vec<(u32, u32)>;

struct Search<'a> {
    targets: &'a [u32],
    /// Sum of the targets: points the profile can hold.
    room: usize,
    /// Levels tried so far, over all rounds.
    tried: usize,
    /// Least area, and then fewest levels, at which each state was searched in this round.
    seen: HashMap<State, (usize, usize)>,
    /// (delay, levels) and the levels of the best circuit of exactly `budget` area.
    best: Option<((u32, usize), Vec<Choice>)>,
    path: Vec<Choice>,
    budget: usize,
}

impl Search<'_> {
    fn dfs(&mut self, state: &State, area: usize) -> Result<(), SearchError> {
        let delay = state.iter().flatten().copied().max().unwrap_or(0);
        let key = (delay, self.path.len());
        if self.best.as_ref().is_some_and(|(b, _)| *b <= key) {
            return Ok(());
        }
        let reached = state
            .iter()
            .enumerate()
            .all(|(c, col)| col.len() as u32 <= self.targets[c]);
        if reached {
            // Earlier rounds found nothing cheaper, so this area is exactly the budget.
            if area == self.budget {
                self.best = Some((key, self.path.clone()));
            }
            return Ok(());
        }
        // Each half of area removes at most one point: an FA removes 1 for 2 halves, or 2 in
        // the top column, whose carry is dropped (CMP-09); an HA removes 1 only there.
        let points: usize = state.iter().map(Vec::len).sum();
        if area + points.saturating_sub(self.room) > self.budget {
            return Ok(());
        }
        let depth = self.path.len();
        if self
            .seen
            .get(state)
            .is_some_and(|&(a, d)| a <= area && d <= depth)
        {
            return Ok(());
        }
        self.seen.insert(state.clone(), (area, depth));

        // Per column, every (FA, HA) that fits, more FA first.
        let options: Vec<Vec<(u32, u32)>> = state
            .iter()
            .map(|col| {
                let n = col.len() as u32;
                (0..=n / 3)
                    .rev()
                    .flat_map(|f| (0..=(n - 3 * f) / 2).rev().map(move |h| (f, h)))
                    .collect()
            })
            .collect();
        let mut choice: Choice = vec![(0, 0); state.len()];
        self.each(state, &options, 0, area, 0, &mut choice)
    }

    /// Walks the choices column by column, then searches the next level of each.
    fn each(
        &mut self,
        state: &State,
        options: &[Vec<(u32, u32)>],
        column: usize,
        area: usize,
        added: usize,
        choice: &mut Choice,
    ) -> Result<(), SearchError> {
        if column == options.len() {
            if added == 0 {
                return Ok(()); // a level needs at least one cell
            }
            self.tried += 1;
            if self.tried > SEARCH_LIMIT {
                return Err(SearchError::TooLarge);
            }
            let next = step(state, choice);
            self.path.push(choice.clone());
            let out = self.dfs(&next, area + added);
            self.path.pop();
            return out;
        }
        for &(f, h) in &options[column] {
            let cost = (2 * f + h) as usize;
            if area + added + cost > self.budget {
                continue;
            }
            choice[column] = (f, h);
            self.each(state, options, column + 1, area, added + cost, choice)?;
        }
        choice[column] = (0, 0);
        Ok(())
    }
}

/// The next level: FA first, then HA, over the earliest points of each column.
fn step(state: &State, choice: &Choice) -> State {
    let width = state.len();
    let mut next: State = vec![Vec::new(); width];
    for (c, col) in state.iter().enumerate() {
        let (f, h) = choice[c];
        let mut i = 0;
        let groups = (0..f).map(|_| (3, 2)).chain((0..h).map(|_| (2, 1)));
        for (size, delay) in groups {
            let ready = col[i..i + size].iter().copied().max().unwrap_or(0) + delay;
            i += size;
            next[c].push(ready);
            if c + 1 < width {
                next[c + 1].push(ready); // a carry beyond W is dropped (CMP-09)
            }
        }
        next[c].extend_from_slice(&col[i..]);
    }
    for col in &mut next {
        col.sort_unstable();
    }
    next
}

/// Builds the plan as a reduction: in each column, the same cells over the earliest points
/// in canonical order, which are the earliest ready.
fn replay(matrix: &DotMatrix, plan: &[Choice]) -> Reduction {
    let mut r = Reduction::new(matrix);
    for choice in plan {
        for (c, &(f, h)) in choice.iter().enumerate() {
            let free: Vec<PointId> = r
                .points()
                .iter()
                .filter(|p| p.column as usize == c)
                .map(|p| p.id)
                .collect();
            let mut i = 0;
            let groups = (0..f)
                .map(|_| CellKind::Fa)
                .chain((0..h).map(|_| CellKind::Ha));
            for kind in groups {
                let size = if kind == CellKind::Fa { 3 } else { 2 };
                r.form_cell(&free[i..i + size])
                    .expect("the plan only forms valid cells");
                i += size;
            }
        }
        r.conclude_level()
            .expect("every level of the plan has a cell");
    }
    r
}
