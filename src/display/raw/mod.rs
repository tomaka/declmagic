use gl;
use libc;
use std;

use std::collections::HashMap;
use std::sync::Arc;

mod context;
pub mod data_types;

pub enum WindowEvent {
	Moved(uint, uint),
	Resized(uint, uint),
	Closed,
	Input(::input::Message)
}

pub enum PrimitiveType {
	PointsList,
	LinesList,
	LineStrip,
	TrianglesList,
	TrianglesListAdjacency,
	TriangleStrip,
	TriangleStripAdjacency,
	TriangleFan
}

pub enum BlendingFunction {
	AlwaysReplace,
	LerpBySourceAlpha,
	LerpByDestinationAlpha
}

/**
 * Culling mode
 * Describes how triangles could be filtered before the fragment part
 */
pub enum BackfaceCullingMode {
	Disabled,
	CullCounterClockWise,		//< brief Cull counter-clockwise faces
	CullClockWise 				//< brief Cull clockwise faces
}

/**
 *
 */
pub enum SamplerWrapFunction {
	Repeat,
	Mirror,
	Clamp
}

/**
 *
 */
pub enum SamplerFilter {
	Nearest,
	Linear
}

/**
 * Type of shader
 */
pub enum ShaderType {
	Vertex,
	Geometry,
	Fragment
}

/**
 * Language of the shader
 */
pub enum ShaderLanguage {
	GLSL,
	HLSL,
	Cg
}

/**
 * Depth function
 */
pub enum DepthFunction {
	Ignore,
	Overwrite,
	IfEqual,
	IfNotEqual,
	IfMore,
	IfMoreOrEqual,
	IfLess,
	IfLessOrEqual
}

pub struct Display {
	context : Arc<context::GLContext>
}

pub struct Texture {
	texture: Arc<TextureImpl>
}

struct TextureImpl {
	display: Arc<context::GLContext>,
	id: gl::types::GLuint,
	bindPoint: gl::types::GLenum,
	width: uint,
	height: uint,
	depth: uint,
	arraySize: uint
}

pub struct Shader {
	shader: Arc<ShaderImpl>
}

struct ShaderImpl {
	display: Arc<context::GLContext>,
	id: gl::types::GLuint,
	shaderType: ShaderType
}

pub struct Program {
	display: Arc<context::GLContext>,
	shaders: Vec<Arc<ShaderImpl>>,
	id: gl::types::GLuint,
	uniforms: Arc<HashMap<String, (gl::types::GLint, gl::types::GLenum, gl::types::GLint)>>		// location, type and size of each uniform, ordered by name
}

#[deriving(Clone)]
pub struct ProgramUniforms {
	display: Arc<context::GLContext>,
	textures: HashMap<gl::types::GLint, Arc<TextureImpl>>,
	values: HashMap<gl::types::GLint, (gl::types::GLenum, Vec<char>)>,
	uniforms: Arc<HashMap<String, (gl::types::GLint, gl::types::GLenum, gl::types::GLint)>>		// same as the program's variable
}

pub struct VertexBuffer {
	display: Arc<context::GLContext>,
	id: gl::types::GLuint,
	elementsSize: uint,
	bindings: HashMap<String, (gl::types::GLenum, gl::types::GLint, gl::types::GLint)>			// for each binding, the data type, number of elems, and offset
}

pub struct IndexBuffer {
	display: Arc<context::GLContext>,
	id: gl::types::GLuint,
	elementsCount: uint,
	dataType: gl::types::GLenum,
	primitives: gl::types::GLenum
}

impl Display {
	pub fn new(width: uint, height: uint, title: &str) -> Display {
		Display {
			context: Arc::new(context::GLContext::new(width, height, title))
		}
	}

	pub fn recv(&self) -> Option<WindowEvent> {
		self.context.recv()
	}

	pub fn swap_buffers(&self) {
		self.context.swap_buffers();

		self.context.exec(proc() {
			gl::ClearColor(0.0, 0.0, 0.0, 1.0);
			gl::Clear(gl::COLOR_BUFFER_BIT);
		});
	}

	pub fn build_vertex_buffer1<T: data_types::GLDataTuple>(&self, data: &[(T)], bindings: &[&str])
		-> VertexBuffer
	{
		let elementsSize = std::mem::size_of_val(data.get(0).unwrap());
		let bufferSize = data.len() * elementsSize;

		let mut b = HashMap::new();
		for offset in range(0, bindings.len()) {
			let binding = bindings.get(offset).unwrap();
			match offset {
				0 => b.insert(binding.to_string(), ( data.get(0).unwrap().get_gl_type(), data.get(0).unwrap().get_num_elems() as i32, 0 )),
				_ => fail!()
			};
		}

		VertexBuffer {
			display: self.context.clone(),
			id: self.build_vertex_buffer(bufferSize, data.as_ptr() as *const libc::c_void),
			elementsSize: elementsSize,
			bindings: b
		}
	}

	pub fn build_vertex_buffer2<T1: data_types::GLDataTuple, T2: data_types::GLDataTuple>(&self, data: &[(T1, T2)], bindings: &[&str])
		-> VertexBuffer
	{
		let elementsSize = std::mem::size_of_val(data.get(0).unwrap().ref0()) + std::mem::size_of_val(data.get(0).unwrap().ref1());
		let bufferSize = data.len() * elementsSize;

		let mut b = HashMap::new();
		for offset in range(0, bindings.len()) {
			let binding = bindings.get(offset).unwrap();
			match offset {
				0 => b.insert(binding.to_string(), ( data.get(0).unwrap().ref0().get_gl_type(), data.get(0).unwrap().ref0().get_num_elems() as i32, 0 )),
				1 => b.insert(binding.to_string(), ( data.get(0).unwrap().ref1().get_gl_type(), data.get(0).unwrap().ref1().get_num_elems() as i32, data.get(0).unwrap().ref0().get_total_size() )),
				_ => fail!()
			};
		}

		VertexBuffer {
			display: self.context.clone(),
			id: self.build_vertex_buffer(bufferSize, data.as_ptr() as *const libc::c_void),
			elementsSize: elementsSize,
			bindings: b
		}
	}

	fn build_vertex_buffer(&self, bufferSize: uint, data: *const libc::c_void)
		-> gl::types::GLuint
	{
		self.context.exec(proc() {
    		unsafe {
    			let mut id: gl::types::GLuint = std::mem::uninitialized();
				gl::GenBuffers(1, &mut id);
	    		gl::BindBuffer(gl::ARRAY_BUFFER, id);
				gl::BufferData(gl::ARRAY_BUFFER, bufferSize as gl::types::GLsizeiptr, data, gl::STATIC_DRAW);
				id
			}
		}).get()
	}

	pub fn build_index_buffer<T: data_types::GLDataType>(&self, prim: PrimitiveType, data: &[T]) -> IndexBuffer {
		let elementsSize = std::mem::size_of_val(&data[0]);
		let dataSize = data.len() * elementsSize;
		let dataPtr: *const libc::c_void = data.as_ptr() as *const libc::c_void;

		let id = self.context.exec(proc() {
    		unsafe {
    			let id: gl::types::GLuint = std::mem::uninitialized();
				gl::GenBuffers(1, std::mem::transmute(&id));
	    		gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id);
				gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, dataSize as gl::types::GLsizeiptr, dataPtr, gl::STATIC_DRAW);
				id
			}
		}).get();

		IndexBuffer {
			display: self.context.clone(),
			id: id,
			elementsCount: data.len(),
			dataType: data[0].get_gl_type(),
			primitives: prim.get_gl_enum()
		}
	}

	pub fn build_shader(&self, language: ShaderLanguage, stype: ShaderType, sourceCode: &str, entryPoint: &str)
		-> Result<Shader, String>
	{
		match language {
			GLSL => (),
        	_ => return Err(format!("Only GLSL is supported"))
        };

		if entryPoint != "main" {
			return Err(format!("GLSL shaders entry point must be main"));
		}

		let srcCode = sourceCode.to_string();

		let idResult = self.context.exec(proc() {
    		unsafe {
    			let id = gl::CreateShader(stype.to_gl());

	    		gl::ShaderSource(id, 1, [ srcCode.to_c_str().unwrap() ].as_ptr(), std::ptr::null());
				gl::CompileShader(id);

			    let mut compilationSuccess: gl::types::GLint = std::mem::uninitialized();
			    gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut compilationSuccess);

    			if compilationSuccess == 0 {
    				let mut errorLogSize: gl::types::GLint = std::mem::uninitialized();
			        gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut errorLogSize);

			        let mut errorLog: Vec<u8> = Vec::with_capacity(errorLogSize as uint);
			        gl::GetShaderInfoLog(id, errorLogSize, &mut errorLogSize, errorLog.as_mut_slice().as_mut_ptr() as *mut gl::types::GLchar);

			        let msg = String::from_utf8(errorLog).unwrap();
    				return Err(msg)
    			}

    			Ok(id)
    		}
    	}).get();

    	idResult.map(|id| {
    		Shader {
    			shader: Arc::new(ShaderImpl {
	    			display: self.context.clone(),
	    			id: id,
	    			shaderType: stype
	    		})
    		}
    	})
	}

	pub fn build_texture<T: data_types::GLDataType>(&self, data: &[T], width: uint, height: uint, depth: uint, arraySize: uint)
		-> Texture
	{
		// TODO: restore when image format is supported
		/*if width * height * depth * arraySize != data.len() {
			fail!("Texture data has different size from width*height*depth*arraySize");
		}*/

		let textureType = if height == 1 && depth == 1 {
			if arraySize == 1 { gl::TEXTURE_1D } else { gl::TEXTURE_1D_ARRAY }
		} else if depth == 1 {
			if arraySize == 1 { gl::TEXTURE_2D } else { gl::TEXTURE_2D_ARRAY }
		} else {
			gl::TEXTURE_3D
		};

		let dataFormat = data[0].get_gl_type();
		let dataRaw: *const libc::c_void = unsafe { std::mem::transmute(data.as_ptr()) };

		let id = self.context.exec(proc() {
    		unsafe {
    			gl::PixelStorei(gl::UNPACK_ALIGNMENT, if width % 4 == 0 { 4 } else if height % 2 == 0 { 2 } else { 1 });

    			let id: gl::types::GLuint = std::mem::uninitialized();
				gl::GenTextures(1, std::mem::transmute(&id));

				gl::BindTexture(textureType, id);

	        	gl::TexParameteri(textureType, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
				if height != 1 || depth != 1 || arraySize != 1 {
					gl::TexParameteri(textureType, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
				}
				if depth != 1 || arraySize != 1 {
					gl::TexParameteri(textureType, gl::TEXTURE_WRAP_R, gl::REPEAT as i32);
				}
		        gl::TexParameteri(textureType, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
		        gl::TexParameteri(textureType, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as i32);

				if textureType == gl::TEXTURE_3D || textureType == gl::TEXTURE_2D_ARRAY {
					gl::TexImage3D(textureType, 0, gl::RGBA as i32, width as i32, height as i32, if depth > 1 { depth } else { arraySize } as i32, 0, gl::RGBA as u32, dataFormat, dataRaw);
				} else if (textureType == gl::TEXTURE_2D || textureType == gl::TEXTURE_1D_ARRAY) {
					gl::TexImage2D(textureType, 0, gl::RGBA as i32, width as i32, height as i32, 0, gl::RGBA as u32, dataFormat, dataRaw);
				} else {
					gl::TexImage1D(textureType, 0, gl::RGBA as i32, width as i32, 0, gl::RGBA as u32, dataFormat, dataRaw);
				}

				gl::GenerateMipmap(textureType);

				id
			}
		}).get();

		Texture {
			texture: Arc::new(TextureImpl {
				display: self.context.clone(),
				id: id,
				bindPoint: textureType,
				width: width,
				height: height,
				depth: depth,
				arraySize: arraySize
			})
		}
	}

	pub fn build_program(&self, shaders: &[&Shader])
		-> Result<Program, String>
	{
		let mut shadersIDs = Vec::new();
		for sh in shaders.iter() {
			shadersIDs.push(sh.shader.id);
		}

		let id = try!(self.context.exec(proc() {
			unsafe {
				let id = gl::CreateProgram();
				if id == 0 {
					return Err(format!("glCreateProgram failed"));
				}

				// attaching shaders
				for sh in shadersIDs.iter() {
					gl::AttachShader(id, sh.clone());
				}

				// linking and checking for errors
				gl::LinkProgram(id);
				{	let mut linkSuccess: gl::types::GLint = std::mem::uninitialized();
					gl::GetProgramiv(id, gl::LINK_STATUS, &mut linkSuccess);
					if (linkSuccess == 0) {
						match gl::GetError() {
							gl::NO_ERROR => (),
							gl::INVALID_VALUE => return Err(format!("glLinkProgram triggered GL_INVALID_VALUE")),
							gl::INVALID_OPERATION => return Err(format!("glLinkProgram triggered GL_INVALID_OPERATION")),
							_ => return Err(format!("glLinkProgram triggered an unknown error"))
						};

	    				let mut errorLogSize: gl::types::GLint = std::mem::uninitialized();
				        gl::GetProgramiv(id, gl::INFO_LOG_LENGTH, &mut errorLogSize);

				        let mut errorLog: Vec<u8> = Vec::with_capacity(errorLogSize as uint);
				        gl::GetProgramInfoLog(id, errorLogSize, &mut errorLogSize, errorLog.as_mut_slice().as_mut_ptr() as *mut gl::types::GLchar);
				        errorLog.set_len(errorLogSize as uint);

				        let msg = String::from_utf8(errorLog).unwrap();
	    				return Err(msg)
					}
				}

				Ok(id)
			}
		}).get());

		let uniforms = self.context.exec(proc() {
			unsafe {
				// reflecting program uniforms
				let mut uniforms = HashMap::new();

				let mut activeUniforms: gl::types::GLint = std::mem::uninitialized();
				gl::GetProgramiv(id, gl::ACTIVE_UNIFORMS, &mut activeUniforms);

				for uniformID in range(0, activeUniforms) {
			        let mut uniformNameTmp: Vec<u8> = Vec::with_capacity(64);
			        let mut uniformNameTmpLen = 63;

			        let mut dataType: gl::types::GLenum = std::mem::uninitialized();
			        let mut dataSize: gl::types::GLint = std::mem::uninitialized();
					gl::GetActiveUniform(id, uniformID as gl::types::GLuint, uniformNameTmpLen, &mut uniformNameTmpLen, &mut dataSize, &mut dataType, uniformNameTmp.as_mut_slice().as_mut_ptr() as *mut gl::types::GLchar);
					uniformNameTmp.set_len(uniformNameTmpLen as uint);

				    let uniformName = String::from_utf8(uniformNameTmp).unwrap();
					let location = gl::GetUniformLocation(id, uniformName.to_c_str().unwrap());

					uniforms.insert(uniformName, (location, dataType, dataSize));
				}

				Arc::new(uniforms)
			}
		}).get();


		let mut shadersStore = Vec::new();
		for sh in shaders.iter() {
			shadersStore.push(sh.shader.clone());
		}


		Ok(Program {
			display: self.context.clone(),
			shaders: shadersStore,
			id: id,
			uniforms: uniforms
		})
	}

	pub fn draw(&self, vertexBuffer: &VertexBuffer, indexBuffer: &IndexBuffer, program: &Program, uniforms: &ProgramUniforms)
	{
		let vbID = vertexBuffer.id.clone();
		let vbBindingsClone = vertexBuffer.bindings.clone();
		let vbElementsSize = vertexBuffer.elementsSize.clone();
		let ibID = indexBuffer.id.clone();
		let ibPrimitives = indexBuffer.primitives.clone();
		let ibElemCounts = indexBuffer.elementsCount.clone();
		let ibDataType = indexBuffer.dataType.clone();
		let programID = program.id.clone();
		let uniformsClone = uniforms.clone();

		self.context.exec(proc() {
			unsafe {
    			gl::Disable(gl::DEPTH_TEST);
    			gl::Enable(gl::BLEND);
    			gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

				// binding program
				gl::UseProgram(programID);

				// binding program uniforms
				{
					let mut activeTexture: uint = 0;
					for (&location, ref texture) in uniformsClone.textures.iter() {
						gl::ActiveTexture(gl::TEXTURE0 + activeTexture as u32);
						gl::BindTexture(texture.bindPoint, texture.id);
						gl::Uniform1i(location, activeTexture as i32);
						activeTexture = activeTexture + 1;
					}

					for (&location, &(ref datatype, ref data)) in uniformsClone.values.iter() {
						match *datatype {
							gl::FLOAT 		=> gl::Uniform1fv(location, 1, data.as_ptr() as *const f32),
							gl::FLOAT_MAT4 	=> gl::UniformMatrix4fv(location, 1, 0, data.as_ptr() as *const f32),
							_ => fail!("Loading uniforms for this type not implemented")
						}
						//gl::Uniform1i(location, activeTexture as i32);
					}
				}

				// binding buffers
			    gl::BindBuffer(gl::ARRAY_BUFFER, vbID);
			    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibID);

			    // binding vertex buffer
			    let mut locations = Vec::new();
			    for (name, &(dataType, dataSize, dataOffset)) in vbBindingsClone.iter() {
			    	let loc = gl::GetAttribLocation(programID, name.to_c_str().unwrap());
			    	locations.push(loc);

					if loc != -1 {
						match dataType {
							gl::BYTE | gl::UNSIGNED_BYTE | gl::SHORT | gl::UNSIGNED_SHORT | gl::INT | gl::UNSIGNED_INT
								=> gl::VertexAttribIPointer(loc as u32, dataSize, dataType, vbElementsSize as i32, dataOffset as *const libc::c_void),
							_ => gl::VertexAttribPointer(loc as u32, dataSize, dataType, 0, vbElementsSize as i32, dataOffset as *const libc::c_void)
						}
						
						gl::EnableVertexAttribArray(loc as u32);
					}
			    }
			    
			    // drawing
			    gl::DrawElements(ibPrimitives, ibElemCounts as i32, ibDataType, std::ptr::null());

			    // disable vertex attrib array
			    for l in locations.iter() {
			    	gl::DisableVertexAttribArray(l.clone() as u32);
			    }
			}
		}).get();
	}
}

impl PrimitiveType {
	fn get_gl_enum(&self) -> gl::types::GLenum {
		match *self {
			PointsList => gl::POINTS,
			LinesList => gl::LINES,
			LineStrip => gl::LINE_STRIP,
			TrianglesList => gl::TRIANGLES,
			TrianglesListAdjacency => gl::TRIANGLES_ADJACENCY,
			TriangleStrip => gl::TRIANGLE_STRIP,
			TriangleStripAdjacency => gl::TRIANGLE_STRIP_ADJACENCY,
			TriangleFan => gl::TRIANGLE_FAN
		}
	}
}

impl Program {
	pub fn build_uniforms(&self)
		-> ProgramUniforms
	{
		ProgramUniforms {
			display: self.display.clone(),
			textures: HashMap::new(),
			values: HashMap::new(),
			uniforms: self.uniforms.clone()
		}
	}
}

impl ProgramUniforms {
	pub fn set_value<T: data_types::UniformValue>(&mut self, uniformName: &str, value: T) {
		let &(location, gltype, typesize) = match self.uniforms.find(&uniformName.to_string()) {
			Some(a) => a,
			None => return		// the uniform is not used, we ignore it
		};

		if gltype != value.get_gl_type() {
			fail!("Type of data passed to set_value must match the type of data requested by the shader")
		}

		let mut data: Vec<char> = Vec::with_capacity(std::mem::size_of_val(&value));
		unsafe { data.set_len(std::mem::size_of_val(&value)); }

		let mut dataInside = data.as_mut_ptr() as *mut T;
		unsafe { (*dataInside) = value; }

		self.values.insert(location.clone(), (gltype, data));
	}

	pub fn set_texture(&mut self, uniformName: &str, texture: &Texture) {
		let &(location, gltype, typesize) = match self.uniforms.find(&uniformName.to_string()) {
			Some(a) => a,
			None => return		// the uniform is not used, we ignore it
		};

		match gltype {
			gl::SAMPLER_1D | gl::SAMPLER_2D | gl::SAMPLER_3D | gl::SAMPLER_CUBE | gl::SAMPLER_1D_SHADOW | gl::SAMPLER_2D_SHADOW | gl::SAMPLER_1D_ARRAY | gl::SAMPLER_2D_ARRAY | gl::SAMPLER_1D_ARRAY_SHADOW | gl::SAMPLER_2D_ARRAY_SHADOW | gl::SAMPLER_2D_MULTISAMPLE | gl::SAMPLER_2D_MULTISAMPLE_ARRAY | gl::SAMPLER_CUBE_SHADOW | gl::SAMPLER_BUFFER | gl::SAMPLER_2D_RECT | gl::SAMPLER_2D_RECT_SHADOW | gl::INT_SAMPLER_1D | gl::INT_SAMPLER_2D | gl::INT_SAMPLER_3D | gl::INT_SAMPLER_CUBE | gl::INT_SAMPLER_1D_ARRAY | gl::INT_SAMPLER_2D_ARRAY | gl::INT_SAMPLER_2D_MULTISAMPLE | gl::INT_SAMPLER_2D_MULTISAMPLE_ARRAY | gl::INT_SAMPLER_BUFFER | gl::INT_SAMPLER_2D_RECT | gl::UNSIGNED_INT_SAMPLER_1D | gl::UNSIGNED_INT_SAMPLER_2D | gl::UNSIGNED_INT_SAMPLER_3D | gl::UNSIGNED_INT_SAMPLER_CUBE | gl::UNSIGNED_INT_SAMPLER_1D_ARRAY | gl::UNSIGNED_INT_SAMPLER_2D_ARRAY | gl::UNSIGNED_INT_SAMPLER_2D_MULTISAMPLE | gl::UNSIGNED_INT_SAMPLER_2D_MULTISAMPLE_ARRAY | gl::UNSIGNED_INT_SAMPLER_BUFFER | gl::UNSIGNED_INT_SAMPLER_2D_RECT => (),
			_ => fail!("Trying to bind a texture to a non-texture uniform")
		};

		self.textures.insert(location.clone(), texture.texture.clone());
	}
}

impl ShaderType {
	fn to_gl(&self) -> gl::types::GLenum {
		match *self {
			Vertex => gl::VERTEX_SHADER,
			Geometry => gl::GEOMETRY_SHADER,
			Fragment => gl::FRAGMENT_SHADER
		}
	}
}

impl Drop for TextureImpl {
	fn drop(&mut self) {
		let id = self.id.clone();
		self.display.exec(proc() {
			unsafe { gl::DeleteTextures(1, [ id ].as_ptr()); }
		});
	}
}

impl Drop for VertexBuffer {
	fn drop(&mut self) {
		let id = self.id.clone();
		self.display.exec(proc() {
			unsafe { gl::DeleteBuffers(1, [ id ].as_ptr()); }
		});
	}
}

impl Drop for IndexBuffer {
	fn drop(&mut self) {
		let id = self.id.clone();
		self.display.exec(proc() {
			unsafe { gl::DeleteBuffers(1, [ id ].as_ptr()); }
		});
	}
}

impl Drop for ShaderImpl {
	fn drop(&mut self) {
		let id = self.id.clone();
		self.display.exec(proc() {
			unsafe { gl::DeleteShader(id); }
		});
	}
}
