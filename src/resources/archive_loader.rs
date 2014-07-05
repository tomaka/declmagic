use std::io::{ IoResult, Reader };
use super::libarchive;
use super::ResourcesLoader;

#[deriving(Clone)]
pub struct MemoryArchiveLoader {
	data: &'static [u8]
}

pub struct MemoryResourceReader {
	archive: *mut libarchive::Struct_archive
}

impl MemoryArchiveLoader {
	pub fn new(data: &'static [u8]) -> MemoryArchiveLoader {
		MemoryArchiveLoader { data: data }
	}
}

impl ResourcesLoader for MemoryArchiveLoader {
	fn load(&self, resourceName: &str) -> IoResult<Box<Reader>> {
		let searchPath = Path::new(resourceName);

		unsafe {
			// TODO: use constants ARCHIVE_OK, ARCHIVE_EOF, etc.

			let archive = libarchive::archive_read_new();
			libarchive::archive_read_support_filter_all(archive);
			libarchive::archive_read_support_format_all(archive);

			if libarchive::archive_read_open_memory(archive, self.data.as_ptr() as *mut ::libc::c_void, self.data.len() as ::libc::size_t) != 0 {
				fail!("Unable to open resources");
			}

			loop {
				let mut entry: *mut libarchive::Struct_archive_entry = ::std::mem::uninitialized();

				if libarchive::archive_read_next_header(archive, &mut entry) != 0 {		// TODO: ARCHIVE_OK constant
					break;
				}
				if libarchive::archive_entry_mode(entry) == 40000 {       // TODO: AE_IFDIR constant
					continue;
				}

				let name = Path::new(::std::c_str::CString::new(libarchive::archive_entry_pathname(entry), false));

				match name.filestem().or_else(|| name.filename()) {
					None => (),
					Some(n) => 
						if name.dirname() == searchPath.dirname() && searchPath.filename().unwrap() == n {
							return Ok(box MemoryResourceReader{ archive: archive } as Box<Reader>);
						}
				}
				
				libarchive::archive_read_data_skip(archive);
			}

			return Err(::std::io::IoError{ kind: ::std::io::FileNotFound, desc: "Resource not found", detail: Some(format!("Resource {} was not found", resourceName)) });
		}
	}
}

impl Reader for MemoryResourceReader {
	fn read(&mut self, buf: &mut [u8]) -> IoResult<uint> {
		unsafe {
			let read = libarchive::archive_read_data(self.archive, buf.as_mut_ptr() as *mut ::libc::c_void, buf.len() as ::libc::size_t);

			if read < 0 {
				return fail!("Error while decompressing resource");
			}

			if read == 0 {
				return Err(::std::io::IoError{ kind: ::std::io::EndOfFile, desc: "End of file", detail: None });
			}

			Ok(read as uint)
		}
	}
}

impl Drop for MemoryResourceReader {
	fn drop(&mut self) {
		unsafe { libarchive::archive_read_free(self.archive); }
	}
}
