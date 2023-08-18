use risclang::*;

pub fn compile(text: &str) -> Vec<u8> {
	let (parsed_insts, _texts, labels) = parse::parse(&text);
	let code = dbg!(compile::compile(parsed_insts, &labels));
	let code = code.into_iter().map(|inst| inst.0.to_le_bytes()).flatten().collect::<Vec<_>>();
	code
}

pub struct Machine {
	pub regs: [i32; 32],
	pub mem: Vec<u8>,
	pub pc: i32,
}

impl Machine {
	pub fn new(mem_size: usize) -> Self {
		let mut this = Self {
			regs: [0; 32],
			mem: vec![0; mem_size],
			pc: 0,
		};
		this.regs[2] = mem_size.try_into().unwrap();
		this
	}

	pub fn run(&mut self, code: &[u8]) {
		while (self.pc as usize) < code.len() {
			let pc = self.pc as usize;
			let inst = Instruction::from_bytes([code[pc], code[pc+1], code[pc+2], code[pc+3]]);
			self.exec(inst);
			self.regs[0] = 0;
		}
	}

	pub fn exec(&mut self, inst: Instruction) -> Option<(i32, i32)> {
		let rs1 = self.regs[inst.rs1() as usize];
		let rs2 = self.regs[inst.rs2() as usize];
		let rd = &mut self.regs[inst.rd() as usize];
		let imm = inst.imm();
		let mut pcmod = false;
		let mut ret = None;
		match inst.opcode() {
			0b0110011 => match (inst.funct3(), inst.funct7()) {
				(0b000, 0b0000000) => *rd = rs1 + rs2,
				(0b000, 0b0100000) => *rd = rs1 - rs2,
				(0b111, 0b0000000) => *rd = rs1 & rs2,
				(0b110, 0b0000000) => *rd = rs1 | rs2,
				(0b100, 0b0000000) => *rd = rs1 ^ rs2,
				(0b001, 0b0000000) => *rd = rs1 << rs2,
				(0b101, 0b0000000) => *rd = (rs1 as u32 >> rs2 as u32) as i32,
				(0b101, 0b0100000) => *rd = rs1 >> rs2,
				(0b010, 0b0000000) => *rd = if rs1 < rs2 { 1 } else { 0 },
				(0b011, 0b0000000) => *rd = if (rs1 as u32) < (rs2 as u32) { 1 } else { 0 },
				(0b000, 0b0000001) => *rd = rs1 * rs2,
				_ => panic!("invalid instruction"),
			},
			0b0010011 => match (inst.funct3(), inst.funct7()) {
				(0b000, _) => *rd = rs1 + imm,
				(0b111, _) => *rd = rs1 & imm,
				(0b110, _) => *rd = rs1 | imm,
				(0b100, _) => *rd = rs1 ^ imm,
				(0b001, 0b0000000) => *rd = rs1 << imm,
				(0b101, 0b0000000) => *rd = ((rs1 as u32) >> imm) as i32,
				(0b101, 0b0100000) => *rd = rs1 >> imm,
				(0b010, _) => *rd = if rs1 < imm { 1 } else { 0 },
				(0b011, _) => *rd = if (rs1 as u32) < (imm as u32) { 1 } else { 0 },
				_ => panic!("invalid instruction"),
			},
			0b0000011 => {
				let addr: usize = (rs1 + imm).try_into().unwrap();
				match inst.funct3() {
					0b000 => *rd = self.mem[addr] as i8 as i32,
					0b100 => *rd = self.mem[addr] as u32 as i32,
					0b001 => *rd = i16::from_le_bytes([self.mem[addr], self.mem[addr + 1]]) as i32,
					0b101 => *rd = u16::from_le_bytes([self.mem[addr], self.mem[addr + 1]]) as u32 as i32,
					0b010 => *rd = i32::from_le_bytes([self.mem[addr], self.mem[addr + 1], self.mem[addr + 2], self.mem[addr + 3]]),
					_ => panic!("invalid instruction"),
				}
			},
			0b0100011 => {
				let addr: usize = (rs1 + imm).try_into().unwrap();
				let bytes = rs2.to_le_bytes();
				match inst.funct3() {
					0b000 => self.mem[addr] = bytes[0],
					0b001 => self.mem[addr..addr+2].copy_from_slice(&bytes[0..2]),
					0b010 => self.mem[addr..addr+4].copy_from_slice(&bytes[0..4]),
					_ => panic!("invalid instruction"),
				}
			},
			0b1100011 => {
				let cond = match inst.funct3() {
					0b000 => rs1 == rs2,
					0b101 => rs1 >= rs2,
					0b111 => (rs1 as u32) >= (rs2 as u32),
					0b100 => rs1 < rs2,
					0b110 => (rs1 as u32) < (rs2 as u32),
					0b001 => rs1 != rs2,
					_ => panic!("invalid instruction"),
				};
				if cond {
					self.pc += imm;
					pcmod = true;
				}
			},
			0b1101111 => {
				*rd = self.pc + 4;
				self.pc += imm;
				pcmod = true;
			},
			0b1100111 => match inst.funct3() {
				0b000 => {
					*rd = self.pc + 4;
					self.pc = rs1 + imm;
					pcmod = true;
				},
				_ => panic!("invalid instruction"),
			},
			0b0010111 => {
				*rd = self.pc + imm;	
			},
			0b0110111 => {
				*rd = imm;
			},
			0b1110011 => match inst.funct3() {
				0b000 => match imm {
					0 => {
						// ebreak
					}
					1 => {
						// ecall
						ret = Some((self.regs[10], self.regs[11]));
					}
					_ => panic!("invalid environment instruction immediate")
				},
				_ => panic!("invalid instruction"),
			},
			_ => panic!("invalid instruction"),
		}
		
		self.regs[0] = 0;
		if !pcmod {
			self.pc += 4;
		}
		
		ret
	}
	
	pub fn dump_registers(&self) {
		println!("\nRegisters\n---------");
		for i in 0..32 {
			println!("{: <3} ({: <3}): {}", format!("x{}", i), def::REG_ALIASES[i], self.regs[i]);
		}
	}
}

#[test]
fn test_machine() {
	let mut machine = Machine::new(1024);
	let test = "
	addi x1 x0 10
	";
	machine.run(&compile(test));
	assert!(machine.regs[1] == 10);
}

#[test]
fn test_machine_fib() {
	let mut machine = Machine::new(1024);
	let test = "
	addi t0 x0 0
	addi t1 x0 1
	addi t2 x0 6
	loop:
	add t3 t0 t1
	add t0 x0 t1
	add t1 x0 t3
	addi t2 t2 -1
	blt x0 t2 loop
	";
	machine.run(&compile(test));
	assert!(machine.regs[5] == 8);
}

#[test]
fn test_machine_fib2() {
	let mut machine = Machine::new(1024);
	let test = "
	addi t0 x0 0
	addi t1 x0 1
	addi t5 x0 10
	start:
	add t2 t0 t1
	addi t0 t1 0
	addi t1 t2 0
	addi t5 t5 -1
	bge t5 x0 start
	addi t3 x0 -1
	";
	let code = compile(test);
	machine.run(&code);
	assert!(machine.regs[5] == 89);
}

#[test]
fn test_pseudo_insts() {
	let mut machine = Machine::new(1024);
	let test = "
	li x1 3
	li x2 2500
	li x3 -10000
	";
	machine.run(&compile(test));
	assert_eq!(machine.regs[1], 3);
	assert_eq!(machine.regs[2], 2500);
	assert_eq!(machine.regs[3], -10000);
}

#[test]
fn kinda_complex() {
	let mut machine = Machine::new(1048576);
	let test = "
main:
    # load the value of n into a0
    li a0 2

    # load the value of exp into a1
    li a1 10

    # call ex3
    jal ex3

    # prints the output of ex3
    mv a1 a0
    li a0 1
    ecall # Print Result

    # exits the program
    li a0 17
    li a1 0
    ecall

ex3:
    # this function is a recursive pow function
    # a0 contains the base
    # a1 contains the power to raise to
    # the return value should be the result of a0^a1
    #     where ^ is the exponent operator, not XOR
    addi sp sp -4
    sw ra 0(sp)

    # return 1 if a0 == 0
    beq a1 x0 ex3_zero_case

    # otherwise, return ex3(a0, a1-1) * a0
    mv t0 a0      # save a0 in t0
    addi a1 a1 -1 # decrement a1
    
    addi sp sp -4
    sw t0 0(sp)
    jal ex3       # call ex3(a0, a1-1)
    lw t0 0(sp)
    addi sp sp 4

    mul a0 a0 t0  # multiply ex3(a0, a1-1) by t0
                  # (which contains the value of a0)

    j ex3_end

ex3_zero_case:
    li a0 1

ex3_end:
    lw ra 0(sp)
    addi sp sp 4
    ret
	";
	machine.run(&compile(test));
}