// Caminho relativo: app/core/src/vhdl/mod.rs
//! Structural VHDL-93 template of the circuit (spec 06; MT-10).
//!
//! Generates `full_adder.vhd`, `half_adder.vhd` (only when there are HAs), the structural
//! top, a behavioral twin with the same ports (`<name>_comportamental`, MT-21) and an
//! exhaustive (or sampled) testbench that checks both. Identifiers are those of the diagram and
//! the wiring table (spec 05). Comments and testbench messages follow the interface language:
//! their templates live here, in both languages, as ADR 0001 allows for generated artifacts.
//! Every generated file is plain ASCII: VHDL-93 sources are Latin-1, and GHDL rejects
//! characters such as an em dash even inside comments, so Portuguese goes without accents.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::expr::Polynomial;
use crate::matrix::DotMatrix;
use crate::netlist::{Netlist, Wire};
use crate::reduce::{CellKind, Time};
use crate::verify::sample_vectors;

/// Language of comments and testbench messages (VHD-06).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    PtBr,
    En,
}

/// Up to this many input bits the testbench is exhaustive; above, it uses the first
/// `TESTBENCH_SAMPLE` vectors of the verification sample (spec 04).
pub const TESTBENCH_EXHAUSTIVE_BITS: u32 = 20;
pub const TESTBENCH_SAMPLE: usize = 4096;
/// Default name of the top entity (spec 06).
pub const DEFAULT_NAME: &str = "dotree_circuito";
/// Suffix of the behavioral entity (VHD-10); the same in both languages, like every
/// identifier (VHD-06).
pub const BEHAVIORAL_SUFFIX: &str = "_comportamental";
/// Largest coefficient written as a VHDL-93 `integer` literal; above, a bit string.
const MAX_INTEGER: u64 = (1 << 31) - 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VhdlFile {
    pub name: String,
    pub contents: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VhdlError {
    /// The circuit is not complete yet (VHD-07).
    NotComplete,
    /// The entity name is not a usable VHDL identifier, or clashes with a design name.
    InvalidName,
}

/// VHDL name of the input port of variable `var`. `R` is the output, so an input variable
/// named `R` becomes `R_IN`.
pub fn port_name(var: char) -> String {
    if var == 'R' {
        "R_IN".to_string()
    } else {
        var.to_string()
    }
}

/// Generates the five files (four without HAs) for `netlist` (VHD-01 to VHD-10).
pub fn generate(
    matrix: &DotMatrix,
    netlist: &Netlist,
    name: &str,
    language: Language,
) -> Result<Vec<VhdlFile>, VhdlError> {
    if !netlist.complete {
        return Err(VhdlError::NotComplete);
    }
    if !is_valid_name(name, netlist) {
        return Err(VhdlError::InvalidName);
    }
    let g = Gen {
        m: matrix,
        n: netlist,
        name,
        lang: language,
    };
    let has_ha = netlist.cells.iter().any(|c| c.kind == CellKind::Ha);
    let mut files = vec![VhdlFile {
        name: "full_adder.vhd".into(),
        contents: g.full_adder(),
    }];
    if has_ha {
        files.push(VhdlFile {
            name: "half_adder.vhd".into(),
            contents: g.half_adder(),
        });
    }
    files.push(VhdlFile {
        name: format!("{name}.vhd"),
        contents: g.top(has_ha),
    });
    files.push(VhdlFile {
        name: format!("{name}{BEHAVIORAL_SUFFIX}.vhd"),
        contents: g.behavioral(),
    });
    files.push(VhdlFile {
        name: format!("tb_{name}.vhd"),
        contents: g.testbench(),
    });
    Ok(files)
}

const RESERVED: &[&str] = &[
    "abs",
    "access",
    "after",
    "alias",
    "all",
    "and",
    "architecture",
    "array",
    "assert",
    "attribute",
    "begin",
    "block",
    "body",
    "buffer",
    "bus",
    "case",
    "component",
    "configuration",
    "constant",
    "disconnect",
    "downto",
    "else",
    "elsif",
    "end",
    "entity",
    "exit",
    "file",
    "for",
    "function",
    "generate",
    "generic",
    "group",
    "guarded",
    "if",
    "impure",
    "in",
    "inertial",
    "inout",
    "is",
    "label",
    "library",
    "linkage",
    "literal",
    "loop",
    "map",
    "mod",
    "nand",
    "new",
    "next",
    "nor",
    "not",
    "null",
    "of",
    "on",
    "open",
    "or",
    "others",
    "out",
    "package",
    "port",
    "postponed",
    "procedure",
    "process",
    "pure",
    "range",
    "record",
    "register",
    "reject",
    "rem",
    "report",
    "return",
    "rol",
    "ror",
    "select",
    "severity",
    "signal",
    "shared",
    "sla",
    "sll",
    "sra",
    "srl",
    "subtype",
    "then",
    "to",
    "transport",
    "type",
    "unaffected",
    "units",
    "until",
    "use",
    "variable",
    "wait",
    "when",
    "while",
    "with",
    "xnor",
    "xor",
];

/// A VHDL-93 basic identifier that is not reserved and does not clash with any name of the
/// design (ports, signals, instances, components), case-insensitively.
fn is_valid_name(name: &str, n: &Netlist) -> bool {
    let b = name.as_bytes();
    let basic = !b.is_empty()
        && b[0].is_ascii_alphabetic()
        && b.iter().all(|c| c.is_ascii_alphanumeric() || *c == b'_')
        && !name.contains("__")
        && !name.ends_with('_');
    if !basic {
        return false;
    }
    let lower = name.to_ascii_lowercase();
    let mut taken: BTreeSet<String> = ["r", "full_adder", "half_adder"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    taken.extend(
        n.variables
            .iter()
            .map(|&(v, _)| port_name(v).to_ascii_lowercase()),
    );
    taken.extend(n.signals.iter().map(|s| s.ident()));
    for c in &n.cells {
        taken.insert(format!("s{}", c.number));
        taken.insert(format!("k{}", c.number));
        taken.insert(format!(
            "{}{}",
            if c.kind == CellKind::Fa { "fa" } else { "ha" },
            c.number
        ));
    }
    !RESERVED.contains(&lower.as_str()) && !taken.contains(&lower)
}

struct Gen<'a> {
    m: &'a DotMatrix,
    n: &'a Netlist,
    name: &'a str,
    lang: Language,
}

impl Gen<'_> {
    fn t<'s>(&self, pt: &'s str, en: &'s str) -> &'s str {
        match self.lang {
            Language::PtBr => pt,
            Language::En => en,
        }
    }

    /// A value in half units (`Time` or area) as text: "6", "14.5" ("14,5" in Portuguese).
    fn halves(&self, h: u32) -> String {
        let text = Time(h).to_string();
        if self.lang == Language::PtBr {
            text.replace('.', ",")
        } else {
            text
        }
    }

    fn full_adder(&self) -> String {
        format!(
            "-- full_adder - {}: s = x xor y xor z; k = {}(x, y, z)\n\
             library ieee;\nuse ieee.std_logic_1164.all;\n\n\
             entity full_adder is\n    port (\n        x, y, z : in  std_logic;\n        s, k    : out std_logic\n    );\nend full_adder;\n\n\
             architecture rtl of full_adder is\nbegin\n    s <= x xor y xor z;\n    k <= (x and y) or (x and z) or (y and z);\nend architecture rtl;\n",
            self.t("somador completo (FA)", "full adder (FA)"),
            self.t("maioria", "majority"),
        )
    }

    fn half_adder(&self) -> String {
        format!(
            "-- half_adder - {}: s = x xor y; k = x and y\n\
             library ieee;\nuse ieee.std_logic_1164.all;\n\n\
             entity half_adder is\n    port (\n        x, y : in  std_logic;\n        s, k : out std_logic\n    );\nend half_adder;\n\n\
             architecture rtl of half_adder is\nbegin\n    s <= x xor y;\n    k <= x and y;\nend architecture rtl;\n",
            self.t("meio-somador (HA)", "half adder (HA)"),
        )
    }

    fn expression(&self) -> String {
        Polynomial {
            terms: self.n.terms.clone(),
        }
        .to_string()
    }

    fn widths(&self) -> String {
        self.n
            .variables
            .iter()
            .map(|(v, w)| format!("{v} = {w}"))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// The entity declaration with the ports shared by the top and its behavioral twin.
    fn entity(&self, entity: &str) -> String {
        let mut o = format!("entity {entity} is\n    port (\n");
        for &(v, width) in &self.n.variables {
            let _ = writeln!(
                o,
                "        {} : in  std_logic_vector({} downto 0);",
                port_name(v),
                width - 1
            );
        }
        let _ = writeln!(
            o,
            "        R : out std_logic_vector({} downto 0)\n    );\nend {entity};\n",
            self.n.output_width - 1
        );
        o
    }

    /// A coefficient as an unsigned of `W` bits: `to_unsigned` while it fits a VHDL-93
    /// integer, a bit string beyond (the value is already reduced mod 2^W).
    fn coefficient(&self, c: u64) -> String {
        let w = self.n.output_width as usize;
        let c = if w >= 64 { c } else { c & ((1u64 << w) - 1) };
        if c <= MAX_INTEGER {
            format!("to_unsigned({c}, W)")
        } else {
            format!("unsigned'(\"{c:0w$b}\")")
        }
    }

    /// The behavioral twin of the top (VHD-10): the expression itself, in `numeric_std`,
    /// with every product and the sum reduced to W bits, as the testbench computes it.
    fn behavioral(&self) -> String {
        let n = self.n;
        let entity = format!("{}{BEHAVIORAL_SUFFIX}", self.name);
        let mut o = String::new();
        let _ = writeln!(
            o,
            "-- {entity} - {} (https://psiagoleal.github.io/dotree/)",
            self.t(
                "versao comportamental gerada pelo dotree",
                "behavioral version generated by dotree"
            )
        );
        let _ = writeln!(
            o,
            "-- {}: R = {}",
            self.t("Expressao", "Expression"),
            self.expression()
        );
        let _ = writeln!(
            o,
            "-- {}: {} bits; {} R {} {} bits",
            self.t("Larguras", "Widths"),
            self.widths(),
            self.t("saida", "output"),
            self.t("com", "with"),
            n.output_width
        );
        let _ = writeln!(
            o,
            "-- {} {}{}",
            self.t("Mesmas portas de", "Same ports as"),
            self.name,
            self.t(
                "; o sintetizador escolhe o circuito (para comparar com o estrutural).",
                "; the synthesizer chooses the circuit (to compare with the structural one)."
            )
        );
        o.push_str("\nlibrary ieee;\nuse ieee.std_logic_1164.all;\nuse ieee.numeric_std.all;\n\n");
        o.push_str(&self.entity(&entity));
        let _ = writeln!(o, "architecture comportamental of {entity} is");
        let _ = writeln!(o, "    constant W : natural := {};", n.output_width);
        o.push_str("begin\n    R <= std_logic_vector(\n");
        for (i, t) in n.terms.iter().enumerate() {
            let mut factors: Vec<String> = Vec::new();
            if t.coefficient != 1 || t.vars.is_empty() {
                factors.push(self.coefficient(t.coefficient));
            }
            factors.extend(
                t.vars
                    .iter()
                    .map(|&v| format!("resize(unsigned({}), W)", port_name(v))),
            );
            let mut term = factors[0].clone();
            for f in &factors[1..] {
                term = format!("resize({term} * {f}, W)");
            }
            if factors.len() == 1 && !term.starts_with("resize(") {
                term = format!("resize({term}, W)");
            }
            let lead = if i == 0 { "      " } else { "    + " };
            let _ = writeln!(o, "        {lead}{term}");
        }
        o.push_str("    );\nend architecture comportamental;\n");
        o
    }

    fn top(&self, has_ha: bool) -> String {
        let n = self.n;
        let w = n.output_width;
        let mut o = String::new();
        let (fa, ha) = n.cell_counts();
        let area = (2 * fa + ha) as u32;

        // ---- header (VHD-03)
        let _ = writeln!(
            o,
            "-- {} - {} (https://psiagoleal.github.io/dotree/)",
            self.name,
            self.t("gerado pelo dotree", "generated by dotree")
        );
        let _ = writeln!(
            o,
            "-- {}: R = {}",
            self.t("Expressao", "Expression"),
            self.expression()
        );
        let _ = writeln!(
            o,
            "-- {}: {} bits; {} R {} {w} bits",
            self.t("Larguras", "Widths"),
            self.widths(),
            self.t("saida", "output"),
            self.t("com", "with")
        );
        let _ = writeln!(
            o,
            "-- {}:",
            self.t("Matriz de pontos (nivel 0)", "Dot matrix (level 0)")
        );
        for col in (0..w).rev() {
            let dots: Vec<String> = self
                .m
                .dots
                .iter()
                .filter(|d| d.column == col)
                .map(|d| self.m.signals[d.signal].ident())
                .collect();
            let _ = writeln!(o, "--   2^{col} : {}", dots.join(" ")).map(|_| ());
        }
        let _ = writeln!(
            o,
            "-- {}: {fa} FA, {ha} HA, {} AND; {} {} A_FA; {} {} T_FA",
            self.t("Metricas", "Metrics"),
            n.and_gates(),
            self.t("area", "area"),
            self.halves(area),
            self.t("atraso", "delay"),
            self.halves(n.delay().0),
        );
        // `use work.all` makes full_adder and half_adder directly visible, which VHDL-93
        // requires for the default binding of their components (GHDL follows it strictly).
        o.push_str("\nlibrary ieee;\nuse ieee.std_logic_1164.all;\nuse work.all;\n\n");

        // ---- entity
        o.push_str(&self.entity(self.name));

        // ---- architecture
        let _ = writeln!(o, "architecture structural of {} is", self.name);
        o.push_str("    component full_adder is\n        port (x, y, z : in std_logic; s, k : out std_logic);\n    end component full_adder;\n");
        if has_ha {
            o.push_str("    component half_adder is\n        port (x, y : in std_logic; s, k : out std_logic);\n    end component half_adder;\n");
        }
        let _ = writeln!(
            o,
            "\n    -- {}",
            self.t(
                "bits de entrada e produtos parciais",
                "input bits and partial products"
            )
        );
        for s in &n.signals {
            let _ = writeln!(o, "    signal {} : std_logic;", s.ident());
        }
        let _ = writeln!(
            o,
            "    -- {}",
            self.t(
                "saidas das celulas (soma s, vai-um k)",
                "cell outputs (sum s, carry k)"
            )
        );
        for c in &n.cells {
            let _ = writeln!(o, "    signal s{} : std_logic;", c.number);
            if c.has_carry {
                let _ = writeln!(o, "    signal k{} : std_logic;", c.number);
            }
        }
        o.push_str("begin\n");
        let _ = writeln!(
            o,
            "    -- {}",
            self.t("bits de entrada e portas AND", "input bits and AND gates")
        );
        for s in &n.signals {
            let rhs: Vec<String> = s
                .factors
                .iter()
                .map(|f| format!("{}({})", port_name(f.var), f.bit))
                .collect();
            let _ = writeln!(o, "    {} <= {};", s.ident(), rhs.join(" and "));
        }
        let mut level = None;
        for c in &n.cells {
            let this = if c.cpa { None } else { Some(c.level) };
            if level != Some(this) {
                level = Some(this);
                let _ = match this {
                    Some(l) => writeln!(
                        o,
                        "\n    -- {} {l}",
                        self.t("celulas formadas sobre o nivel", "cells formed over level")
                    ),
                    None => writeln!(
                        o,
                        "\n    -- CPA ({})",
                        self.t("propagacao de vai-um", "ripple carry")
                    ),
                };
            }
            let (label, comp, names): (&str, &str, &[&str]) = match c.kind {
                CellKind::Fa => ("FA", "full_adder", &["x", "y", "z"]),
                CellKind::Ha => ("HA", "half_adder", &["x", "y"]),
            };
            let ins: Vec<String> = names
                .iter()
                .zip(&c.inputs)
                .map(|(f, &wire)| format!("{f} => {}", n.ident(wire)))
                .collect();
            let k = if c.has_carry {
                format!("k{}", c.number)
            } else {
                "open".into()
            };
            let _ = writeln!(
                o,
                "    {label}{} : {comp} port map ({}, s => s{}, k => {k});  -- 2^{}",
                c.number,
                ins.join(", "),
                c.number,
                c.column
            );
        }
        let _ = writeln!(o, "\n    -- {}", self.t("resultado", "result"));
        for (j, wire) in n.result.iter().enumerate() {
            let rhs = wire.map_or("'0'".to_string(), |w: Wire| n.ident(w));
            let _ = writeln!(o, "    R({j}) <= {rhs};");
        }
        o.push_str("end architecture structural;\n");
        o
    }

    fn testbench(&self) -> String {
        let n = self.n;
        let w = n.output_width as usize;
        let bits = n.input_bits();
        let exhaustive = bits <= TESTBENCH_EXHAUSTIVE_BITS;
        let total: u64 = if exhaustive {
            1 << bits
        } else {
            TESTBENCH_SAMPLE as u64
        };
        let tb = format!("tb_{}", self.name);
        let mut o = String::new();

        let _ = writeln!(
            o,
            "-- {tb} - {} (https://psiagoleal.github.io/dotree/)",
            self.t(
                "testbench gerado pelo dotree",
                "testbench generated by dotree"
            )
        );
        let _ = writeln!(
            o,
            "-- {}: R = {}",
            self.t("Confere", "Checks"),
            self.expression()
        );
        let _ = if exhaustive {
            writeln!(
                o,
                "-- {} {total} {}",
                self.t("para todas as", "for all"),
                self.t("combinacoes de entrada", "input combinations")
            )
        } else {
            writeln!(
                o,
                "-- {} {total} {}",
                self.t("para", "for"),
                self.t(
                    "vetores da amostra do dotree (xorshift64*, semente 0x00D07EE5)",
                    "vectors of the dotree sample (xorshift64*, seed 0x00D07EE5)"
                )
            )
        };
        o.push_str("library ieee;\nuse ieee.std_logic_1164.all;\nuse ieee.numeric_std.all;\n\n");
        let _ = writeln!(o, "entity {tb} is\nend {tb};\n");
        let _ = writeln!(o, "architecture test of {tb} is");
        for &(v, width) in &n.variables {
            let _ = writeln!(
                o,
                "    signal {} : std_logic_vector({} downto 0);",
                port_name(v),
                width - 1
            );
        }
        let _ = writeln!(o, "    signal R : std_logic_vector({} downto 0);", w - 1);
        let _ = writeln!(
            o,
            "    signal R_COMP : std_logic_vector({} downto 0);",
            w - 1
        );
        if !exhaustive {
            let _ = writeln!(
                o,
                "    type vectors_t is array (0 to {}) of std_logic_vector({} downto 0);",
                TESTBENCH_SAMPLE - 1,
                bits - 1
            );
            let _ = writeln!(o, "    constant VECTORS : vectors_t := (");
            let vs = sample_vectors(bits, TESTBENCH_SAMPLE);
            for (i, v) in vs.iter().enumerate() {
                let sep = if i + 1 == vs.len() { "" } else { "," };
                let _ = writeln!(o, "        \"{v:0width$b}\"{sep}", width = bits as usize);
            }
            o.push_str("    );\n");
        }
        o.push_str(
            "    function to_bin(v : std_logic_vector) return string is\n\
             \x20       variable s : string(1 to v'length);\n\
             \x20       variable j : integer := 1;\n\
             \x20   begin\n\
             \x20       for i in v'range loop\n\
             \x20           if v(i) = '1' then s(j) := '1'; else s(j) := '0'; end if;\n\
             \x20           j := j + 1;\n\
             \x20       end loop;\n\
             \x20       return s;\n\
             \x20   end function to_bin;\n",
        );
        o.push_str("begin\n");
        let ports = |r: &str| -> String {
            n.variables
                .iter()
                .map(|&(v, _)| format!("{0} => {0}", port_name(v)))
                .chain(std::iter::once(format!("R => {r}")))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let _ = writeln!(
            o,
            "    dut : entity work.{} port map ({});",
            self.name,
            ports("R")
        );
        let _ = writeln!(
            o,
            "    dut_comp : entity work.{}{BEHAVIORAL_SUFFIX} port map ({});\n",
            self.name,
            ports("R_COMP")
        );
        o.push_str("    stimulus : process\n");
        let _ = writeln!(
            o,
            "        variable v           : std_logic_vector({} downto 0);",
            bits - 1
        );
        let _ = writeln!(
            o,
            "        variable expected    : unsigned({} downto 0);",
            w - 1
        );
        let _ = writeln!(
            o,
            "        variable term        : unsigned({} downto 0);",
            w - 1
        );
        o.push_str("        variable errors      : natural := 0;\n        variable errors_comp : natural := 0;\n    begin\n");
        if exhaustive {
            let _ = writeln!(o, "        for n in 0 to {} loop", total - 1);
            let _ = writeln!(
                o,
                "            v := std_logic_vector(to_unsigned(n, {bits}));"
            );
        } else {
            o.push_str("        for n in VECTORS'range loop\n            v := VECTORS(n);\n");
        }
        let offsets = n.offsets();
        for &(v, width) in &n.variables {
            let lo = offsets[&v];
            let _ = writeln!(
                o,
                "            {} <= v({} downto {lo});",
                port_name(v),
                lo + width - 1
            );
        }
        o.push_str("            wait for 10 ns;\n            expected := (others => '0');\n");
        let mask: u128 = if w >= 128 {
            u128::MAX
        } else {
            (1u128 << w) - 1
        };
        for t in &n.terms {
            let coef = u128::from(t.coefficient) & mask;
            let _ = writeln!(o, "            term := \"{coef:0w$b}\";");
            for v in &t.vars {
                let _ = writeln!(
                    o,
                    "            term := resize(term * resize(unsigned({}), {w}), {w});",
                    port_name(*v)
                );
            }
            o.push_str("            expected := expected + term;\n");
        }
        let structural = self.t("ESTRUTURAL", "STRUCTURAL");
        let beh = self.t("COMPORTAMENTAL", "BEHAVIORAL");
        for (out, count, label) in [("R", "errors", structural), ("R_COMP", "errors_comp", beh)] {
            let _ = writeln!(
                o,
                "            if unsigned({out}) /= expected then\n                {count} := {count} + 1;\n                if {count} <= 10 then"
            );
            let _ = writeln!(
                o,
                "                    report \"{label}: {}\" & integer'image(n) & \": R = \" & to_bin({out}) & \", {} \" & to_bin(std_logic_vector(expected)) severity error;",
                self.t("ERRO no vetor ", "ERROR at vector "),
                self.t("esperado", "expected"),
            );
            o.push_str("                end if;\n            end if;\n");
        }
        o.push_str("        end loop;\n");
        for (count, label) in [("errors", structural), ("errors_comp", beh)] {
            let _ = writeln!(
                o,
                "        if {count} = 0 then\n            report \"{label}: {} {total} {}\" severity note;\n        else\n            report \"{label}: \" & integer'image({count}) & \" {}\" severity error;\n        end if;",
                self.t("TODOS OS", "ALL"),
                self.t("CASOS CORRETOS", "CASES CORRECT"),
                self.t("CASOS ERRADOS", "CASES WRONG"),
            );
        }
        let _ = writeln!(
            o,
            "        assert errors + errors_comp = 0 report \"{}\" severity failure;",
            self.t("VERIFICACAO FALHOU", "VERIFICATION FAILED")
        );
        o.push_str("        wait;\n    end process stimulus;\nend architecture test;\n");
        o
    }
}
