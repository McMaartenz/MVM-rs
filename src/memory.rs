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
}
