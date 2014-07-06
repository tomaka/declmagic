use nalgebra::na::Mat4;

pub use self::system::DisplaySystem;

pub mod managed_display;
#[allow(dead_code)]
pub mod raw;
pub mod sprite_displayer;
mod system;

pub trait Drawable {
	fn draw(&self, matrix: &Mat4<f32>);
}
