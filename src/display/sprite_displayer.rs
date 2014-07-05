use nalgebra::na::{ Eye, Mat4 };
use std::sync::{ Arc, Mutex };
use super::managed_display::{ ManagedDisplay, Texture };
use super::Drawable;

static vertexShader: &'static str = "
#version 110

uniform mat4 uMatrix;

attribute vec2 iPosition;
attribute vec2 iTexCoords;

varying vec2 vTexCoords;

void main() {
	gl_Position = vec4(iPosition, 0.0, 1.0) * uMatrix;
	vTexCoords = iTexCoords;
}
";

static fragmentShader: &'static str = "
#version 110
uniform sampler2D uTexture;
varying vec2 vTexCoords;

void main() {
	gl_FragColor = texture2D(uTexture, vTexCoords);
}
";

pub struct SpriteDisplayer {
	display: Arc<ManagedDisplay>,
	insideMatrix: Mat4<f32>,
	texture: Texture,
	vertexBuffer: super::raw::VertexBuffer,
	indexBuffer: super::raw::IndexBuffer,
	program: super::raw::Program,
	uniforms: Mutex<super::raw::ProgramUniforms>
}

impl SpriteDisplayer {
	pub fn new(display: Arc<ManagedDisplay>, resourceName: &str) -> Result<SpriteDisplayer, String> {
		let texture = try!(display.load_texture(resourceName));

		let vs = display.build_shader(super::raw::GLSL, super::raw::Vertex, vertexShader, "main").unwrap();
		let fs = display.build_shader(super::raw::GLSL, super::raw::Fragment, fragmentShader, "main").unwrap();
		let program = display.build_program(&[ &vs, &fs ]).unwrap();
		let mut uniforms = program.build_uniforms();

		let vb = display.build_vertex_buffer2(
			&[
				( (-1.0 as f32, -1.0 as f32), (0.0 as f32, 1.0 as f32) ),
				( (-1.0 as f32,  1.0 as f32), (0.0 as f32, 0.0 as f32) ),
				( ( 1.0 as f32,  1.0 as f32), (1.0 as f32, 0.0 as f32) ),
				( ( 1.0 as f32, -1.0 as f32), (1.0 as f32, 1.0 as f32) )
			],
			&[ "iPosition", "iTexCoords" ]
		);
		let ib = display.build_index_buffer(super::raw::TrianglesList, &[ 0 as u16, 1, 2, 0, 2, 3 ]);

		uniforms.set_texture("uTexture", texture.deref());

		Ok(SpriteDisplayer {
			display: display,
			insideMatrix: Eye::new_identity(4),
			texture: texture,
			vertexBuffer: vb,
			indexBuffer: ib,
			program: program,
			uniforms: Mutex::new(uniforms)
		})
	}

	pub fn set_rectangle_coords(&mut self, leftCoord: f32, topCoord: f32, rightCoord: f32, bottomCoord: f32) {
		self.insideMatrix = Eye::new_identity(4);
		self.insideMatrix.m11 = (rightCoord - leftCoord) / 2.0;
		self.insideMatrix.m41 = (rightCoord + leftCoord) / 2.0;
		self.insideMatrix.m22 = (topCoord - bottomCoord) / 2.0;
		self.insideMatrix.m42 = (topCoord + bottomCoord) / 2.0;
	}

	pub fn set_resource(&mut self, resourceName: &str)
	{
		self.texture = self.display.load_texture(resourceName).unwrap(); 
	}
}

impl Drawable for SpriteDisplayer {
	fn draw(&self, matrix: &Mat4<f32>) {
		let mut uniforms = self.uniforms.lock();
		uniforms.set_value("uMatrix", self.insideMatrix * matrix.clone());
		self.display.draw(&self.vertexBuffer, &self.indexBuffer, &self.program, uniforms.deref());
	}
}
