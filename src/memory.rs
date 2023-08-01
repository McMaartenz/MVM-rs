use crate::computer::CPUError;

pub struct Memory {
	pub size: usize,
	pub data: Vec<u8>
}

impl Memory {
	pub fn new(size: usize) -> Self {
		let data = vec![0; size];
		
		Self {
			size,
			data
		}
	}

	pub fn get(&self, address: usize) -> Result<u8, CPUError> {
		return if address >= self.size {
			Err(CPUError::MemoryAccessError(address))
		} else {
			Ok(self.data[address])
		}
	}

	pub fn set(&mut self, address: usize, value: u8) -> Result<(), CPUError> {
		return if address >= self.size {
			Err(CPUError::MemoryAccessError(address))
		} else {
			self.data[address] = value;
			Ok(())
		}
	}
}
