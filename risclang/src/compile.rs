use std::{collections::HashMap};

use crate::{Instruction, parse, def};


pub fn compile(mut input: Vec<parse::Inst>, labels: &HashMap<String, u32>) -> Vec<Instruction> {
	let mut output = Vec::new();
	process_labels(&mut input, labels);
	for inst in &input {
		output.push(gen_code(inst));
	}
	output
}

pub fn process_labels(input: &mut Vec<parse::Inst>, labels: &HashMap<String, u32>) {
	for i in 0..input.len() {
		if let Some(ref mut imm) = input[i].imm {
			if let parse::Imm::Label(ref label) = imm.clone() {
				let target = *labels.get(label).unwrap();
				let offset = target as i32 - i as i32;
				*imm = parse::Imm::Value(offset * 4);
			}
		}
	}
}

fn gen_code(input: &parse::Inst) -> Instruction {
	let isetelem = def::ISET_DEFINITION.iter().find(|t| t.3 == input.name).unwrap();
	let mut inst = Instruction(0);
	inst.set_opcode(isetelem.0);
	if let Some(funct3) = isetelem.1 {
		inst.set_funct3(funct3);
	}
	if let Some(funct7) = isetelem.2 {
		inst.set_funct7(funct7);
	}
	if let Some(rd) = input.rd {
		inst.set_rd(rd);
	}
	if let Some(rs1) = input.rs1 {
		inst.set_rs1(rs1);
	}
	if let Some(rs2) = input.rs2 {
		inst.set_rs2(rs2);
	}
	if let Some(ref imm) = input.imm {
		match imm {
			parse::Imm::Value(val) => inst.set_imm_by_format(inst.format(), *val),
			parse::Imm::Label(label) => panic!("unresolved label {label}"),
		}
	}
	inst
}

pub fn expand_pseudo(inst: &parse::Inst) -> Vec<parse::Inst> {
	match &*inst.name {
		"beqz" => vec![parse::Inst {
			name: "beq".to_owned(),
			rs1: inst.rs1,
			rs2: Some(0),
			rd: None,
			imm: inst.imm.clone(),
		}],
		"bnez" => vec![parse::Inst {
			name: "bne".to_owned(),
			rs1: inst.rs1,
			rs2: Some(0),
			rd: None,
			imm: inst.imm.clone(),
		}],
		"j" => vec![parse::Inst {
			name: "jal".to_owned(),
			rs1: None,
			rs2: None,
			rd: inst.rd,
			imm: inst.imm.clone(),
		}],
		"jr" => vec![parse::Inst {
			name: "jalr".to_owned(),
			rs1: inst.rs1,
			rs2: None,
			rd: Some(0),
			imm: Some(parse::Imm::Value(0)),
		}],
		"la" => {
			let val = if let Some(ref imm) = inst.imm {
				if let parse::Imm::Value(val) = *imm {
					val
				} else {
					panic!("unresolved label");
				}
			} else {
				panic!("la label not specified");
			};
			let (h, l) = split_large_imm(val);
			vec![
				parse::Inst {
					name: "auipc".to_owned(),
					rs1: None,
					rs2: None,
					rd: inst.rd,
					imm: Some(parse::Imm::Value(h)),
				},
				parse::Inst {
					name: "addi".to_owned(),
					rs1: inst.rd,
					rs2: None,
					rd: inst.rd,
					imm: Some(parse::Imm::Value(l)),
				}
			]
		},
		"li" => {
			let val = if let Some(ref imm) = inst.imm {
				if let parse::Imm::Value(val) = *imm {
					val
				} else {
					panic!("unresolved label");
				}
			} else {
				panic!("li label not specified");
			};
			if val >= -2048 && val < 2048 {
				vec![parse::Inst {
					name: "addi".to_owned(),
					rs1: Some(0),
					rs2: None,
					rd: inst.rd,
					imm: inst.imm.clone(),
				}]
			} else {
				let (h, l) = split_large_imm(val);
				vec![
					parse::Inst {
						name: "lui".to_owned(),
						rs1: None,
						rs2: None,
						rd: inst.rd,
						imm: Some(parse::Imm::Value(h)),
					},
					parse::Inst {
						name: "addi".to_owned(),
						rs1: inst.rd,
						rs2: None,
						rd: inst.rd,
						imm: Some(parse::Imm::Value(l)),
					},
				]
			}
		},
		"mv" => vec![parse::Inst {
			name: "addi".to_owned(),
			rs1: inst.rs1,
			rs2: None,
			rd: inst.rd,
			imm: Some(parse::Imm::Value(0)),
		}],
		"neg" => vec![parse::Inst {
			name: "sub".to_owned(),
			rs1: Some(0),
			rs2: inst.rs1,
			rd: inst.rd,
			imm: None,
		}],
		"nop" => vec![parse::Inst {
			name: "addi".to_owned(),
			rs1: Some(0),
			rs2: None,
			rd: Some(0),
			imm: Some(parse::Imm::Value(0)),
		}],
		"not" => vec![parse::Inst {
			name: "xori".to_owned(),
			rs1: inst.rs1,
			rs2: None,
			rd: inst.rd,
			imm: Some(parse::Imm::Value(-1)),
		}],
		"ret" => vec![parse::Inst {
			name: "jalr".to_owned(),
			rs1: Some(1),
			rs2: None,
			rd: Some(0),
			imm: Some(parse::Imm::Value(0)),
		}],
		_ => vec![inst.clone()],
	}
}

fn split_large_imm(val: i32) -> (i32, i32) {
	let l = val & 0b111111111111;
	let h = (val & !0b111111111111) >> 12;
	if l & 0b100000000000 == 0 {
		(h, l)
	} else {
		(h - 0b1111_11111111_11111111, l)
	}
}

#[test]
fn test_imm_split() {
	let cases = &[
		0,
		1,
		0b100000000000,
		0b100000000001,
		0b111111111110,
		0b111111111111,
		0b1000000000000,
		0b1000000000001,
		0b1111111111110,
		0b1111111111111,
		-1,
		-2,
		unsafe { std::mem::transmute::<u32, i32>(0b11111111_11111111_11110000_00000000) },
		unsafe { std::mem::transmute::<u32, i32>(0b11111111_11111111_11110000_00000001) },
		unsafe { std::mem::transmute::<u32, i32>(0b11111111_11111111_11111000_00000000) },
		unsafe { std::mem::transmute::<u32, i32>(0b11111111_11111111_11111000_00000001) },
	];
	for &case in cases {
		let (h, l) = split_large_imm(case);
		println!("{:0b}, {:0b}", h, l);
		let r = (h << 12).checked_add((l << 20) >> 20).unwrap();
		assert_eq!(case, r);
	}
}