use crate::port::Port;

pub fn generate_serial_port() -> Port {
	Port::new(
		Box::new(serial_in),
		Box::new(serial_out)
	)
}

static mut BUFFER: Vec<u8> = vec![];

fn serial_in() -> u8 {
    unsafe {
        while BUFFER.is_empty() {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).expect("Failed to read input");
            BUFFER = input.chars().filter(|&c| c != '\r' && c != '\n').map(|c| c as u8).collect();
        }
        let byte = BUFFER[0];
        BUFFER.remove(0);
        byte
    }
}

fn serial_out(c: u8) {
	print!("{}", c as char);
}
