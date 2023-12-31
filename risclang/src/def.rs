/// opcode, funct3, funct7, instruction name, instruction type
pub struct ISetElem(pub u32, pub Option<u32>, pub Option<u32>, pub &'static str, pub &'static str);

pub static ISET_DEFINITION: &[ISetElem] = &[
	ISetElem(0b0110011, Some(0b000), Some(0b0000000), "add", "R"),
	ISetElem(0b0110011, Some(0b000), Some(0b0100000), "sub", "R"),
	ISetElem(0b0110011, Some(0b111), Some(0b0000000), "and", "R"),
	ISetElem(0b0110011, Some(0b110), Some(0b0000000), "or", "R"),
	ISetElem(0b0110011, Some(0b100), Some(0b0000000), "xor", "R"),
	ISetElem(0b0110011, Some(0b001), Some(0b0000000), "sll", "R"),
	ISetElem(0b0110011, Some(0b101), Some(0b0000000), "srl", "R"),
	ISetElem(0b0110011, Some(0b101), Some(0b0100000), "sra", "R"),
	ISetElem(0b0110011, Some(0b010), Some(0b0000000), "slt", "R"),
	ISetElem(0b0110011, Some(0b011), Some(0b0000000), "sltu", "R"),
	ISetElem(0b0010011, Some(0b000), None, "addi", "I"),
	ISetElem(0b0010011, Some(0b000), None, "adni", "I"),
	ISetElem(0b0010011, Some(0b000), None, "ori", "I"),
	ISetElem(0b0010011, Some(0b000), None, "xori", "I"),
	ISetElem(0b0010011, Some(0b000), Some(0b0000000), "slli", "I"),
	ISetElem(0b0010011, Some(0b000), Some(0b0000000), "srli", "I"),
	ISetElem(0b0010011, Some(0b000), Some(0b0100000), "srai", "I"),
	ISetElem(0b0010011, Some(0b000), None, "slti", "I"),
	ISetElem(0b0010011, Some(0b000), None, "sltiu", "I"),
	ISetElem(0b0000011, Some(0b000), None, "lb", "I"),
	ISetElem(0b0000011, Some(0b000), None, "lbu", "I"),
	ISetElem(0b0000011, Some(0b000), None, "lh", "I"),
	ISetElem(0b0000011, Some(0b000), None, "lhu", "I"),
	ISetElem(0b0000011, Some(0b000), None, "lw", "I"),
	ISetElem(0b0100011, Some(0b000), None, "sb", "S"),
	ISetElem(0b0100011, Some(0b001), None, "sh", "S"),
	ISetElem(0b0100011, Some(0b010), None, "sw", "S"),
	ISetElem(0b1100011, Some(0b000), None, "beq", "B"),
	ISetElem(0b1100011, Some(0b101), None, "bge", "B"),
	ISetElem(0b1100011, Some(0b111), None, "bgeu", "B"),
	ISetElem(0b1100011, Some(0b100), None, "blt", "B"),
	ISetElem(0b1100011, Some(0b110), None, "bltu", "B"),
	ISetElem(0b1100011, Some(0b001), None, "bne", "B"),
	ISetElem(0b1101111, None, None, "jal", "J"),
	ISetElem(0b1100111, Some(0b000), None, "jalr", "I"),
	ISetElem(0b0010111, None, None, "auipc", "I"),
	ISetElem(0b0110111, None, None, "lui", "U"),
	ISetElem(0b1110011, Some(0b000), None, "ebreak", "I"),
	ISetElem(0b1110011, Some(0b000), None, "ecall", "I"),
	ISetElem(0b0110011, Some(0b000), Some(0b0000001), "mul", "R"),
];

pub static INST_FORMAT_IMM_PIECES: &[(&'static str, (&[(u32, u32, u32)], bool))] = &[
	("I", (&[(20, 0, 12)], true)),
	("S", (&[(7, 0, 5), (25, 5, 7)], true)),
	("B", (&[(7, 11, 1), (8, 1, 4), (25, 5, 6), (31, 12, 1)], true)),
	("U", (&[(12, 12, 20)], false)),
	("J", (&[(12, 12, 8), (20, 11, 1), (21, 1, 10), (31, 20, 1)], true)),
];

pub static INST_PIECES: &[(&'static str, (u32, u32))] = &[
	("opcode", (0, 7)),
	("rd", (7, 5)),
	("funct3", (12, 3)),
	("rs1", (15, 5)),
	("rs2", (20, 5)),
	("funct7", (25, 7)),
];

pub static REG_ALIASES: &[&'static str; 32] = &[
	"zero",
	"ra",
	"sp",
	"gp",
	"tp",
	"t0",
	"t1",
	"t2",
	"s0",
	"s1",
	"a0",
	"a1",
	"a2",
	"a3",
	"a4",
	"a5",
	"a6",
	"a7",
	"s2",
	"s3",
	"s4",
	"s5",
	"s6",
	"s7",
	"s8",
	"s9",
	"s10",
	"s11",
	"t3",
	"t4",
	"t5",
	"t6",
];

pub static PSEUDO_INSTS: &[&'static str] = &[
	"beqz",
	"bnez",
	"j",
	"jr",
	"la",
	"li",
	"mv",
	"neg",
	"nop",
	"not",
	"ret",
];