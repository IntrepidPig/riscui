use std::collections::HashMap;

use crate::{Instruction, parse, def};


pub fn compile(mut input: Vec<parse::Inst>, labels: &HashMap<String, u32>) -> Vec<Instruction> {
	let mut output = Vec::new();
	process_labels(&mut input, labels);
	for inst in &input {
		output.push(gen_code(inst));
	}
	output
}

fn process_labels(input: &mut Vec<parse::Inst>, labels: &HashMap<String, u32>) {
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

