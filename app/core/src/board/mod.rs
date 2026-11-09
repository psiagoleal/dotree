// Caminho relativo: app/core/src/board/mod.rs
//! The DE2 board (spec 10; MT-22): which circuits fit it, its pins and the seven-segment
//! digits. The VHDL and the Quartus files are written by `vhdl`.
//!
//! Pins: *DE2 Development and Education Board User Manual* (Terasic), tables of the toggle
//! switches, red LEDs and seven-segment displays; the same data as the `DE2_pin_assignments`
//! file shipped with the board. Checked pin by pin against two copies of that file on
//! 2026-11-09 (test `pla05_pins_match_the_board_pin_file`).

use crate::netlist::Netlist;

/// Toggle switches on the board: at most this many input bits (PLA-01).
pub const MAX_INPUT_BITS: u32 = 18;
/// Eight hexadecimal displays: at most this many output bits (PLA-01).
pub const MAX_OUTPUT_WIDTH: u32 = 32;
/// Red LEDs.
pub const LEDS: u32 = 18;
/// Seven-segment displays.
pub const DISPLAYS: u32 = 8;

/// Why a circuit does not fit the board (PLA-01).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardLimit {
    /// More input bits than switches (the number of input bits).
    TooManyInputs(u32),
    /// More output bits than the displays show (the output width).
    TooWide(u32),
}

/// Whether the complete circuit `n` fits the DE2 board (PLA-01).
pub fn fits(n: &Netlist) -> Result<(), BoardLimit> {
    let bits = n.input_bits();
    if bits > MAX_INPUT_BITS {
        return Err(BoardLimit::TooManyInputs(bits));
    }
    if n.output_width > MAX_OUTPUT_WIDTH {
        return Err(BoardLimit::TooWide(n.output_width));
    }
    Ok(())
}

/// Segments of each hexadecimal digit, active low: bit `i` is segment a + i, '1' is off
/// (PLA-04).
pub const SEGMENTS: [u8; 16] = [
    0x40, 0x79, 0x24, 0x30, 0x19, 0x12, 0x02, 0x78, 0x00, 0x10, 0x08, 0x03, 0x46, 0x21, 0x06, 0x0E,
];

/// Board port bit and FPGA pin, in port order: SW, LEDR, HEX0 to HEX7 (PLA-05).
pub const DE2_PINS: [(&str, &str); 92] = [
    ("SW[0]", "PIN_N25"),
    ("SW[1]", "PIN_N26"),
    ("SW[2]", "PIN_P25"),
    ("SW[3]", "PIN_AE14"),
    ("SW[4]", "PIN_AF14"),
    ("SW[5]", "PIN_AD13"),
    ("SW[6]", "PIN_AC13"),
    ("SW[7]", "PIN_C13"),
    ("SW[8]", "PIN_B13"),
    ("SW[9]", "PIN_A13"),
    ("SW[10]", "PIN_N1"),
    ("SW[11]", "PIN_P1"),
    ("SW[12]", "PIN_P2"),
    ("SW[13]", "PIN_T7"),
    ("SW[14]", "PIN_U3"),
    ("SW[15]", "PIN_U4"),
    ("SW[16]", "PIN_V1"),
    ("SW[17]", "PIN_V2"),
    ("LEDR[0]", "PIN_AE23"),
    ("LEDR[1]", "PIN_AF23"),
    ("LEDR[2]", "PIN_AB21"),
    ("LEDR[3]", "PIN_AC22"),
    ("LEDR[4]", "PIN_AD22"),
    ("LEDR[5]", "PIN_AD23"),
    ("LEDR[6]", "PIN_AD21"),
    ("LEDR[7]", "PIN_AC21"),
    ("LEDR[8]", "PIN_AA14"),
    ("LEDR[9]", "PIN_Y13"),
    ("LEDR[10]", "PIN_AA13"),
    ("LEDR[11]", "PIN_AC14"),
    ("LEDR[12]", "PIN_AD15"),
    ("LEDR[13]", "PIN_AE15"),
    ("LEDR[14]", "PIN_AF13"),
    ("LEDR[15]", "PIN_AE13"),
    ("LEDR[16]", "PIN_AE12"),
    ("LEDR[17]", "PIN_AD12"),
    ("HEX0[0]", "PIN_AF10"),
    ("HEX0[1]", "PIN_AB12"),
    ("HEX0[2]", "PIN_AC12"),
    ("HEX0[3]", "PIN_AD11"),
    ("HEX0[4]", "PIN_AE11"),
    ("HEX0[5]", "PIN_V14"),
    ("HEX0[6]", "PIN_V13"),
    ("HEX1[0]", "PIN_V20"),
    ("HEX1[1]", "PIN_V21"),
    ("HEX1[2]", "PIN_W21"),
    ("HEX1[3]", "PIN_Y22"),
    ("HEX1[4]", "PIN_AA24"),
    ("HEX1[5]", "PIN_AA23"),
    ("HEX1[6]", "PIN_AB24"),
    ("HEX2[0]", "PIN_AB23"),
    ("HEX2[1]", "PIN_V22"),
    ("HEX2[2]", "PIN_AC25"),
    ("HEX2[3]", "PIN_AC26"),
    ("HEX2[4]", "PIN_AB26"),
    ("HEX2[5]", "PIN_AB25"),
    ("HEX2[6]", "PIN_Y24"),
    ("HEX3[0]", "PIN_Y23"),
    ("HEX3[1]", "PIN_AA25"),
    ("HEX3[2]", "PIN_AA26"),
    ("HEX3[3]", "PIN_Y26"),
    ("HEX3[4]", "PIN_Y25"),
    ("HEX3[5]", "PIN_U22"),
    ("HEX3[6]", "PIN_W24"),
    ("HEX4[0]", "PIN_U9"),
    ("HEX4[1]", "PIN_U1"),
    ("HEX4[2]", "PIN_U2"),
    ("HEX4[3]", "PIN_T4"),
    ("HEX4[4]", "PIN_R7"),
    ("HEX4[5]", "PIN_R6"),
    ("HEX4[6]", "PIN_T3"),
    ("HEX5[0]", "PIN_T2"),
    ("HEX5[1]", "PIN_P6"),
    ("HEX5[2]", "PIN_P7"),
    ("HEX5[3]", "PIN_T9"),
    ("HEX5[4]", "PIN_R5"),
    ("HEX5[5]", "PIN_R4"),
    ("HEX5[6]", "PIN_R3"),
    ("HEX6[0]", "PIN_R2"),
    ("HEX6[1]", "PIN_P4"),
    ("HEX6[2]", "PIN_P3"),
    ("HEX6[3]", "PIN_M2"),
    ("HEX6[4]", "PIN_M3"),
    ("HEX6[5]", "PIN_M5"),
    ("HEX6[6]", "PIN_M4"),
    ("HEX7[0]", "PIN_L3"),
    ("HEX7[1]", "PIN_L2"),
    ("HEX7[2]", "PIN_L9"),
    ("HEX7[3]", "PIN_L6"),
    ("HEX7[4]", "PIN_L7"),
    ("HEX7[5]", "PIN_P9"),
    ("HEX7[6]", "PIN_N9"),
];
