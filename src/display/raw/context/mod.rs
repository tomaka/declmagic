extern crate std;

use std::sync::{ Arc, Future, Mutex };
use std::rc::Rc;
use threaded_executer::CommandsThread;
use std::string::String;

#[allow(dead_code)]
mod libglfw3;
mod window;

pub struct GLContext {
	window: Mutex<window::Window>
}

impl GLContext {
	pub fn new(width: uint, height: uint, title: &str) -> GLContext {
		let window = window::Window::new(width, height, title);
		window.make_context_current();

		window.exec(proc() {
			::gl::load_with(|s| unsafe { std::mem::transmute(libglfw3::glfwGetProcAddress(s.to_c_str().unwrap())) });
		}).get();

		GLContext {
			window: Mutex::new(window)
		}
	}

	pub fn recv(&self) -> Option<super::WindowEvent> {
		let mut lock = self.window.lock();
		lock.recv()
	}

	pub fn exec<T:Send>(&self, f: proc(): Send -> T) -> Future<T> {
		let mut lock = self.window.lock();
		lock.exec(f)
	}

	pub fn swap_buffers(&self) {
		let mut lock = self.window.lock();
		lock.swap_buffers()
	}
}
