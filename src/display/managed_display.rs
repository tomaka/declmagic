
use super::raw::Display;
use super::raw::Texture;
use std::io::Reader;
use std::any::Any;
use std::sync::Arc;
use std::string::String;
use std::collections::HashMap;
use std::sync::Mutex;
use resources::ResourcesLoader;

pub struct ManagedDisplay {
	display: Display,
	loader: Box<ResourcesLoader+Send+Share>,
	textures: Mutex<HashMap<String, ::std::sync::Arc<super::raw::Texture>>>
}

pub struct Texture {
	texture: ::std::sync::Arc<super::raw::Texture>
}

impl ManagedDisplay {
	pub fn new(display: Display, loader: Box<ResourcesLoader+Send+Share>)
		-> ManagedDisplay
	{
		ManagedDisplay {
			display: display,
			loader: loader,
			textures: Mutex::new(HashMap::new())
		}
	}

	pub fn load_texture(&self, name: &str)
		-> Result<Texture, String>
	{
		let mut lock = self.textures.lock();

		match lock.find(&String::from_str(name)) {
			Some(v) => return Ok(Texture { texture: v.clone() }),
			_ => ()
		};

		let mut stream = match self.loader.load(name) {
			Ok(v) => v,
			Err(e) => return Err(format!("{}", e))
		};

		let data = match stream.read_to_end() {
			Ok(d) => d,
			Err(e) => return Err(format!("{}", e))
		};

		let texture = Arc::new(match ::stb_image::image::load_from_memory(data.as_slice()) {
			::stb_image::image::Error(s) => return Err(format!("load_from_memory failed")),
			::stb_image::image::ImageU8(img) => {
				//let data: &[u32] = ::std::mem::transmute(img.data.as_slice());
				self.display.build_texture(img.data.as_slice(), img.width, img.height, 1, 1)		// TODO: image depth not taken into account
			},
    		::stb_image::image::ImageF32(img) => self.display.build_texture(img.data.as_slice(), img.width, img.height, 1, 1)
		});

		lock.insert(name.to_string(), texture.clone());
		Ok(Texture { texture: texture })
	}


	pub fn recv(&self)
		-> Option<super::raw::WindowEvent>
	{
		self.display.recv()
	}

	pub fn swap_buffers(&self) {
		self.display.swap_buffers()
	}

	pub fn build_vertex_buffer1<T: super::raw::data_types::GLDataTuple>(&self, data: &[(T)], bindings: &[&str])
		-> super::raw::VertexBuffer
	{
		self.display.build_vertex_buffer1(data, bindings)
	}

	pub fn build_vertex_buffer2<T1: super::raw::data_types::GLDataTuple, T2: super::raw::data_types::GLDataTuple>(&self, data: &[(T1, T2)], bindings: &[&str])
		-> super::raw::VertexBuffer
	{
		self.display.build_vertex_buffer2(data, bindings)
	}

	pub fn build_index_buffer<T: super::raw::data_types::GLDataType>(&self, prim: super::raw::PrimitiveType, data: &[T])
		-> super::raw::IndexBuffer
	{
		self.display.build_index_buffer(prim, data)
	}

	pub fn build_shader(&self, language: super::raw::ShaderLanguage, stype: super::raw::ShaderType, sourceCode: &str, entryPoint: &str)
		-> Result<super::raw::Shader, String>
	{
		self.display.build_shader(language, stype, sourceCode, entryPoint)
	}

	pub fn build_program(&self, shaders: &[&super::raw::Shader])
		-> Result<super::raw::Program, String>
	{
		self.display.build_program(shaders)
	}

	pub fn draw(&self, vertexBuffer: &super::raw::VertexBuffer, indexBuffer: &super::raw::IndexBuffer, program: &super::raw::Program, uniforms: &super::raw::ProgramUniforms)
	{
		self.display.draw(vertexBuffer, indexBuffer, program, uniforms)
	}
}

impl Deref<super::raw::Texture> for Texture {
	fn deref<'a>(&'a self) -> &'a super::raw::Texture {
		self.texture.deref()
	}
}
