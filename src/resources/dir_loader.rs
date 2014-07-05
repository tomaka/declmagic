use std::io::{ IoResult, Reader };
use std::io::fs::File;
use std::path::Path;
use super::libarchive;
use super::ResourcesLoader;

pub struct DirLoader {
	directory: Path
}

impl DirLoader {
	pub fn new(directory: Path) -> DirLoader {
		DirLoader { directory: directory }
	}
}

impl ResourcesLoader for DirLoader {
	fn load(&self, resourceName: &str)
		-> IoResult<Box<Reader>>
	{
		let pathToSearch = self.directory.join(resourceName).join(".*");

		match ::glob::glob(format!("{}", pathToSearch.display()).as_slice()).next()
		{
			None =>
				Err(::std::io::IoError{
					kind: ::std::io::FileNotFound,
					desc: "Could not find resource",
					detail: None
				}),
				
			Some(file) =>
				Ok(box try!(File::open(&file)) as Box<Reader>)
		}
	}
}

