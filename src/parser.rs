use crate::opcodes::Opcode;

#[derive(Debug)]
pub struct Parser {
	pub selection: Selection,
	pub opcode: Opcode,

	pub operand: u8,
	pub address: u8,

	pub is_relative: bool,
	pub length: usize,

	pub register_1: u8,
	pub register_2: u8,

	pub operand_1: Type,
	pub operand_2: Type
}

#[derive(Copy, Clone, Debug)]
pub enum Type {
	Empty,
	Register,
	Address,
	Constant
}

#[derive(Debug)]
pub enum Selection {
	RegReg,
	RegIm8,
	MemReg,
	RegMem,
	NoArguments,
	JustReg,
	JustMem,
	JustIm8
}

#[derive(Debug)]
pub enum ParserError {
	BusErrorOccurred(usize), // offset of bus error
	UnexpectedLength(usize), // the actual length read
}

impl Parser {
	pub fn new(
		byte_1: Option<&u8>,
		byte_2: Option<&u8>,
		byte_3: Option<&u8>) -> Result<Self, ParserError> {
		if byte_1.is_none() {
			return Err(ParserError::BusErrorOccurred(0));
		}
		
		let selection_bits = (*byte_1.unwrap() & 0b1110) >> 1;
		let selection = match selection_bits {
			0b000 => Selection::RegReg,
			0b001 => Selection::RegIm8,
			0b010 => Selection::MemReg,
			0b011 => Selection::RegMem,

			0b100 => Selection::NoArguments,
			0b101 => Selection::JustReg,
			0b110 => Selection::JustMem,
			0b111 => Selection::JustIm8,
			_ => unreachable!()
		};

		let opcode_bits = *byte_1.unwrap() >> 4;
		let opcode = match opcode_bits {
			0b0000 => Opcode::MOV,
			0b0001 => Opcode::HLT,
			0b0010 => Opcode::IN,
			0b0011 => Opcode::OUT,
			0b0100 => Opcode::JNZ,
			0b0101 => Opcode::ADD,
			0b0110 => Opcode::SUB,
			0b0111 => Opcode::PUSH,
			0b1000 => Opcode::POP,
			0b1001 => Opcode::INT,
			_ => unreachable!()
		};

		let is_relative = *byte_1.unwrap() & 0b1 > 0;

		let length: usize = match selection {
			Selection::NoArguments => 1,
			
			Selection::RegReg |
			Selection::JustIm8 |
			Selection::JustReg |
			Selection::JustMem => 2,
		
			Selection::RegMem |
			Selection::MemReg |
			Selection::RegIm8 => 3
		};

		let types = match selection {
			Selection::RegReg => (Type::Register, Type::Register),
			Selection::RegIm8 => (Type::Register, Type::Constant),
			Selection::MemReg => (Type::Address, Type::Register),
			Selection::RegMem => (Type::Register, Type::Address),
			Selection::NoArguments => (Type::Empty, Type::Empty),
			Selection::JustReg => (Type::Register, Type::Empty),
			Selection::JustMem => (Type::Address, Type::Empty),
			Selection::JustIm8 => (Type::Constant, Type::Empty)
		};

		let operand_1 = types.0;
		let operand_2 = types.1;

		let mut register_1 = 0u8;
		let mut register_2 = 0u8;

		let mut operand = 0u8;

		if length > 1 {
			if byte_2.is_none() {
				return Err(ParserError::UnexpectedLength(1));
			}
		
			match selection {
				Selection::RegReg |
				Selection::JustReg |
				Selection::RegMem |
				Selection::MemReg |
				Selection::RegIm8 => {
					register_1 = (*byte_2.unwrap() & 0xF0) >> 4;
					register_2 = *byte_2.unwrap() & 0x0F;
				}

				_ => { operand = *byte_2.unwrap() }
			}
		}

		if length > 2 {
			if byte_3.is_none() {
				return Err(ParserError::UnexpectedLength(2));
			}

			operand = *byte_3.unwrap();
		}

		Ok(Self {
			length,
			opcode,
			selection,
			is_relative,

			operand_1,
			operand_2,

			register_1,
			register_2,

			operand,
			address: operand
		})
	}
}
