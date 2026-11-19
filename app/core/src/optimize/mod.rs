// Caminho relativo: app/core/src/optimize/mod.rs
//! The optimizer (spec 03, AJU-14; MT-46): circuits not dominated in area and delay, under
//! the costs in force (GER-12), for an expression or a height profile.
//!
//! The model is the one of the optimal circuit of a profile (AJU-12): points are told apart
//! only by column and readiness, and each level picks, per column, how many cells of each
//! kind it forms, the larger cells over the earliest points. The goal is the profile, or,
//! for an expression, the complete circuit, which may be closed by the CPA (CMP-11).
//!
//! The **exact search** runs the area up through the sums some count of cells adds up to:
//! at each area, the least delay below the delay already found at a smaller area, so each
//! answer is a new point of the front. It stops at the largest area among the heuristic's
//! circuits, or gives up past [`SEARCH_LIMIT`] levels. The **heuristic** is a family of
//! level-by-level rules (the targets of Dadda or of Wallace; an HA only when 1 is left to
//! remove, an FA whenever 3 points are free, or every FA the points allow; counters or not)
//! plus the solutions of Dadda, Dadda (Parhami) and Wallace themselves. The options shown are
//! the front of everything found, so none of them is dominated by those three methods.

use std::collections::HashMap;

use crate::help::{Method, dadda_target, solution_with_costs, wallace_target};
use crate::matrix::DotMatrix;
use crate::profile::{Profile, SEARCH_LIMIT};
use crate::reduce::{CellKind, Costs, PointId, Reduction};

/// Largest matrix searched exactly; above it only the heuristic runs, since the search
/// would only reach its limit after a long time.
pub const MAX_EXACT_DOTS: usize = 64;

/// How the options were found (AJU-14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// The front is exact up to the largest area of the heuristic's circuits.
    Exact,
    /// The search reached its limit: the front is exact up to this area (hundredths of
    /// `A_FA`), and above it the options are the heuristic's.
    ExactUpTo(u32),
    /// The matrix was too large for the exact search; the options are the heuristic's.
    Heuristic,
}

/// The options of the optimizer: complete circuits (or circuits at the profile) ascending in
/// area and strictly descending in delay.
#[derive(Debug, Clone)]
pub struct Options {
    pub mode: Mode,
    pub circuits: Vec<Reduction>,
}

/// Cell kinds in the order they take the earliest points of a column.
const KINDS: [CellKind; 4] = [CellKind::C154, CellKind::C73, CellKind::Fa, CellKind::Ha];

/// Readiness times (hundredths of `T_FA`) of the points of each column, ascending.
type State = Vec<Vec<u32>>;
/// Cells of each kind of [`KINDS`] per column on one level.
type Choice = Vec<[u32; 4]>;

/// A circuit in the model: its levels and whether the CPA closes it.
#[derive(Debug, Clone, Default)]
struct Plan {
    levels: Vec<Choice>,
    cpa: bool,
}

/// Where the reduction stops: no column above its target; an expression may close with the
/// CPA.
struct Goal {
    targets: Vec<u32>,
    cpa: bool,
}

impl Goal {
    fn reached(&self, state: &State) -> bool {
        state
            .iter()
            .zip(&self.targets)
            .all(|(col, &t)| col.len() as u32 <= t)
    }
}

/// The non-dominated circuits of `matrix`, at `profile` when given, under `costs`.
pub fn optimize(matrix: &DotMatrix, profile: Option<&Profile>, costs: Costs) -> Options {
    let width = matrix.output_width as usize;
    let goal = Goal {
        targets: (0..width)
            .map(|c| profile.map_or(1, |p| p.target(c)))
            .collect(),
        cpa: profile.is_none(),
    };
    let mut start: State = vec![Vec::new(); width];
    for (c, &h) in matrix.heights().iter().enumerate() {
        start[c] = vec![0; h as usize];
    }
    let kinds = useful_kinds(&costs);

    let mut heuristic: Vec<Reduction> = greedy_family(&start, &goal, &costs, kinds)
        .iter()
        .map(|plan| replay(matrix, plan, costs))
        .collect();
    heuristic.extend(
        [Method::Dadda, Method::DaddaParhami, Method::Wallace]
            .into_iter()
            .map(|m| solution_with_costs(matrix, m, costs)),
    );
    let cap = heuristic
        .iter()
        .map(|r| r.metrics().area)
        .max()
        .unwrap_or(0);

    let mut pool = Vec::new();
    let mut mode = Mode::Heuristic;
    if matrix.dots.len() <= MAX_EXACT_DOTS {
        let found = exact(&start, &goal, &costs, kinds, cap);
        pool.extend(found.plans.iter().map(|plan| replay(matrix, plan, costs)));
        mode = match found.up_to {
            None => Mode::Exact,
            Some(area) if !found.plans.is_empty() => Mode::ExactUpTo(area),
            Some(_) => Mode::Heuristic,
        };
    }
    pool.extend(heuristic);
    pool.retain(|r| match profile {
        Some(p) => p.reached(r),
        None => r.is_complete(),
    });
    Options {
        mode,
        circuits: front(pool),
    }
}

/// The kinds worth trying. A 7:3 costs, by default, what its network of 4 FA costs, and is
/// no faster than it (3 FA deep); likewise a 15:4 and its 11 FA, 5 deep (ADR 0005). Such a
/// counter is left out, which keeps the search small; a cheaper one is tried.
fn useful_kinds(costs: &Costs) -> [bool; 4] {
    let cheaper = |k: CellKind, fas: u32, depth: u32| {
        costs.area(k) < fas * costs.fa_area || costs.delay(k).0 < depth * costs.fa_delay
    };
    [
        cheaper(CellKind::C154, 11, 5),
        cheaper(CellKind::C73, 4, 3),
        true,
        true,
    ]
}

/// Keeps the circuits no other beats in both area and delay: ascending area, strictly
/// descending delay. At equal area and delay, the fewest levels, then the first found.
fn front(mut pool: Vec<Reduction>) -> Vec<Reduction> {
    let key = |r: &Reduction| {
        let m = r.metrics();
        (m.area, m.delay.0, r.levels().len())
    };
    pool.sort_by_key(key);
    let mut out: Vec<Reduction> = Vec::new();
    for r in pool {
        let d = r.metrics().delay;
        if out.last().is_none_or(|last| d < last.metrics().delay) {
            out.push(r);
        }
    }
    out
}

/// The next level: per column, the cells of each kind over the earliest points; outputs
/// beyond W are dropped (CMP-09).
fn step(state: &State, choice: &Choice, costs: &Costs) -> State {
    let width = state.len();
    let mut next: State = vec![Vec::new(); width];
    for (c, col) in state.iter().enumerate() {
        let mut i = 0;
        for (k, &kind) in KINDS.iter().enumerate() {
            for _ in 0..choice[c][k] {
                let size = kind.inputs();
                let ready =
                    col[i..i + size].iter().copied().max().unwrap_or(0) + costs.delay(kind).0;
                i += size;
                for bit in 0..kind.outputs() as usize {
                    if c + bit < width {
                        next[c + bit].push(ready);
                    }
                }
            }
        }
        next[c].extend_from_slice(&col[i..]);
    }
    for col in &mut next {
        col.sort_unstable();
    }
    next
}

/// Area and delay of closing `state` with the CPA (CMP-10, CMP-11), or `None` when it
/// cannot close it.
fn cpa(state: &State, costs: &Costs) -> Option<(u32, u32)> {
    if state.iter().any(|col| col.len() > 3) {
        return None;
    }
    let first = state.iter().position(|col| col.len() >= 2)?;
    if state
        .iter()
        .enumerate()
        .any(|(c, col)| col.len() == 3 && c != first)
    {
        return None;
    }
    let width = state.len();
    let mut delay = state[..first].iter().flatten().copied().max().unwrap_or(0);
    let (mut area, mut carry) = (0, None);
    for (c, col) in state.iter().enumerate().skip(first) {
        let mut inputs = col.clone();
        inputs.extend(carry.take());
        let kind = match inputs.len() {
            0 => continue,
            1 => {
                delay = delay.max(inputs[0]);
                continue;
            }
            2 => CellKind::Ha,
            _ => CellKind::Fa,
        };
        let ready = inputs.iter().copied().max().unwrap_or(0) + costs.delay(kind).0;
        area += costs.area(kind);
        delay = delay.max(ready);
        carry = (c + 1 < width).then_some(ready);
    }
    Some((area, delay))
}

/// Builds a plan as a reduction, through the rules: in each column the same cells over the
/// earliest points in canonical order, which are the earliest ready.
fn replay(matrix: &DotMatrix, plan: &Plan, costs: Costs) -> Reduction {
    let mut r = Reduction::with_costs(matrix, costs);
    for choice in &plan.levels {
        for (c, counts) in choice.iter().enumerate() {
            let free: Vec<PointId> = r
                .points()
                .iter()
                .filter(|p| p.column as usize == c)
                .map(|p| p.id)
                .collect();
            let mut i = 0;
            for (k, &kind) in KINDS.iter().enumerate() {
                for _ in 0..counts[k] {
                    let size = kind.inputs();
                    r.form_cell(&free[i..i + size])
                        .expect("a plan only forms valid cells");
                    i += size;
                }
            }
        }
        r.conclude_level()
            .expect("every level of a plan has a cell");
    }
    if plan.cpa {
        r.close_with_cpa().expect("a plan closes only when it can");
    }
    r
}

/// How a level rule spends the free points of a column over its target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Policy {
    /// Dadda's: an FA while 2 or more are left to remove, an HA when 1 is (AJU-05).
    Classic,
    /// Parhami's: an FA whenever 3 points are free, an HA when 2 are (AJU-13).
    EveryFa,
    /// Wallace's: every FA the points allow, in every column; an HA only if still over the
    /// target (AJU-10).
    AllFa,
}

/// The heuristic's level rules: the targets of Dadda or Wallace, each policy, with and
/// without the counters worth trying.
fn greedy_family(start: &State, goal: &Goal, costs: &Costs, kinds: [bool; 4]) -> Vec<Plan> {
    let counters = kinds[0] || kinds[1];
    let mut out = Vec::new();
    for targets in [dadda_target as fn(u32) -> Option<u32>, wallace_target] {
        for policy in [Policy::Classic, Policy::EveryFa, Policy::AllFa] {
            for with in [false, true] {
                if with && !counters {
                    continue;
                }
                let use_kinds = [kinds[0] && with, kinds[1] && with, true, true];
                out.extend(greedy(start, goal, costs, targets, policy, use_kinds));
            }
        }
    }
    out
}

/// One level rule followed until the goal: the profile, or the CPA once it can close.
fn greedy(
    start: &State,
    goal: &Goal,
    costs: &Costs,
    targets: fn(u32) -> Option<u32>,
    policy: Policy,
    kinds: [bool; 4],
) -> Option<Plan> {
    let width = start.len();
    let mut state = start.clone();
    let mut plan = Plan::default();
    // Every level forms a cell and lowers the weighted height; this only guards a hang.
    for _ in 0..4 * (width + 1) * (state.iter().map(Vec::len).sum::<usize>() + 1) {
        if goal.reached(&state) {
            return Some(plan);
        }
        let tallest = state.iter().map(Vec::len).max().unwrap_or(0) as u32;
        let Some(t) = targets(tallest) else {
            plan.cpa = cpa(&state, costs).is_some();
            return plan.cpa.then_some(plan);
        };
        let mut incoming = vec![0u32; width + 4];
        let mut choice: Choice = vec![[0; 4]; width];
        for c in 0..width {
            let target = t.max(goal.targets[c]);
            let mut free = state[c].len() as u32;
            let mut height = free + incoming[c];
            let mut form = |k: usize, choice: &mut Choice, free: &mut u32, height: &mut u32| {
                let kind = KINDS[k];
                choice[c][k] += 1;
                *free -= kind.inputs() as u32;
                *height -= kind.inputs() as u32 - 1;
                for bit in 1..kind.outputs() as usize {
                    incoming[c + bit] += 1;
                }
            };
            if policy == Policy::AllFa {
                while free >= 3 {
                    if kinds[1] && free >= 7 && height >= target + 6 {
                        form(1, &mut choice, &mut free, &mut height);
                    } else {
                        form(2, &mut choice, &mut free, &mut height);
                    }
                }
            }
            while height > target {
                let over = height - target;
                let k = if kinds[0] && free >= 15 && over >= 14 {
                    0
                } else if kinds[1] && free >= 7 && over >= 6 {
                    1
                } else if free >= 3 && (over >= 2 || policy != Policy::Classic) {
                    2
                } else if free >= 2 {
                    3
                } else {
                    break;
                };
                form(k, &mut choice, &mut free, &mut height);
            }
        }
        if choice.iter().all(|c| c.iter().all(|&n| n == 0)) {
            return None;
        }
        state = step(&state, &choice, costs);
        plan.levels.push(choice);
    }
    None
}

/// What the exact search found: the plans of new points of the front, ascending in area,
/// and, when it stopped at the limit, the largest area it finished.
struct Found {
    plans: Vec<Plan>,
    up_to: Option<u32>,
}

/// The exact front up to `cap` area: at each area, the least delay below the delay of the
/// option found at a smaller area.
fn exact(start: &State, goal: &Goal, costs: &Costs, kinds: [bool; 4], cap: u32) -> Found {
    // Per kind tried: how much a cell lowers its own column (inputs - 1) and its area per
    // unit lowered, times 84 (= 2 x 6 x 7, so every ratio is whole).
    let lowering = |k: CellKind| k.inputs() as u64 - 1;
    let tried: Vec<CellKind> = KINDS
        .iter()
        .zip(kinds)
        .filter(|&(_, on)| on)
        .map(|(&k, _)| k)
        .collect();
    let per_unit = tried
        .iter()
        .map(|&k| 84 * u64::from(costs.area(k)) / lowering(k))
        .min()
        .expect("FA and HA are always tried");
    let most = tried.iter().map(|&k| lowering(k)).max().unwrap_or(2);
    let areas = reachable_areas(costs, kinds, cap);
    let mut search = Search {
        costs,
        goal,
        kinds,
        per_unit,
        most,
        tried: 0,
        seen: HashMap::new(),
        best: None,
        path: Vec::new(),
        budget: 0,
        bound: u32::MAX,
    };
    let mut plans = Vec::new();
    let mut found_delay = u32::MAX;
    let mut done = None;
    for budget in areas {
        search.bound = found_delay;
        if search.bound == 0 {
            break;
        }
        search.budget = budget;
        search.seen.clear();
        if search.dfs(start, 0).is_err() {
            return Found {
                plans,
                up_to: Some(done.unwrap_or(0)),
            };
        }
        done = Some(budget);
        if let Some(((delay, _), plan)) = search.best.take() {
            found_delay = delay;
            plans.push(plan);
        }
    }
    Found { plans, up_to: None }
}

/// Every area up to `cap` that some count of the kinds tried adds up to, ascending.
fn reachable_areas(costs: &Costs, kinds: [bool; 4], cap: u32) -> Vec<u32> {
    let cap = cap as usize;
    let mut can = vec![false; cap + 1];
    can[0] = true;
    let sizes: Vec<usize> = KINDS
        .iter()
        .zip(kinds)
        .filter(|&(_, on)| on)
        .map(|(&k, _)| costs.area(k) as usize)
        .collect();
    for a in 1..=cap {
        can[a] = sizes.iter().any(|&s| s <= a && can[a - s]);
    }
    (0..=cap).filter(|&a| can[a]).map(|a| a as u32).collect()
}

struct Search<'a> {
    costs: &'a Costs,
    goal: &'a Goal,
    kinds: [bool; 4],
    /// Least area per unit a cell lowers its column, times 84.
    per_unit: u64,
    /// Most a cell lowers its column: 2 for an FA, 6 for a 7:3, 14 for a 15:4.
    most: u64,
    /// Levels tried so far, over all rounds.
    tried: usize,
    /// Least area, and then fewest levels, at which each state was searched in this round.
    seen: HashMap<State, (u32, usize)>,
    /// (delay, levels) and the plan of the best circuit of exactly `budget` area.
    best: Option<((u32, usize), Plan)>,
    path: Vec<Choice>,
    budget: u32,
    /// Delay of the option found at a smaller area: only a faster circuit is new.
    bound: u32,
}

/// The search went past [`SEARCH_LIMIT`] levels.
struct TooLarge;

impl Search<'_> {
    fn record(&mut self, key: (u32, usize), cpa: bool) {
        if key.0 < self.bound && self.best.as_ref().is_none_or(|(b, _)| key < *b) {
            self.best = Some((
                key,
                Plan {
                    levels: self.path.clone(),
                    cpa,
                },
            ));
        }
    }

    /// Least area, times 84, still needed to reach the goal from `state`. A cell keeps one
    /// output in its column, so its column goes down by its inputs - 1; and every cell sends
    /// one output to the column above. So, from the lowest column up, a column holding `n`
    /// points plus the carries of the fewest cells the column below needs must go down by
    /// `n + carries - target`.
    fn still_needed(&self, state: &State) -> u64 {
        let mut carries = 0;
        let mut units = 0;
        for (col, &t) in state.iter().zip(&self.goal.targets) {
            let over = (col.len() as u64 + carries).saturating_sub(u64::from(t));
            units += over;
            carries = over.div_ceil(self.most);
        }
        units * self.per_unit
    }

    fn dfs(&mut self, state: &State, area: u32) -> Result<(), TooLarge> {
        let delay = state.iter().flatten().copied().max().unwrap_or(0);
        let depth = self.path.len();
        let key = (delay, depth);
        if delay >= self.bound || self.best.as_ref().is_some_and(|(b, _)| *b <= key) {
            return Ok(());
        }
        if self.goal.reached(state) {
            // Earlier rounds found nothing as fast and cheaper, so this area is the budget.
            if area == self.budget {
                self.record(key, false);
            }
            return Ok(());
        }
        if 84 * u64::from(area) + self.still_needed(state) > 84 * u64::from(self.budget) {
            return Ok(());
        }
        if self
            .seen
            .get(state)
            .is_some_and(|&(a, d)| a <= area && d <= depth)
        {
            return Ok(());
        }
        self.seen.insert(state.clone(), (area, depth));
        if self.goal.cpa
            && let Some((a, d)) = cpa(state, self.costs)
            && area + a == self.budget
        {
            self.record((d, depth + 1), true);
        }

        // Per column, every count of cells that fits: without counters first, then more
        // FA first.
        let options: Vec<Vec<[u32; 4]>> = state
            .iter()
            .map(|col| column_options(col.len() as u32, self.kinds))
            .collect();
        let mut choice: Choice = vec![[0; 4]; state.len()];
        self.each(state, &options, 0, area, 0, &mut choice)
    }

    /// Walks the choices column by column, then searches the next level of each.
    fn each(
        &mut self,
        state: &State,
        options: &[Vec<[u32; 4]>],
        column: usize,
        area: u32,
        added: u32,
        choice: &mut Choice,
    ) -> Result<(), TooLarge> {
        if column == options.len() {
            if added == 0 {
                return Ok(()); // a level needs at least one cell
            }
            self.tried += 1;
            if self.tried > SEARCH_LIMIT {
                return Err(TooLarge);
            }
            let next = step(state, choice, self.costs);
            self.path.push(choice.clone());
            let out = self.dfs(&next, area + added);
            self.path.pop();
            return out;
        }
        for counts in &options[column] {
            let cost: u32 = KINDS
                .iter()
                .zip(counts)
                .map(|(&k, &n)| n * self.costs.area(k))
                .sum();
            if area + added + cost > self.budget {
                continue;
            }
            choice[column] = *counts;
            self.each(state, options, column + 1, area, added + cost, choice)?;
        }
        choice[column] = [0; 4];
        Ok(())
    }
}

/// Every count of 15:4, 7:3, FA and HA that `n` points allow, among the kinds tried.
fn column_options(n: u32, kinds: [bool; 4]) -> Vec<[u32; 4]> {
    let mut out = Vec::new();
    let most = |size: u32, on: bool, left: u32| if on { left / size } else { 0 };
    for a in 0..=most(15, kinds[0], n) {
        for b in 0..=most(7, kinds[1], n - 15 * a) {
            let left = n - 15 * a - 7 * b;
            for f in (0..=left / 3).rev() {
                for h in (0..=(left - 3 * f) / 2).rev() {
                    out.push([a, b, f, h]);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::analyze;

    #[test]
    fn the_cpa_closes_only_what_cmp10_lets_it() {
        let c = Costs::default();
        // Column 0: FA (ready at 1); column 1: its 2 points and the carry, FA (2); column 2:
        // the carry alone, a wire.
        assert_eq!(
            cpa(&vec![vec![0; 3], vec![0; 2], vec![]], &c),
            Some((200, 200))
        );
        assert_eq!(cpa(&vec![vec![0; 4], vec![]], &c), None, "a column of 4");
        assert_eq!(
            cpa(&vec![vec![0; 2], vec![0; 3]], &c),
            None,
            "3 above the first"
        );
        assert_eq!(cpa(&vec![vec![0], vec![0]], &c), None, "nothing to add");
    }

    /// The level rules of the heuristic with the targets and the policy of a method give
    /// that method's area and delay on R1 (AJU-05, AJU-10, AJU-13).
    #[test]
    fn the_family_holds_the_three_methods() {
        let w = [
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
        let goal = Goal {
            targets: vec![1; 7],
            cpa: true,
        };
        let start: State = m.heights().iter().map(|&h| vec![0; h as usize]).collect();
        let c = Costs::default();
        let kinds = [false, false, true, true];
        for (targets, policy, method) in [
            (
                dadda_target as fn(u32) -> Option<u32>,
                Policy::Classic,
                Method::Dadda,
            ),
            (dadda_target, Policy::EveryFa, Method::DaddaParhami),
            (wallace_target, Policy::AllFa, Method::Wallace),
        ] {
            let plan = greedy(&start, &goal, &c, targets, policy, kinds).unwrap();
            let ours = replay(&m, &plan, c).metrics();
            let theirs = solution_with_costs(&m, method, c).metrics();
            assert_eq!(
                (ours.area, ours.delay, ours.fa, ours.ha),
                (theirs.area, theirs.delay, theirs.fa, theirs.ha),
                "{method:?}"
            );
        }
    }
}
