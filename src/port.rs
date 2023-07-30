pub struct Port {
	in_fn: Box<dyn Fn() -> u8>,
	out_fn: Box<dyn Fn(u8)>,
}

impl Port {
	pub fn new(in_fn: Box<dyn Fn() -> u8>, out_fn: Box<dyn Fn(u8)>) -> Self {
		Self {
			out_fn,
			in_fn,
		}
	}

	pub fn r#in(&self) -> u8 {
		(*self.in_fn)()
	}

	pub fn out(&self, value: u8) {
		(*self.out_fn)(value);
	}
}
