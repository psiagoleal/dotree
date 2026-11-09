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

use crate::board::{self, DE2_PINS, DISPLAYS, LEDS, MAX_INPUT_BITS, SEGMENTS};
use crate::expr::Polynomial;
use crate::matrix::DotMatrix;
use crate::netlist::{Netlist, Wire};
use crate::reduce::{CellKind, Time};
use crate::verify::sample_vectors;

/// `to_bin` of the testbenches: a vector as a string of 0s and 1s for the reports.
const TO_BIN: &str = "    function to_bin(v : std_logic_vector) return string is\n\
             \x20       variable s : string(1 to v'length);\n\
             \x20       variable j : integer := 1;\n\
             \x20   begin\n\
             \x20       for i in v'range loop\n\
             \x20           if v(i) = '1' then s(j) := '1'; else s(j) := '0'; end if;\n\
             \x20           j := j + 1;\n\
             \x20       end loop;\n\
             \x20       return s;\n\
             \x20   end function to_bin;\n";

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
/// Suffix of the DE2 board top (PLA-01).
pub const BOARD_SUFFIX: &str = "_de2";
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
    if board::fits(netlist).is_ok() {
        let sources: Vec<String> = files
            .iter()
            .map(|f| f.name.clone())
            .filter(|f| !f.starts_with("tb_"))
            .chain(std::iter::once(format!("{name}{BOARD_SUFFIX}.vhd")))
            .collect();
        files.push(VhdlFile {
            name: format!("{name}{BOARD_SUFFIX}.vhd"),
            contents: g.board_top(),
        });
        files.push(VhdlFile {
            name: format!("tb_{name}{BOARD_SUFFIX}.vhd"),
            contents: g.board_testbench(),
        });
        files.push(VhdlFile {
            name: format!("{name}.qpf"),
            contents: g.qpf(),
        });
        files.push(VhdlFile {
            name: format!("{name}.qsf"),
            contents: g.qsf(&sources),
        });
    }
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
    // The board top (PLA-07) uses `dut`, `seg7` and the board ports in every design.
    let mut taken: BTreeSet<String> = [
        "r",
        "full_adder",
        "half_adder",
        "dut",
        "seg7",
        "sw",
        "ledr",
        "hex0",
        "hex1",
        "hex2",
        "hex3",
        "hex4",
        "hex5",
        "hex6",
        "hex7",
    ]
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
            // The fixed dots of the constant (EXP-15) are the wire `one`, tied to '1'.
            let rhs = if rhs.is_empty() {
                "'1'".to_string()
            } else {
                rhs.join(" and ")
            };
            let _ = writeln!(o, "    {} <= {rhs};", s.ident());
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

    /// Testbench lines that compute `expected`, R mod 2^W, from the input signals.
    fn expected_lines(&self) -> String {
        let w = self.n.output_width as usize;
        let n = self.n;
        let mut o = String::from("            expected := (others => '0');\n");
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
        o
    }

    /// Which switches drive each variable, in the packing order of the testbench vectors
    /// (PLA-02): `(variable, "SW(hi downto lo)")`.
    fn switches(&self) -> Vec<(char, String)> {
        let offsets = self.n.offsets();
        self.n
            .variables
            .iter()
            .map(|&(v, w)| {
                let lo = offsets[&v];
                (v, format!("SW({} downto {lo})", lo + w - 1))
            })
            .collect()
    }

    /// Displays that show R: one hexadecimal digit per 4 bits (PLA-04).
    fn digits(&self) -> u32 {
        self.n.output_width.div_ceil(4)
    }

    /// The DE2 top (PLA-02 to PLA-04): switches into the structural top, R onto the red LEDs
    /// and, in hexadecimal, onto the seven-segment displays.
    fn board_top(&self) -> String {
        let n = self.n;
        let w = n.output_width;
        let entity = format!("{}{BOARD_SUFFIX}", self.name);
        let mut o = String::new();
        let _ = writeln!(
            o,
            "-- {entity} - {} (https://psiagoleal.github.io/dotree/)",
            self.t(
                "topo para a placa DE2 gerado pelo dotree",
                "DE2 board top generated by dotree"
            )
        );
        let _ = writeln!(
            o,
            "-- {}: R = {}",
            self.t("Expressao", "Expression"),
            self.expression()
        );
        let used = n.input_bits();
        let sw: Vec<String> = self
            .switches()
            .iter()
            .map(|(v, s)| format!("{v} = {s}"))
            .collect();
        let _ = writeln!(
            o,
            "-- {}: {}{}",
            self.t("Chaves", "Switches"),
            sw.join(", "),
            if used < MAX_INPUT_BITS {
                format!(
                    "; SW({} downto {used}) {}",
                    MAX_INPUT_BITS - 1,
                    self.t("sem uso", "unused")
                )
            } else {
                String::new()
            }
        );
        let shown = w.min(LEDS);
        let displays: Vec<String> = (0..self.digits())
            .rev()
            .map(|d| format!("HEX{d}"))
            .collect();
        let _ = writeln!(
            o,
            "-- {}: R ({w} bits) {} LEDR({} downto 0) {} {}",
            self.t("Resultado", "Result"),
            self.t("em", "on"),
            shown - 1,
            self.t("e em hexadecimal em", "and in hexadecimal on"),
            displays.join(" ")
        );
        let _ = writeln!(
            o,
            "-- {}: {}.qpf (Cyclone II EP2C35F672C6)",
            self.t("Projeto do Quartus", "Quartus project"),
            self.name
        );
        let _ = writeln!(
            o,
            "-- {} {}{BEHAVIORAL_SUFFIX} {}",
            self.t(
                "Para sintetizar a versao comportamental, troque o nome do componente e da instancia por",
                "To synthesize the behavioral version, rename the component and the instance to"
            ),
            self.name,
            self.t("(mesmas portas).", "(same ports).")
        );
        o.push_str("\nlibrary ieee;\nuse ieee.std_logic_1164.all;\nuse work.all;\n\n");
        let _ = writeln!(o, "entity {entity} is\n    port (");
        let _ = writeln!(
            o,
            "        SW   : in  std_logic_vector({} downto 0);",
            MAX_INPUT_BITS - 1
        );
        let _ = writeln!(
            o,
            "        LEDR : out std_logic_vector({} downto 0);",
            LEDS - 1
        );
        for h in 0..DISPLAYS {
            let end = if h + 1 == DISPLAYS { "" } else { ";" };
            let _ = writeln!(o, "        HEX{h} : out std_logic_vector(6 downto 0){end}");
        }
        let _ = writeln!(o, "    );\nend {entity};\n");

        let _ = writeln!(o, "architecture placa of {entity} is");
        let ports: Vec<String> = n
            .variables
            .iter()
            .map(|&(v, width)| {
                format!(
                    "{} : in std_logic_vector({} downto 0)",
                    port_name(v),
                    width - 1
                )
            })
            .chain(std::iter::once(format!(
                "R : out std_logic_vector({} downto 0)",
                w - 1
            )))
            .collect();
        let _ = writeln!(
            o,
            "    component {0} is\n        port ({1});\n    end component {0};\n",
            self.name,
            ports.join("; ")
        );
        let _ = writeln!(
            o,
            "    -- {}",
            self.t(
                "segmentos de um digito hexadecimal: bit i = segmento a+i, ativo em nivel baixo",
                "segments of a hexadecimal digit: bit i = segment a+i, active low"
            )
        );
        o.push_str("    subtype nibble is std_logic_vector(3 downto 0);\n");
        o.push_str("    function seg7(d : nibble) return std_logic_vector is\n    begin\n        case d is\n");
        for (d, segs) in SEGMENTS.iter().enumerate() {
            let _ = writeln!(o, "            when \"{d:04b}\" => return \"{segs:07b}\";");
        }
        o.push_str("            when others => return \"1111111\";\n        end case;\n    end function seg7;\n\n");
        let _ = writeln!(o, "    signal R : std_logic_vector({} downto 0);", w - 1);
        o.push_str("begin\n");
        let map: Vec<String> = self
            .switches()
            .iter()
            .map(|(v, s)| format!("{} => {s}", port_name(*v)))
            .chain(std::iter::once("R => R".to_string()))
            .collect();
        let _ = writeln!(
            o,
            "    dut : {} port map ({});\n",
            self.name,
            map.join(", ")
        );
        if w == LEDS {
            o.push_str("    LEDR <= R;\n");
        } else if w > LEDS {
            let _ = writeln!(o, "    LEDR <= R({} downto 0);", LEDS - 1);
        } else {
            let _ = writeln!(o, "    LEDR({} downto 0) <= R;", w - 1);
            let _ = writeln!(o, "    LEDR({} downto {w}) <= (others => '0');", LEDS - 1);
        }
        o.push('\n');
        for d in 0..DISPLAYS {
            let lo = 4 * d;
            if d < self.digits() {
                let hi = (lo + 3).min(w - 1);
                let pad = lo + 3 - hi;
                let bits = format!("R({hi} downto {lo})");
                let arg = if pad == 0 {
                    bits
                } else {
                    format!("\"{}\" & {bits}", "0".repeat(pad as usize))
                };
                let _ = writeln!(o, "    HEX{d} <= seg7({arg});");
            } else {
                let _ = writeln!(o, "    HEX{d} <= (others => '1');");
            }
        }
        o.push_str("end architecture placa;\n");
        o
    }

    /// The board testbench (PLA-06): every combination of the used switches, checking the
    /// LEDs and every display against the expression.
    fn board_testbench(&self) -> String {
        let n = self.n;
        let w = n.output_width as usize;
        let bits = n.input_bits();
        let total: u64 = 1 << bits;
        let entity = format!("{}{BOARD_SUFFIX}", self.name);
        let tb = format!("tb_{entity}");
        let mut o = String::new();
        let _ = writeln!(
            o,
            "-- {tb} - {} (https://psiagoleal.github.io/dotree/)",
            self.t(
                "testbench da placa DE2 gerado pelo dotree",
                "DE2 board testbench generated by dotree"
            )
        );
        let _ = writeln!(
            o,
            "-- {}: R = {} {} LEDR {} HEX0-HEX7, {} {total} {}",
            self.t("Confere", "Checks"),
            self.expression(),
            self.t("em", "on"),
            self.t("e", "and"),
            self.t("para todas as", "for all"),
            self.t("combinacoes das chaves", "switch combinations")
        );
        o.push_str("library ieee;\nuse ieee.std_logic_1164.all;\nuse ieee.numeric_std.all;\n\n");
        let _ = writeln!(o, "entity {tb} is\nend {tb};\n");
        let _ = writeln!(o, "architecture test of {tb} is");
        let _ = writeln!(
            o,
            "    signal SW   : std_logic_vector({} downto 0) := (others => '0');",
            MAX_INPUT_BITS - 1
        );
        let _ = writeln!(
            o,
            "    signal LEDR : std_logic_vector({} downto 0);",
            LEDS - 1
        );
        let hex: Vec<String> = (0..DISPLAYS).map(|h| format!("HEX{h}")).collect();
        let _ = writeln!(
            o,
            "    signal {} : std_logic_vector(6 downto 0);",
            hex.join(", ")
        );
        for &(v, width) in &n.variables {
            let _ = writeln!(
                o,
                "    signal {} : std_logic_vector({} downto 0);",
                port_name(v),
                width - 1
            );
        }
        o.push_str("    type segments_t is array (0 to 15) of std_logic_vector(6 downto 0);\n");
        let segs: Vec<String> = SEGMENTS.iter().map(|s| format!("\"{s:07b}\"")).collect();
        let _ = writeln!(
            o,
            "    constant SEGS : segments_t := ({});",
            segs.join(", ")
        );
        o.push_str("    constant OFF : std_logic_vector(6 downto 0) := \"1111111\";\n");
        o.push_str(TO_BIN);
        o.push_str("begin\n");
        let map: Vec<String> = std::iter::once("SW => SW".to_string())
            .chain(std::iter::once("LEDR => LEDR".to_string()))
            .chain(hex.iter().map(|h| format!("{h} => {h}")))
            .collect();
        let _ = writeln!(
            o,
            "    dut : entity work.{entity} port map ({});\n",
            map.join(", ")
        );
        o.push_str("    stimulus : process\n");
        let _ = writeln!(
            o,
            "        variable v        : std_logic_vector({} downto 0);",
            bits - 1
        );
        let _ = writeln!(
            o,
            "        variable expected : unsigned({} downto 0);",
            w - 1
        );
        let _ = writeln!(
            o,
            "        variable term     : unsigned({} downto 0);",
            w - 1
        );
        o.push_str("        variable wide     : unsigned(31 downto 0);\n");
        o.push_str("        variable ok       : boolean;\n");
        o.push_str("        variable errors   : natural := 0;\n    begin\n");
        let _ = writeln!(o, "        for n in 0 to {} loop", total - 1);
        let _ = writeln!(
            o,
            "            v := std_logic_vector(to_unsigned(n, {bits}));"
        );
        let _ = writeln!(o, "            SW({} downto 0) <= v;", bits - 1);
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
        o.push_str("            wait for 10 ns;\n");
        o.push_str(&self.expected_lines());
        o.push_str("            wide := resize(expected, 32);\n");
        let _ = writeln!(
            o,
            "            ok := LEDR = std_logic_vector(resize(expected, {LEDS}));"
        );
        for d in 0..DISPLAYS {
            if d < self.digits() {
                let _ = writeln!(
                    o,
                    "            ok := ok and HEX{d} = SEGS(to_integer(wide({} downto {})));",
                    4 * d + 3,
                    4 * d
                );
            } else {
                let _ = writeln!(o, "            ok := ok and HEX{d} = OFF;");
            }
        }
        let label = self.t("PLACA", "BOARD");
        o.push_str("            if not ok then\n                errors := errors + 1;\n                if errors <= 10 then\n");
        let _ = writeln!(
            o,
            "                    report \"{label}: {}\" & integer'image(n) & \": LEDR = \" & to_bin(LEDR) & \", {} \" & to_bin(std_logic_vector(expected)) & \", HEX0 = \" & to_bin(HEX0) severity error;",
            self.t("ERRO no vetor ", "ERROR at vector "),
            self.t("esperado R =", "expected R ="),
        );
        o.push_str("                end if;\n            end if;\n        end loop;\n");
        let _ = writeln!(
            o,
            "        if errors = 0 then\n            report \"{label}: {} {total} {}\" severity note;\n        else\n            report \"{label}: \" & integer'image(errors) & \" {}\" severity failure;\n        end if;",
            self.t("TODOS OS", "ALL"),
            self.t("CASOS CORRETOS", "CASES CORRECT"),
            self.t("CASOS ERRADOS", "CASES WRONG"),
        );
        o.push_str("        wait;\n    end process stimulus;\nend architecture test;\n");
        o
    }

    /// The Quartus II project file: its revision is the `.qsf` of the same name.
    fn qpf(&self) -> String {
        format!(
            "# {}.qpf - {} (https://psiagoleal.github.io/dotree/)\n\
             QUARTUS_VERSION = \"13.0\"\n\
             PROJECT_REVISION = \"{}\"\n",
            self.name,
            self.t(
                "projeto do Quartus II gerado pelo dotree",
                "Quartus II project generated by dotree"
            ),
            self.name
        )
    }

    /// The Quartus II settings (PLA-05): device, top, VHDL sources and the DE2 pins.
    fn qsf(&self, sources: &[String]) -> String {
        let mut o = String::new();
        let _ = writeln!(
            o,
            "# {}.qsf - {} (https://psiagoleal.github.io/dotree/)",
            self.name,
            self.t(
                "configuracao do Quartus II para a placa DE2 gerada pelo dotree",
                "Quartus II settings for the DE2 board generated by dotree"
            )
        );
        let _ = writeln!(
            o,
            "# {}",
            self.t(
                "Pinos: DE2 User Manual (Terasic), tabelas das chaves, LEDs e displays.",
                "Pins: DE2 User Manual (Terasic), tables of the switches, LEDs and displays."
            )
        );
        o.push_str("set_global_assignment -name FAMILY \"Cyclone II\"\n");
        o.push_str("set_global_assignment -name DEVICE EP2C35F672C6\n");
        let _ = writeln!(
            o,
            "set_global_assignment -name TOP_LEVEL_ENTITY {}{BOARD_SUFFIX}",
            self.name
        );
        o.push_str("set_global_assignment -name VHDL_INPUT_VERSION VHDL_1993\n");
        o.push_str("set_global_assignment -name RESERVE_ALL_UNUSED_PINS \"AS INPUT TRI-STATED\"\n");
        for f in sources {
            let _ = writeln!(o, "set_global_assignment -name VHDL_FILE {f}");
        }
        for (port, pin) in DE2_PINS {
            let _ = writeln!(o, "set_location_assignment {pin} -to {port}");
        }
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
        o.push_str(TO_BIN);
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
        o.push_str("            wait for 10 ns;\n");
        o.push_str(&self.expected_lines());
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
