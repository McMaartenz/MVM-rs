mod computer;
mod parser;
mod memory;
mod disk;
mod port;
mod opcodes;
mod serial;
mod interrupts;

use std::path::Path;

use crate::computer::Computer;
use crate::disk::Disk;
use crate::serial::generate_serial_port;

fn main() {
    println!("Hello, world!");

    let disk = Disk::new(512, Path::new("a.bin"));
    let mut computer = Computer::new(1024, disk.unwrap());
    
    computer.io.insert(0, generate_serial_port());
    computer.boot();

    while computer.running {
        if let Err(tick) = computer.tick() {
            println!("Tick error: {:?}", tick);
        }
    }
}
