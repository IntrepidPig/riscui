use std::fmt;

pub mod compile;
pub mod parse;
pub mod def;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instruction(pub u32);

impl Instruction {
	pub fn opcode(self) -> u32 {
		self.0 & 0b1111111
	}

	pub fn rd(self) -> u32 {
		(self.0 >> 7) & 0b11111
	}

	pub fn funct3(self) -> u32 {
		(self.0 >> (7 + 5)) & 0b111
	}

	pub fn rs1(self) -> u32 {
		(self.0 >> (7 + 5 + 3)) & 0b11111
	}

	pub fn rs2(self) -> u32 {
		(self.0 >> (7 + 5 + 3 + 5)) & 0b11111
	}

	pub fn funct7(self) -> u32 {
		(self.0 >> (7 + 5 + 3 + 5 + 5)) & 0b1111111
	}
	
	pub fn set_opcode(&mut self, opcode: u32) {
		self.0 &= !0b1111111;
		self.0 |= opcode;
	}

	pub fn set_rd(&mut self, rd: u32) {
		self.0 &= !(0b11111 << 7);
		self.0 |= rd << 7;
	}

	pub fn set_funct3(&mut self, funct3: u32) {
		self.0 &= !(0b111 << (7 + 5));
		self.0 |= funct3 << (7 + 5);
	}

	pub fn set_rs1(&mut self, rs1: u32) {
		self.0 &= !(0b11111 << (7 + 5 + 3));
		self.0 |= rs1 << (7 + 5 + 3);
	}

	pub fn set_rs2(&mut self, rs2: u32) {
		self.0 &= !(0b11111 << (7 + 5 + 3 + 5));
		self.0 |= rs2 << (7 + 5 + 3 + 5);
	}

	pub fn set_funct7(&mut self, funct7: u32) {
		self.0 &= !(0b1111111 << (7 + 5 + 3 + 5 + 5));
		self.0 |= funct7 << (7 + 5 + 3 + 5 + 5);
	}

	pub fn format(self) -> InstructionFormat {
		use InstructionFormat::*;
		match self.opcode() {
			0b0110011 => R,
			0b0010011 => I,
			0b0000011 => I,
			0b0100011 => S,
			0b1100011 => B,
			0b1101111 => J,
			0b1100111 => I,
			0b0010111 => U,
			0b0110111 => U,
			0b1110011 => I,
			_ => panic!("Unknown opcode"),
		}
	}

	pub fn imm(self) -> i32 {
		self.imm_by_format(self.format())
	}

	pub fn imm_by_format(self, format: InstructionFormat) -> i32 {
		use InstructionFormat::*;
		match format {
			R => 0,
			I => self.imm_by_pieces(&[(20, 0, 12)], true),
			S => self.imm_by_pieces(&[(7, 0, 5), (25, 5, 7)], true),
			B => self.imm_by_pieces(&[(7, 11, 1), (8, 1, 4), (25, 5, 6), (31, 12, 1)], true),
			U => self.imm_by_pieces(&[(12, 12, 20)], false),
			J => self.imm_by_pieces(&[(12, 12, 8), (20, 11, 1), (21, 1, 10), (31, 20, 1)], true),
		}
	}

	/// Given a list of pieces specifed as tuples (instruction bit index start, immediate output bit
	/// index start, length), return the immediate created by combining those pieces and optionally
	/// sign extending the result to 32 bits.
	fn imm_by_pieces(self, pieces: &[(u32, u32, u32)], sign_extend: bool) -> i32 {
		let mut output: u32 = 0;
		// the highest bit that was controlled by the list of pieces
		let mut highest = 0;
		for &piece in pieces {
			// mask that zeroes out all but the selected bits
			let mask = (2u32.pow(piece.2) - 1) << piece.0;
			// move the selected bits back to the start, then to the desired output position, and
			// add them to the output
			output |= ((self.0 & mask) >> piece.0) << piece.1;
			// record the highest bit affected
			highest = std::cmp::max(highest, piece.1 + piece.2 - 1);
		}
		if sign_extend {
			let sign = output & (1 << highest) != 0;
			let mask = !(2u32.pow(highest) - 1);
			if sign {
				output |= 0xFFFFFFFF & mask;
			}
		}
		i32::from_le_bytes(output.to_le_bytes())
	}

	pub fn set_imm_by_format(&mut self, format: InstructionFormat, imm: i32) {
		use InstructionFormat::*;
		match format {
			R => panic!("R type instructions do not have immediates"),
			I => self.set_imm_by_pieces(&[(20, 0, 12)], imm),
			S => self.set_imm_by_pieces(&[(7, 0, 5), (25, 5, 7)], imm),
			B => self.set_imm_by_pieces(&[(7, 11, 1), (8, 1, 4), (25, 5, 6), (31, 12, 1)], imm),
			U => self.set_imm_by_pieces(&[(12, 12, 20)], imm),
			J => self.set_imm_by_pieces(&[(12, 12, 8), (20, 11, 1), (21, 1, 10), (31, 20, 1)], imm),
		}
	}
	
	fn set_imm_by_pieces(&mut self, pieces: &[(u32, u32, u32)], imm: i32) {
		let imm = imm as u32;
		for &piece in pieces {
			let value = (imm >> piece.1) & (2u32.pow(piece.2) - 1);
			self.0 |= value << piece.0;
		}
	}
	
	pub fn from_bytes(bytes: [u8; 4]) -> Self {
		Self(u32::from_le_bytes(bytes))
	}
}

impl fmt::Debug for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut buf = String::new();
		for i in 0..8 {
			let bits = (self.0 & (0b1111 << (i * 4))) >> (i * 4);
			buf = format!("{bits:0>4b}_{buf}");
		}
        write!(f, "Instruction(0b{})", buf.trim_matches('_'))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InstructionFormat {
	R,
	I,
	S,
	B,
	U,
	J,
}

#[test]
fn test_imm_pieces() {
	let tests = &[
		(
			0b11110000_11110000_11110000_11001100,
			&[(0, 0, 4), (12, 8, 4)],
			false,
			0b1111_00001100,
		),
		(
			0b11110000_11110000_11110000_11001100,
			&[(0, 0, 4), (12, 8, 4)],
			true,
			0xFFFFFF0Cu32 as i32,
		),
	];
	for test in tests {
		let output = Instruction(test.0).imm_by_pieces(test.1, test.2);
		println!("{:0b}", output);
		assert_eq!(output, test.3);
	}
}

#[test]
fn test_imm_pieces_formats() {
	use InstructionFormat::*;
	let tests = &[
		(0x00000013, I, 0),
		(0xfff00013, I, -1),
		(0xaaa00013, I, -1366),
		(0x00000023, S, 0),
		(0xfe000fa3, S, -1),
		(0xaa000523, S, -1366),
	];
	for test in tests {
		let output = Instruction(test.0).imm_by_format(test.1);
		println!("{output:0b}");
		assert_eq!(output, test.2);
	}
}
