use gl;
use std;

pub trait GLDataType: Num + Copy {
	fn get_gl_type(&self) -> gl::types::GLenum;
}

impl GLDataType for i8 {
	fn get_gl_type(&self) -> gl::types::GLenum {
		gl::BYTE
	}
}

impl GLDataType for u8 {
	fn get_gl_type(&self) -> gl::types::GLenum {
		gl::UNSIGNED_BYTE
	}
}

impl GLDataType for i16 {
	fn get_gl_type(&self) -> gl::types::GLenum {
		gl::SHORT
	}
}

impl GLDataType for u16 {
	fn get_gl_type(&self) -> gl::types::GLenum {
		gl::UNSIGNED_SHORT
	}
}

impl GLDataType for f32 {
	fn get_gl_type(&self) -> gl::types::GLenum {
		gl::FLOAT
	}
}

impl GLDataType for f64 {
	fn get_gl_type(&self) -> gl::types::GLenum {
		gl::DOUBLE
	}
}

pub trait GLDataTuple {
	fn get_gl_type(&self) -> gl::types::GLenum;
	fn get_num_elems(&self) -> uint;
	fn get_total_size(&self) -> gl::types::GLsizei;
}

impl GLDataTuple for (f32) {
	fn get_gl_type(&self) -> gl::types::GLenum { gl::FLOAT }
	fn get_num_elems(&self) -> uint { 1 }
	fn get_total_size(&self) -> gl::types::GLsizei { std::mem::size_of_val(self) as gl::types::GLsizei }
}

impl GLDataTuple for (f32, f32) {
	fn get_gl_type(&self) -> gl::types::GLenum { gl::FLOAT }
	fn get_num_elems(&self) -> uint { 2 }
	fn get_total_size(&self) -> gl::types::GLsizei { 2 * std::mem::size_of_val(self.ref0()) as gl::types::GLsizei }
}

impl GLDataTuple for (f32, f32, f32) {
	fn get_gl_type(&self) -> gl::types::GLenum { gl::FLOAT }
	fn get_num_elems(&self) -> uint { 3 }
	fn get_total_size(&self) -> gl::types::GLsizei { 3 * std::mem::size_of_val(self.ref0()) as gl::types::GLsizei }
}

impl GLDataTuple for (f32, f32, f32, f32) {
	fn get_gl_type(&self) -> gl::types::GLenum { gl::FLOAT }
	fn get_num_elems(&self) -> uint { 4 }
	fn get_total_size(&self) -> gl::types::GLsizei { 4 * std::mem::size_of_val(self.ref0()) as gl::types::GLsizei }
}


pub trait UniformValue {
	fn get_gl_type(&self) -> gl::types::GLenum;
}

impl UniformValue for i8 {
	fn get_gl_type(&self) -> gl::types::GLenum {
		gl::BYTE
	}
}

impl UniformValue for u8 {
	fn get_gl_type(&self) -> gl::types::GLenum {
		gl::UNSIGNED_BYTE
	}
}

impl UniformValue for i16 {
	fn get_gl_type(&self) -> gl::types::GLenum {
		gl::SHORT
	}
}

impl UniformValue for u16 {
	fn get_gl_type(&self) -> gl::types::GLenum {
		gl::UNSIGNED_SHORT
	}
}

impl UniformValue for f32 {
	fn get_gl_type(&self) -> gl::types::GLenum {
		gl::FLOAT
	}
}

impl UniformValue for f64 {
	fn get_gl_type(&self) -> gl::types::GLenum {
		gl::DOUBLE
	}
}

impl UniformValue for ::nalgebra::na::Mat2<f32> {
	fn get_gl_type(&self) -> gl::types::GLenum {
		gl::FLOAT_MAT2
	}
}

impl UniformValue for ::nalgebra::na::Mat3<f32> {
	fn get_gl_type(&self) -> gl::types::GLenum {
		gl::FLOAT_MAT3
	}
}

impl UniformValue for ::nalgebra::na::Mat4<f32> {
	fn get_gl_type(&self) -> gl::types::GLenum {
		gl::FLOAT_MAT4
	}
}
