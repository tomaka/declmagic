use nalgebra::na::{ Eye, Mat4 };
use std::sync::{ Arc, Mutex };
use std::num;
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

	pub fn set_rectangle_coords(&mut self, leftCoord: Option<f32>, topCoord: Option<f32>, rightCoord: Option<f32>, bottomCoord: Option<f32>) {
		self.insideMatrix = Eye::new_identity(4);

		let heightToWidthRatio = (self.texture.get_height() as f32) / (self.texture.get_width() as f32);

		let (leftCoord, topCoord, rightCoord, bottomCoord) =
			match (leftCoord, topCoord, rightCoord, bottomCoord) {
				(Some(l), Some(t), Some(r), Some(b))
					=> (l, t, r, b),

				(Some(l), None, Some(r), Some(b))
					=> (l, b + heightToWidthRatio * num::abs(r - l), r, b),

				(Some(l), Some(t), Some(r), None)
					=> (l, t, r, t - heightToWidthRatio * num::abs(r - l)),

				(None, Some(t), Some(r), Some(b))
					=> (r - (t - b) / heightToWidthRatio, t, r, b),

				(Some(l), Some(t), None, Some(b))
					=> (l, t, l + (t - b) / heightToWidthRatio, b),

				(None, None, Some(r), Some(b))
					=> (-r, -b, r, b),

				(None, Some(t), None, Some(b))
					=> (-0.5 * num::abs(t - b) / heightToWidthRatio, t, 0.5 * num::abs(t - b) / heightToWidthRatio, b),

				(None, Some(t), Some(r), None)
					=> (-r, t, r, -t),

				(Some(l), None, None, Some(b))
					=> (l, -b, -l, b),

				(Some(l), None, Some(r), None)
					=> (l, 0.5 * num::abs(r - l) * heightToWidthRatio, r, -0.5 * num::abs(r - l) * heightToWidthRatio),

				(Some(l), Some(t), None, None)
					=> (l, t, -l, -t),

				(Some(l), None, None, None)
					=> (l, 0.5 * num::abs(l * 2.0) * heightToWidthRatio, -l, -0.5 * num::abs(l * 2.0) * heightToWidthRatio),

				(None, None, Some(r), None)
					=> (-r, 0.5 * num::abs(r * 2.0) * heightToWidthRatio, r, -0.5 * num::abs(r * 2.0) * heightToWidthRatio),

				(None, Some(t), None, None)
					=> (-0.5 * num::abs(t * 2.0) / heightToWidthRatio, t, 0.5 * num::abs(t * 2.0) / heightToWidthRatio, -t),

				(None, None, None, Some(b))
					=> (-0.5 * num::abs(b * 2.0) / heightToWidthRatio, -b, 0.5 * num::abs(b * 2.0) / heightToWidthRatio, b),

				(None, None, None, None)
					=> (-0.5, -0.5, 0.5, 0.5)
			};

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
