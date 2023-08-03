use std::collections::HashMap;

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

pub fn parse(input: &str) -> (Vec<Inst>, HashMap<String, u32>) {
	let mut insts = Vec::new();
	let mut labels = HashMap::new();
	for line in input.lines() {
		let line = line.split('#').next().unwrap();
		let line = line.trim();
		if line.is_empty() {
			continue;
		}
		if line.contains(':') {
			let label = line.split(':').next().unwrap().trim();
			labels.insert(label.to_owned(), insts.len().try_into().unwrap());
		} else {
			insts.push(parse_line(line));
		}
	}
	(insts, labels)
}

fn parse_line(line: &str) -> Inst {
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
		"add" | "sub" | "and" | "or" | "xor" | "sll" | "srl" | "sra" | "slt" | "sltu" => {
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
			inst.rd = Some(parse_register(args[0]));
			inst.imm = Some(parse_imm(args[1]));
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
			todo!()
		},
		"ecall" => {
			todo!()
		},
		u => panic!("Unknown instruction {u}"),
	}
	dbg!(inst)
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
	assert!(class == 'x' || class == 's' || class == 't' || class == 'a');
	let num = s[1..].parse::<u32>().unwrap();
	if class == 'x' {
		return num;
	}
	if class == 't' {
		if num <= 2 {
			return num + 5;
		}
		if 3 <= num && num <= 6 {
			return num - 3 + 28;
		}
	}
	if class == 's' {
		if num <= 1 {
			return num + 8;
		}
		if 2 <= num && num <= 11 {
			return num - 2 + 18;
		}
	}
	if class == 'a' {
		return num + 10;
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