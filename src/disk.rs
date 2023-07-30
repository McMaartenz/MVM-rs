use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Disk {
	pub path: PathBuf,
	pub data: Vec<u8>
}

#[derive(Debug)]
pub enum DiskReadError {
	NoSuchFile
}

impl Disk {
	pub fn new(size: usize, path: &Path) -> Result<Self, DiskReadError> {
		let path_buf = path.to_path_buf();
		let mut data = vec![0u8; size];

		return if let Ok(file) = File::open(path) {
			let mut handle = file.take(512);
			let _ = handle.read(data.as_mut_slice());
	
			Ok(Self {
				path: path_buf,
				data
			})
		} else {
			Err(DiskReadError::NoSuchFile)
		}
	}
}
