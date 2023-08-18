use std::collections::HashMap;

use crate::compile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inst {
	pub name: String,
	pub rd: Option<u32>,
	pub rs1: Option<u32>,
	pub rs2: Option<u32>,
	pub imm: Option<Imm>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Imm {
	Value(i32),
	Label(String),
}

pub fn parse(input: &str) -> (Vec<Inst>, Vec<String>, HashMap<String, u32>) {
    let mut insts = Vec::new();
    let mut texts = Vec::new();
	let mut labels = HashMap::new();
	for line in input.lines() {
		let full_line = line;
		let line = line.split('#').next().unwrap();
		let line = line.trim();
		if line.is_empty() {
			continue;
		}
		if line.contains(':') {
			let label = line.split(':').next().unwrap().trim();
			labels.insert(label.to_owned(), insts.len().try_into().unwrap());
		} else {
			let inst = parse_line(line);
			let cinsts = compile::expand_pseudo(&inst);
			for inst in cinsts {
				insts.push(inst);
				texts.push(full_line.trim_start().to_owned());
			}
		}
	}
	(insts, texts, labels)
}

pub fn parse_line(line: &str) -> Inst {
	let line = line.to_lowercase();
	let line = line.replace(',', " ");
	let name = line.split_whitespace().next().unwrap();
	let rest = &line[name.len()..];
	let mut inst = Inst {
		name: name.to_owned(),
		rd: None,
		rs1: None,
		rs2: None,
		imm: None,
	};
	match name {
		"add" | "sub" | "and" | "or" | "xor" | "sll" | "srl" | "sra" | "slt" | "sltu" | "mul" => {
			let args = rest.split_whitespace().collect::<Vec<_>>();
			inst.rd = Some(parse_register(args[0]));
			inst.rs1 = Some(parse_register(args[1]));
			inst.rs2 = Some(parse_register(args[2]));
		},
		"addi" | "andi" | "ori" | "xori" => {
			let args = rest.split_whitespace().collect::<Vec<_>>();
			inst.rd = Some(parse_register(args[0]));
			inst.rs1 = Some(parse_register(args[1]));
			inst.imm = Some(parse_imm(args[2]));
		},
		"slli" | "srli" | "srai" => {
			let args = rest.split_whitespace().collect::<Vec<_>>();
			inst.rd = Some(parse_register(args[0]));
			inst.rs1 = Some(parse_register(args[1]));
			inst.imm = Some(parse_imm(args[2]));
		},
		"lb" | "lbu" | "lh" | "lhu" | "lw" => {
			let args = rest.split_whitespace().collect::<Vec<_>>();
			let rd = parse_register(args[0]);
			let other = args[1].split(|c| c == '(' || c == ')').collect::<Vec<_>>();
			let imm = parse_imm(other[0]);
			let rs1 = parse_register(other[1]);
			inst.rd = Some(rd);
			inst.rs1 = Some(rs1);
			inst.imm = Some(imm);
		},
		"sb" | "sh" | "sw" => {
			let args = rest.split_whitespace().collect::<Vec<_>>();
			let rs2 = parse_register(args[0]);
			let other = args[1].split(|c| c == '(' || c == ')').collect::<Vec<_>>();
			let imm = parse_imm(other[0]);
			let rs1 = parse_register(other[1]);
			inst.rs2 = Some(rs2);
			inst.rs1 = Some(rs1);
			inst.imm = Some(imm);
		},
		"beq" | "bge"  | "bgeu" | "blt" | "bltu" | "bne" => {
			let args = rest.split_whitespace().collect::<Vec<_>>();
			inst.rs1 = Some(parse_register(args[0]));
			inst.rs2 = Some(parse_register(args[1]));
			inst.imm = Some(parse_imm(args[2]));
		},
		"jal" => {
			let args = rest.split_whitespace().collect::<Vec<_>>();
			if args.len() == 1 {
				inst.rd = Some(1);
				inst.imm = Some(parse_imm(args[0]));
			} else {
				inst.rd = Some(parse_register(args[0]));
				inst.imm = Some(parse_imm(args[1]));
			}
		},
		"jalr" => {
			let args = rest.split_whitespace().collect::<Vec<_>>();
			inst.rd = Some(parse_register(args[0]));
			inst.rs1 = Some(parse_register(args[1]));
			inst.imm = Some(parse_imm(args[2]));
		},
		"auipc" | "lui" => {
			let args = rest.split_whitespace().collect::<Vec<_>>();
			inst.rd = Some(parse_register(args[0]));
			inst.imm = Some(parse_imm(args[1]));
		},
		"ebreak" => {
			inst.imm = Some(Imm::Value(0));
		},
		"ecall" => {
			inst.imm = Some(Imm::Value(1));
		},
		"beqz" | "bnez" => {
			let args = rest.split_whitespace().collect::<Vec<_>>();
			inst.rs1 = Some(parse_register(args[0]));
			inst.imm = Some(parse_imm(args[1]));
		},
		"j" => {
			let args = rest.split_whitespace().collect::<Vec<_>>();
			inst.imm = Some(parse_imm(args[0]));
		},
		"jr" => {
			let args = rest.split_whitespace().collect::<Vec<_>>();
			inst.rs1 = Some(parse_register(args[0]));
		},
		"la" => {
			let args = rest.split_whitespace().collect::<Vec<_>>();
			inst.rd = Some(parse_register(args[0]));
			inst.imm = Some(parse_imm(args[1]));
		},
		"li" => {
			let args = rest.split_whitespace().collect::<Vec<_>>();
			inst.rd = Some(parse_register(args[0]));
			inst.imm = Some(parse_imm(args[1]));
		},
		"mv" => {
			let args = rest.split_whitespace().collect::<Vec<_>>();
			inst.rd = Some(parse_register(args[0]));
			inst.rs1 = Some(parse_register(args[1]));
		},
		"neg" => {
			let args = rest.split_whitespace().collect::<Vec<_>>();
			inst.rd = Some(parse_register(args[0]));
			inst.rs1 = Some(parse_register(args[1]));
		},
		"nop" => {},
		"not" => {
			let args = rest.split_whitespace().collect::<Vec<_>>();
			inst.rd = Some(parse_register(args[0]));
			inst.rs1 = Some(parse_register(args[1]));
		},
		"ret" => {},
		u => panic!("Unknown instruction {u}"),
	}
	inst
}

fn parse_register(s: &str) -> u32 {
	match s {
		"zero" => return 0,
		"ra" => return 1,
		"sp" => return 2,
		"gp" => return 3,
		"tp" => return 4,
		_ => {},
	};
	let class = s.chars().next().unwrap();
	let num = s[1..].parse::<u32>().unwrap();
	if class == 'x' {
		return num;
	}
	for (i, &reg) in crate::def::REG_ALIASES.iter().enumerate() {
		if reg == s {
			return i as u32;
		}
	}
	
	panic!("invalid register {s}");
}

fn parse_imm(s: &str) -> Imm {
	if s.as_bytes()[0].is_ascii_alphabetic() {
		return Imm::Label(s.to_owned());
	} else {
		// TODO
		return Imm::Value(s.parse().unwrap())
	}
}