// Caminho relativo: app/core/src/verify/mod.rs
//! Verification by simulation (spec 04; MT-7).
//!
//! Simulates the netlist 64 input vectors at a time (one per bit of a `u64`) and compares
//! with the expression. Up to `EXHAUSTIVE_LIMIT_BITS` input bits every combination is
//! checked; above that, a fixed sample. Work is done in chunks so the interface can show
//! progress and cancel (VER-04).

use std::collections::BTreeMap;

use crate::netlist::{Netlist, Wire};
use crate::reduce::CellKind;

/// Up to this many input bits, verification is exhaustive (spec 04).
pub const EXHAUSTIVE_LIMIT_BITS: u32 = 24;
/// Number of vectors checked above the exhaustive limit.
pub const SAMPLE_SIZE: u64 = 1 << 20;
/// Seed of the xorshift64* sample generator.
pub const SAMPLE_SEED: u64 = 0x00D0_7EE5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coverage {
    Exhaustive,
    Sample,
}

/// A failing input vector (VER-03).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Counterexample {
    pub inputs: BTreeMap<char, u64>,
    pub expected: u128,
    pub obtained: u128,
}

impl Counterexample {
    /// Bits where the circuit disagrees with the expression.
    pub fn differing_bits(&self) -> u128 {
        self.expected ^ self.obtained
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Progress {
    pub checked: u64,
    pub total: u64,
    pub coverage: Coverage,
    /// The first failing vector, if any; verification stops there.
    pub counterexample: Option<Counterexample>,
}

impl Progress {
    pub fn done(&self) -> bool {
        self.checked == self.total || self.counterexample.is_some()
    }
}

/// The sample of spec 04: all zeros, all ones, then xorshift64* from `SAMPLE_SEED`, each
/// masked to `bits`. The VHDL testbench uses the first 4096 of the same sequence.
pub fn sample_vectors(bits: u32, count: usize) -> Vec<u64> {
    let mut s = Sampler::new(bits);
    (0..count).map(|_| s.next_vector()).collect()
}

/// Generator of the sample sequence, one vector at a time.
#[derive(Debug, Clone)]
struct Sampler {
    mask: u64,
    state: u64,
    emitted: u64,
}

impl Sampler {
    fn new(bits: u32) -> Sampler {
        let mask = if bits >= 64 {
            u64::MAX
        } else {
            (1u64 << bits) - 1
        };
        Sampler {
            mask,
            state: SAMPLE_SEED,
            emitted: 0,
        }
    }

    fn next_vector(&mut self) -> u64 {
        self.emitted += 1;
        match self.emitted {
            1 => 0,
            2 => self.mask,
            _ => {
                // xorshift64* (Vigna, 2016)
                let mut x = self.state;
                x ^= x >> 12;
                x ^= x << 25;
                x ^= x >> 27;
                self.state = x;
                x.wrapping_mul(0x2545_F491_4F6C_DD1D) & self.mask
            }
        }
    }
}

/// A term of the expression prepared for fast evaluation: coefficient and the
/// (offset, mask) of each variable in a packed vector.
#[derive(Debug, Clone)]
struct FastTerm {
    coefficient: u128,
    vars: Vec<(u32, u64)>,
}

/// An interruptible verification run.
#[derive(Debug, Clone)]
pub struct Verifier {
    netlist: Netlist,
    /// Packed-vector bit index of each factor of each input signal.
    signal_bits: Vec<Vec<u32>>,
    terms: Vec<FastTerm>,
    bits: u32,
    total: u64,
    coverage: Coverage,
    checked: u64,
    sampler: Sampler,
    counterexample: Option<Counterexample>,
}

impl Verifier {
    pub fn new(netlist: Netlist) -> Verifier {
        let bits = netlist.input_bits();
        let offsets = netlist.offsets();
        let signal_bits = netlist
            .signals
            .iter()
            .map(|s| s.factors.iter().map(|f| offsets[&f.var] + f.bit).collect())
            .collect();
        let widths: BTreeMap<char, u32> = netlist.variables.iter().copied().collect();
        let terms = netlist
            .terms
            .iter()
            .map(|t| FastTerm {
                coefficient: u128::from(t.coefficient),
                vars: t
                    .vars
                    .iter()
                    .map(|v| (offsets[v], (1u64 << widths[v]) - 1))
                    .collect(),
            })
            .collect();
        let (coverage, total) = if bits <= EXHAUSTIVE_LIMIT_BITS {
            (Coverage::Exhaustive, 1u64 << bits)
        } else {
            (Coverage::Sample, SAMPLE_SIZE)
        };
        Verifier {
            netlist,
            signal_bits,
            terms,
            bits,
            total,
            coverage,
            checked: 0,
            sampler: Sampler::new(bits),
            counterexample: None,
        }
    }

    pub fn progress(&self) -> Progress {
        Progress {
            checked: self.checked,
            total: self.total,
            coverage: self.coverage,
            counterexample: self.counterexample.clone(),
        }
    }

    /// Checks up to `max_vectors` more vectors (rounded up to a batch of 64).
    pub fn run(&mut self, max_vectors: u64) -> Progress {
        let stop = self.checked.saturating_add(max_vectors.max(1));
        while self.checked < self.total && self.checked < stop && self.counterexample.is_none() {
            let lanes = (self.total - self.checked).min(64) as usize;
            let vectors: Vec<u64> = match self.coverage {
                Coverage::Exhaustive => (0..lanes as u64).map(|i| self.checked + i).collect(),
                Coverage::Sample => (0..lanes).map(|_| self.sampler.next_vector()).collect(),
            };
            match self.check_batch(&vectors) {
                None => self.checked += lanes as u64,
                Some((lane, ce)) => {
                    self.checked += lane as u64 + 1;
                    self.counterexample = Some(ce);
                }
            }
        }
        self.progress()
    }

    /// Runs until the end or the first counterexample.
    pub fn run_to_end(&mut self) -> Progress {
        self.run(u64::MAX)
    }

    fn expected(&self, packed: u64) -> u128 {
        self.terms
            .iter()
            .map(|t| {
                t.vars.iter().fold(t.coefficient, |acc, &(off, mask)| {
                    acc * u128::from((packed >> off) & mask)
                })
            })
            .sum()
    }

    /// Simulates up to 64 vectors at once, one per bit of each `u64`; returns the first
    /// failing lane.
    fn check_batch(&self, vectors: &[u64]) -> Option<(usize, Counterexample)> {
        // Transpose: bit b of input vector `lane` becomes bit `lane` of masks[b].
        let mut masks = vec![0u64; self.bits as usize];
        for (lane, &v) in vectors.iter().enumerate() {
            for (b, m) in masks.iter_mut().enumerate() {
                *m |= ((v >> b) & 1) << lane;
            }
        }
        let inputs: Vec<u64> = self
            .signal_bits
            .iter()
            .map(|bits| {
                bits.iter()
                    .fold(u64::MAX, |acc, &b| acc & masks[b as usize])
            })
            .collect();

        let n = &self.netlist;
        let mut sums = vec![0u64; n.cells.len() + 1];
        let mut carries = vec![0u64; n.cells.len() + 1];
        let value = |w: Wire, sums: &[u64], carries: &[u64]| match w {
            Wire::Input(i) => inputs[i],
            Wire::Sum(c) => sums[c],
            Wire::Carry(c) => carries[c],
        };
        for cell in &n.cells {
            let x: Vec<u64> = cell
                .inputs
                .iter()
                .map(|&w| value(w, &sums, &carries))
                .collect();
            let (s, k) = match cell.kind {
                CellKind::Fa => (
                    x[0] ^ x[1] ^ x[2],
                    (x[0] & x[1]) | (x[0] & x[2]) | (x[1] & x[2]),
                ),
                CellKind::Ha => (x[0] ^ x[1], x[0] & x[1]),
            };
            sums[cell.number] = s;
            carries[cell.number] = k;
        }
        let result: Vec<u64> = n
            .result
            .iter()
            .map(|w| w.map_or(0, |w| value(w, &sums, &carries)))
            .collect();

        vectors.iter().enumerate().find_map(|(lane, &v)| {
            let obtained = result.iter().enumerate().fold(0u128, |acc, (j, &m)| {
                acc | (u128::from((m >> lane) & 1) << j)
            });
            let expected = self.expected(v);
            (obtained != expected).then(|| {
                (
                    lane,
                    Counterexample {
                        inputs: n.unpack(v),
                        expected,
                        obtained,
                    },
                )
            })
        })
    }
}
