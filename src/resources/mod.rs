extern crate glob;
extern crate std;

use std::io::{ IoResult, Reader };

#[allow(dead_code)]
mod libarchive;
pub mod archive_loader;
pub mod dir_loader;

pub trait ResourcesLoader : Clone {
	fn load(&self, resourceName: &str) -> IoResult<Box<Reader>>;
}
