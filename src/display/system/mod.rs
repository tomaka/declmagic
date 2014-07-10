use super::managed_display::ManagedDisplay;
use entities::{ EntitiesState, EntitiesHelper, EntityID, ComponentID, NativeComponentType };
use nalgebra::na;
use nalgebra::na::{ Vec3, Eye };
use std::collections::{ HashSet, HashMap };
use std::sync::Arc;
use super::sprite_displayer::SpriteDisplayer;
use super::Drawable;

mod custom_display_system;

pub struct DisplaySystem {
	display: Arc<ManagedDisplay>,
	customDisplay: custom_display_system::CustomDisplaySystem,
	sprites: HashMap<ComponentID, (Option<SpriteDisplayer>, String)>,
	logger: Box<::log::Logger>
}

impl DisplaySystem {
	pub fn new<L: ::log::Logger + 'static>(display: Arc<ManagedDisplay>, state: &EntitiesState, mut logger: L)
		-> DisplaySystem
	{
		declmagic_info!(logger, "created display system");

		DisplaySystem {
			display: display.clone(),
			customDisplay: custom_display_system::CustomDisplaySystem::new(display.clone(), state, logger.clone()),
			sprites: HashMap::new(),
			logger: box logger
		}
	}

	pub fn draw(&mut self, state: &EntitiesState, _: &u64)
	{
		self.update_sprite_displayers(state);

		let camera = DisplaySystem::get_camera(state).unwrap_or_else(|| {
			declmagic_warn!(self.logger, "no active camera on the scene");
			Eye::new_identity(4)
		});

	 	for (cmp, &(ref sprite, _)) in self.sprites.iter() {
	 		let pos = ::physics::PhysicsSystem::get_entity_position(state, &state.get_owner(cmp).unwrap());
	 		let translationMatrix = na::Mat4::new(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, pos.x, pos.y, pos.z, 1.0);
			sprite.as_ref().map(|e| e.draw(&(translationMatrix * camera)));
		}

		self.customDisplay.draw(state);
	}

	fn update_sprite_displayers(&mut self, state: &EntitiesState)
	{
		// getting the list of all sprite displayer components
		let listOfComponents = state.get_components_iter()
            .filter(|c| state.is_component_visible(*c).unwrap())
			.filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "spriteDisplay", _ => false })
			.map(|c| c.clone())
			.collect::<HashSet<ComponentID>>();

		// removing from the list the elements that have disappeared
		{	let toRemove = self.sprites.keys().filter(|e| !listOfComponents.contains(e.clone())).map(|e| e.clone()).collect::<Vec<ComponentID>>();
			for c in toRemove.move_iter() {
				self.sprites.remove(&c);
			}
		}

		// adding elements that are not yet created
		{	let toCreate = listOfComponents.iter().filter(|e| !self.sprites.contains_key(e.clone())).map(|e| e.clone()).collect::<Vec<ComponentID>>();
			for spriteDisplayComponent in toCreate.move_iter() {
				// inserting in sprites list
				self.sprites.insert(spriteDisplayComponent.clone(), (None, format!("")));
			}
		}

		// updating everything
		for (component, &(ref mut sprite, ref mut currTexName)) in self.sprites.mut_iter() {
			// getting the name of the texture
			let textureName = match state.get_as_string(component, "texture") {
				Some(s) => s,
				_ => {
					// TODO: 
					//declmagic_error!(self.logger, "component {} has no valid \"texture\" element", component)
					continue
				}
			};

			if sprite.is_none() || currTexName.as_slice() != textureName.as_slice() {
				(*sprite) = Some(SpriteDisplayer::new(self.display.clone(), textureName.as_slice()).unwrap());
			}

			// getting coordinates
			match sprite {
				&Some(ref mut s) =>
					s.set_rectangle_coords(
						match state.get(component, "leftX") { Ok(&::entities::Number(ref nb)) => Some(*nb as f32), _ => None },
						match state.get(component, "topY") { Ok(&::entities::Number(ref nb)) => Some(*nb as f32), _ => None },
						match state.get(component, "rightX") { Ok(&::entities::Number(ref nb)) => Some(*nb as f32), _ => None },
						match state.get(component, "bottomY") { Ok(&::entities::Number(ref nb)) => Some(*nb as f32), _ => None }
					),
				_ => fail!()
			}
		}
	}

	/// Returns the camera matrix of the scene.
	pub fn get_camera(state: &EntitiesState)
		-> Option<na::Mat4<f32>>
	{
		let cameraInfos = state
			.get_components_iter()
			.filter(|c| state.is_component_visible(*c).unwrap())
			.filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "camera", _ => false })
			.max_by(|c| match state.get(*c, "priority") { Ok(&::entities::Number(ref n)) => (((*n) * 1000f64) as int), _ => 1000 })
			.and_then(|c| match state.get(c, "matrix") { Ok(&::entities::List(ref data)) => Some((c, data)), _ => None })
			.map(|(c, data)| (c, data.iter().filter_map(|elem| match elem { &::entities::Number(ref n) => Some(n.clone() as f32), _ => None }).collect::<Vec<f32>>()) );

		if cameraInfos.is_none() {
			return None;
		}

		let (cameraComponent, matrixData) = cameraInfos.unwrap();
		let matrix = na::Mat4::new(*matrixData.get(0), *matrixData.get(1), *matrixData.get(2), *matrixData.get(3), *matrixData.get(4), *matrixData.get(5), *matrixData.get(6), *matrixData.get(7), *matrixData.get(8), *matrixData.get(9), *matrixData.get(10), *matrixData.get(11), *matrixData.get(12), *matrixData.get(13), *matrixData.get(14), *matrixData.get(15));

		let position = ::physics::PhysicsSystem::get_entity_position(state, &state.get_owner(cameraComponent).unwrap());
		let positionMatrix = na::Mat4::new(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -position.x, -position.y, -position.z, 1.0);

		Some(positionMatrix * matrix)
	}
}
