use entities::{ EntitiesState, EntitiesHelper, EntityID, ComponentID, NativeComponentType };
use nalgebra::na::{ Mat4, Vec3, Eye };

use std::collections::{ HashSet, HashMap };
use std::sync::{ Arc, Mutex };

use super::super::managed_display::ManagedDisplay;
use super::super::sprite_displayer::SpriteDisplayer;
use super::super::Drawable;
use super::super::raw::{ VertexBuffer, IndexBuffer, Shader, Program, ProgramUniforms };

use log::Logger;

pub struct CustomDisplaySystem {
	display: Arc<ManagedDisplay>,
	elements: HashMap<ComponentID, Element>,
	logger: Box<Logger>
}

struct Element {
	vertexShaderSrc: String,
	vertexShader: Option<Shader>,
	fragmentShaderSrc: String,
	fragmentShader: Option<Shader>,
	program: Option<Program>,
	vertexBuffer: Option<VertexBuffer>,
	indexBuffer: Option<IndexBuffer>,
	uniforms: Option<Mutex<ProgramUniforms>>
}

impl CustomDisplaySystem {
	pub fn new<L: ::log::Logger + 'static>(display: Arc<ManagedDisplay>, _: &EntitiesState, mut logger: L)
		-> CustomDisplaySystem
	{
		CustomDisplaySystem {
			display: display.clone(),
			elements: HashMap::new(),
			logger: box logger
		}
	}

	pub fn draw(&mut self, state: &EntitiesState)
	{
		self.update_custom_displayers(state);

		let camera = super::DisplaySystem::get_camera(state).unwrap_or_else(|| {
			declmagic_warn!(self.logger, "no active camera on the scene");
			Eye::new_identity(4)
		});

	 	for (cmp, element) in self.elements.mut_iter() {
	 		let pos = ::physics::PhysicsSystem::get_entity_position(state, &state.get_owner(cmp).unwrap());
	 		let translationMatrix = Mat4::new(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, pos.x, pos.y, pos.z, 1.0);
	 		let finalMatrix = translationMatrix * camera;

	 		if element.uniforms.is_some() {
		 		for uniName in match state.get(cmp, "matrixUniforms") { Ok(&::entities::List(ref l)) => l.clone(), _ => Vec::new() }.move_iter() {
		 			let mut uniLocked = element.uniforms.as_ref().unwrap().lock();

		 			match uniName {
		 				::entities::String(s) =>
							uniLocked.set_value(s.as_slice(), finalMatrix),
						_ => ()
		 			}
				}
			}

			match (&element.vertexBuffer, &element.indexBuffer, &element.program, &element.uniforms) {
				(&Some(ref vb), &Some(ref ib), &Some(ref prg), &Some(ref uni)) => {
					let uniLocked = uni.lock();
					self.display.draw(vb, ib, prg, &*uniLocked)
				},
				_ => ()
			}
		}
	}

	fn update_custom_displayers(&mut self, state: &EntitiesState)
	{
		// getting the list of all sprite displayer components
		let listOfComponents = state.get_components_iter()
            .filter(|c| state.is_component_visible(*c).unwrap())
			.filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "customDisplay", _ => false })
			.map(|c| c.clone())
			.collect::<HashSet<ComponentID>>();

		// removing from the list the elements that have disappeared
		{	let toRemove = self.elements.keys().filter(|e| !listOfComponents.contains(e.clone())).map(|e| e.clone()).collect::<Vec<ComponentID>>();
			for c in toRemove.move_iter() {
				self.elements.remove(&c);
			}
		}

		// adding elements that are not yet created
		{	let toCreate = listOfComponents.iter().filter(|e| !self.elements.contains_key(e.clone())).map(|e| e.clone()).collect::<Vec<ComponentID>>();
			for cmp in toCreate.move_iter() {
				let mut element = Element {
					vertexShaderSrc: "".to_string(),
					vertexShader: None,
					fragmentShaderSrc: "".to_string(),
					fragmentShader: None,
					program: None,
					vertexBuffer: None,
					indexBuffer: None,
					uniforms: None
				};

				element.vertexBuffer = Some(self.display.build_vertex_buffer2(
					&[
						( (-1.0 as f32, -1.0 as f32), (0.0 as f32, 1.0 as f32) ),
						( (-1.0 as f32,  1.0 as f32), (0.0 as f32, 0.0 as f32) ),
						( ( 1.0 as f32,  1.0 as f32), (1.0 as f32, 0.0 as f32) ),
						( ( 1.0 as f32, -1.0 as f32), (1.0 as f32, 1.0 as f32) )
					],
					&[ "iPosition", "iTexCoords" ]
				));
				element.indexBuffer = Some(self.display.build_index_buffer(super::super::raw::TrianglesList, &[ 0 as u16, 1, 2, 0, 2, 3 ]));


				// inserting in sprites list
				self.elements.insert(cmp.clone(), element);
			}
		}

		// updating everything
		for (component, element) in self.elements.mut_iter() {
			let vertexShader = match state.get_as_string(component, "vertexShader") { Some(s) => s, _ => continue };
			let fragmentShader = match state.get_as_string(component, "fragmentShader") { Some(s) => s, _ => continue };

			let mut recompilePrograms = false;

			if vertexShader != element.vertexShaderSrc {
				element.vertexShader = self.display.build_shader(super::super::raw::GLSL, super::super::raw::Vertex, vertexShader.as_slice(), "main").ok();
				recompilePrograms = true;
			}

			if fragmentShader != element.fragmentShaderSrc {
				element.fragmentShader = self.display.build_shader(super::super::raw::GLSL, super::super::raw::Fragment, fragmentShader.as_slice(), "main").ok();
				recompilePrograms = true;
			}

			if recompilePrograms {
				match (&element.vertexShader, &element.fragmentShader) {
					(&Some(ref vs), &Some(ref fs)) =>	{
						element.program = self.display.build_program(&[ vs, fs ]).ok();
						match &element.program {
							&Some(ref p) => element.uniforms = Some(Mutex::new(p.build_uniforms())),
							&None => element.uniforms = None
						};
					},
					_ => ()
				}
			}

		}
	}
}
