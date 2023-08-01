pub struct InterruptVec {
	pub address: usize
}

#[repr(u8)]
pub enum Interrupt {
	DivideByZero,
	InvalidOpcode = 0x06,
	MachineCheck = 0x12,
}
