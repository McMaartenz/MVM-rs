use std::collections::HashMap;
use std::num::Wrapping;

use crate::memory::Memory;
use crate::disk::Disk;
use crate::opcodes::Opcode;
use crate::parser::{Parser, Type, Selection, ParserError};
use crate::port::Port;
use crate::interrupts::{Interrupt, InterruptVec};

macro_rules! get_op{($s:expr,$p:expr,$b:expr) => { $s.get_operand($b, $p)? }}
macro_rules! set_op{($s:expr,$p:expr,$b:expr,$v:expr) => { $s.set_operand($b, $p, $v)? }}

macro_rules! get_op_1{($s:expr,$p:expr) => { get_op!($s, $p, true) }}
macro_rules! get_op_2{($s:expr,$p:expr) => { get_op!($s, $p, false) }}

macro_rules! set_op_1{($s:expr,$p:expr,$v:expr) => { set_op!($s, $p, true, $v) }}
macro_rules! set_op_2{($s:expr,$p:expr,$v:expr) => { set_op!($s, $p, false, $v) }}

pub struct Computer {
	pub memory: Memory,
	pub disk: Disk,
	pub io: HashMap<usize, Port>,
	pub ivt: Vec<InterruptVec>,
	
	pub running: bool,

	pub ip: u16,
	pub sp: u16,
	pub ax: u16,
	pub bx: u16,
	pub cx: u16,
	pub dx: u16,
	pub ex: u16,

	pub flags: u8,
}

#[derive(Debug)]
pub enum CPUError {
	MemoryAccessError(usize),
	IOPortAccessError(usize),
	SetImmutableOperand,
	GetEmptyOperand,
	IVTAccessError(usize),
	InvalidOpcode,
}

impl Computer {
	pub fn new(memory_size: usize, disk: Disk) -> Self {
		let memory = Memory::new(memory_size);

		Self {
			memory,
			disk,
			io: HashMap::new(),
			ivt: Vec::new(),
			running: false,
			ip: 0,
			sp: 0,
			ax: 0,
			bx: 0,
			cx: 0,
			dx: 0,
			ex: 0,
			flags: 0
		}
	}

	pub fn boot(&mut self) {
		self.running = true;

		let mut i = 0usize;
		for n in &self.disk.data {
			self.memory.data[i] = *n;
			i += 1;
		}

		self.ip = 0x0000;
		self.sp = 0x02FF;
	}

	pub fn tick(&mut self) {
		if let Err(tick_err) = self.tick_internal() {
			let irq_creation = match tick_err {
				CPUError::MemoryAccessError(address) => {
					self.ax = 0;
					self.irq(Interrupt::MachineCheck as u8)
				},

				CPUError::IOPortAccessError(port_number) => {
					self.ax = 1;
					self.irq(Interrupt::MachineCheck as u8)
				},

				CPUError::IVTAccessError(iv_entry) => {
					self.ax = 2; // should be NMI
					self.irq(Interrupt::MachineCheck as u8)
				},

				CPUError::SetImmutableOperand |
				CPUError::GetEmptyOperand |
				CPUError::InvalidOpcode => {
					self.irq(Interrupt::InvalidOpcode as u8)
				}
			};

			if irq_creation.is_err() {
				todo!("nmi, double, triple faults");
			}
		}
	}

	#[inline(always)]
	fn tick_internal(&mut self) -> Result<(), CPUError> {
		let byte_1 = self.memory.data.get(self.ip as usize);
		let byte_2 = self.memory.data.get(self.ip as usize + 1);
		let byte_3 = self.memory.data.get(self.ip as usize + 2);

		let parse_result = Parser::new(byte_1, byte_2, byte_3);
		if parse_result.is_err() {
			return Err(match parse_result.unwrap_err() {
				ParserError::BusErrorOccurred(address) => {
					CPUError::MemoryAccessError(address)
				},
				ParserError::UnexpectedLength(actual) => {
					todo!("unexpected instruction length")
				}
			});
		}

		let parsed = parse_result.unwrap();
		match parsed.opcode {
			Opcode::MOV => {
				set_op_1!(self, &parsed, get_op_2!(self, &parsed));
			},

			Opcode::OUT => {
				let port_address = get_op_2!(self, &parsed) as usize;
				if let Some(port) = self.io.get(&port_address) {
					port.out(get_op_1!(self, &parsed));
				} else {
					return Err(CPUError::IOPortAccessError(port_address));
				}
			},

			Opcode::IN => {
				let port_address = get_op_2!(self, &parsed) as usize;
				if let Some(port) = self.io.get(&port_address) {
					set_op_1!(self, &parsed, port.r#in());
				}
				else {
					return Err(CPUError::IOPortAccessError(port_address));
				}
			},

			Opcode::SUB => {
				let a = get_op_1!(self, &parsed) as u16;
				let b = get_op_2!(self, &parsed) as u16;

				let c = Wrapping(a) - Wrapping(b);
				set_op_1!(self, &parsed, c.0 as u8);
				self.set_flags(c.0);
			},

			Opcode::ADD => {
				let a = get_op_1!(self, &parsed) as u16;
				let b = get_op_2!(self, &parsed) as u16;

				let c = Wrapping(a) + Wrapping(b);
				set_op_1!(self, &parsed, c.0 as u8);
				self.set_flags(c.0);
			},

			Opcode::PUSH => {
				match parsed.selection {
					Selection::RegReg |
					Selection::RegIm8 |
					Selection::MemReg |
					Selection::RegMem => {
						let val: u16 =
							(get_op_1!(self, &parsed) as u16) << 8 |
							get_op_2!(self, &parsed) as u16;

						self.memory.set(self.sp as usize, (val >> 8) as u8)?;
						self.memory.set(self.sp as usize + 1, val as u8)?;
						self.sp -= 2;
					},

					Selection::JustMem |
					Selection::JustReg |
					Selection::JustIm8 => {
						let val: u8 = get_op_1!(self, &parsed);
					
						self.memory.set(self.sp.into(), val)?;
						self.sp -= 1;
					},

					_ => {
						return Err(CPUError::InvalidOpcode);
					}
				}
			},

			Opcode::POP => {
				match parsed.selection {
					Selection::RegReg |
					Selection::MemReg |
					Selection::RegMem => {
						set_op_1!(self, &parsed, self.memory.get(self.sp as usize)?);
						set_op_2!(self, &parsed, self.memory.get(self.sp as usize + 1)?);
						self.sp += 2;
					},

					Selection::JustMem |
					Selection::JustReg => {
						set_op_1!(self, &parsed, self.memory.get(self.sp.into())?);
						self.sp += 1;
					},

					_ => {
						return Err(CPUError::InvalidOpcode);
					}
				}
			},

			Opcode::JNZ => {
				if (self.flags & 0b1000_0000) == 0 {
					let jump_address: u16 = match parsed.selection {
						Selection::RegReg |
						Selection::RegIm8 |
						Selection::MemReg |
						Selection::RegMem => {
							let higher = get_op_1!(self, &parsed) as u16;
							let lower = get_op_2!(self, &parsed) as u16;
							
							(higher << 8) | lower
						},

						Selection::JustMem |
						Selection::JustReg |
						Selection::JustIm8 => {
							get_op_1!(self, &parsed) as u16
						}

						_ => {
							return Err(CPUError::InvalidOpcode);
						},
					};

					self.ip = jump_address;
					return Ok(());
				}
			},

			Opcode::HLT => {
				self.running = false;
			},

			Opcode::INT => {
				let n = get_op_1!(self, &parsed);
				let res = self.irq(n);

				if res.is_err() {
					return res;
				}
			}
		}

		self.ip += parsed.length as u16;
		Ok(())
	}

	fn irq(&mut self, int_number: u8) -> Result<(), CPUError> {
		self.memory.set(self.sp as usize, self.ip as u8)?;
		self.memory.set(self.sp as usize + 1, (self.ip >> 8) as u8)?;
		self.sp -= 2;
		
		return if let Some(iv) = self.ivt.get(int_number as usize) {
			self.ip = iv.address as u16;
			Ok(())
		}
		else {
			Err(CPUError::IVTAccessError(int_number.into()))
		}
	}

	fn set_flags(&mut self, value: u16) {
		let zf = if value == 0
			{ 0b1000_0000u8 } else { 0 };

		let cf = if value > u8::MAX as u16
			{ 0b0100_0000u8 } else { 0 };

		let sf = if value & 0b1000_0000 > 0
			{ 0b0010_0000u8 } else { 0 };

		self.flags &= 0b0001_1111;
		self.flags |= zf | cf | sf;
	}

	// Gets the lower/higher 8-bit part of any register.
	// NOT the full 16-bit part!
	fn get_register(&self, register: u8) -> u8 {
		let binding = [
			self.ip,
			self.sp,
			self.ax,
			self.bx,
			self.cx,
			self.dx,
			self.ex,
		];

		return if let Some(reg) = binding.get(register as usize / 2) {
			if register % 2 == 0 {
				*reg as u8
			}
			else {
				(*reg >> 8) as u8
			}
		} else if register == 0b1110 {
			self.flags
		} else {
			todo!();
		}
	}

	fn set_register(&mut self, register: u8, value: u8) {
		let higher = register % 2 == 0;
		let mask = if higher { 0xFF00u16 } else { 0x00FF };
		let converted_value = if higher { value as u16 } else { (value as u16) << 8 };

        match register {
            0b0000 |
            0b0001 => self.ip = (self.ip & mask) | converted_value,

            0b0010 |
            0b0011 => self.sp = (self.sp & mask) | converted_value,

            0b0100 |
            0b0101 => self.ax = (self.ax & mask) | converted_value,

            0b0110 |
            0b0111 => self.bx = (self.bx & mask) | converted_value,

            0b1000 |
            0b1001 => self.cx = (self.cx & mask) | converted_value,

            0b1010 |
            0b1011 => self.dx = (self.dx & mask) | converted_value,

            0b1100 |
            0b1101 => self.ex = (self.ex & mask) | converted_value,
            0b1110 => self.flags = value,
            _ => todo!()
        }
    }

	fn set_operand(&mut self, first_operand: bool, parsed: &Parser, value: u8) -> Result<(), CPUError> {
		match if first_operand { parsed.operand_1 } else { parsed.operand_2 } {
			Type::Register => {
				self.set_register(if first_operand { parsed.register_1 } else { parsed.register_2 }, value);
				Ok(())
			},
			Type::Constant => Err(CPUError::SetImmutableOperand),
			Type::Address => {
				self.memory.set(parsed.address.into(), value)
			},
			_ => Err(CPUError::SetImmutableOperand)
		}
	}

	fn get_operand(&self, first_operand: bool, parsed: &Parser) -> Result<u8, CPUError> {
		match if first_operand { parsed.operand_1 } else { parsed.operand_2 } {
			Type::Register => Ok(self.get_register(if first_operand { parsed.register_1 } else { parsed.register_2 })),
			Type::Constant => Ok(parsed.operand),
			Type::Address => {
				if let Some(accessee) = self.memory.data.get(parsed.address as usize) {
					Ok(*accessee)
				} else {
					Err(CPUError::MemoryAccessError(parsed.address as usize))
				}
			},
			_ => Err(CPUError::GetEmptyOperand),
		}
	}
}
