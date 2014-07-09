use super::managed_display::ManagedDisplay;
use entities::{ EntitiesState, EntitiesHelper, EntityID, ComponentID, NativeComponentType };
use nalgebra::na::{ Mat4, Vec3, Eye };
use std::collections::{ HashSet, HashMap };
use std::sync::Arc;
use super::sprite_displayer::SpriteDisplayer;
use super::Drawable;

mod custom_display_system;

pub struct DisplaySystem {
	display: Arc<ManagedDisplay>,
	customDisplay: custom_display_system::CustomDisplaySystem,
	sprites: HashMap<ComponentID, SpriteDisplayer>,
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

	 	for (cmp, sprite) in self.sprites.iter() {
	 		let pos = DisplaySystem::get_position(state, &state.get_owner(cmp).unwrap());
	 		let translationMatrix = Mat4::new(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, pos.x, pos.y, pos.z, 1.0);
			sprite.draw(&(translationMatrix * camera));
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
				// getting the name of the texture
				let textureName = match state.get_as_string(&spriteDisplayComponent, "texture") {
					Some(s) => s,
					_ => {
						declmagic_error!(self.logger, "component {} has no valid \"texture\" element", spriteDisplayComponent)
						continue
					}
				};

				let mut spriteDisplayer = SpriteDisplayer::new(self.display.clone(), textureName.as_slice()).unwrap();

				// getting coordinates
				spriteDisplayer.set_rectangle_coords(
					match state.get(&spriteDisplayComponent, "leftX") { Ok(&::entities::Number(ref nb)) => Some(*nb as f32), _ => None },
					match state.get(&spriteDisplayComponent, "topY") { Ok(&::entities::Number(ref nb)) => Some(*nb as f32), _ => None },
					match state.get(&spriteDisplayComponent, "rightX") { Ok(&::entities::Number(ref nb)) => Some(*nb as f32), _ => None },
					match state.get(&spriteDisplayComponent, "bottomY") { Ok(&::entities::Number(ref nb)) => Some(*nb as f32), _ => None }
				);

				// inserting in sprites list
				self.sprites.insert(spriteDisplayComponent.clone(), spriteDisplayer);
			}
		}
	}

	/// returns the camera matrix of the scene
	fn get_camera(state: &EntitiesState)
		-> Option<Mat4<f32>>
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
		let matrix = Mat4::new(*matrixData.get(0), *matrixData.get(1), *matrixData.get(2), *matrixData.get(3), *matrixData.get(4), *matrixData.get(5), *matrixData.get(6), *matrixData.get(7), *matrixData.get(8), *matrixData.get(9), *matrixData.get(10), *matrixData.get(11), *matrixData.get(12), *matrixData.get(13), *matrixData.get(14), *matrixData.get(15));

		let position = DisplaySystem::get_position(state, &state.get_owner(cameraComponent).unwrap());
		let positionMatrix = Mat4::new(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -position.x, -position.y, -position.z, 1.0);

		Some(positionMatrix * matrix)
	}

	/// returns the position of an entity
	fn get_position(state: &EntitiesState, id: &EntityID)
		-> Vec3<f32>
	{
		state
			.get_components_iter()

			// filter out non-visible components
			.filter(|c| state.is_component_visible(*c).unwrap())

			// take only the components owned by the entity
			.filter(|c| state.get_owner(*c).unwrap() == *id)

			// take only the "position" components
			.filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "position", _ => false })

			// build a vector from each of the component
			.filter_map(|cmp| match (state.get(cmp, "x"), state.get(cmp, "y"), state.get(cmp, "z")) {
				(Ok(&::entities::Number(ref x)), Ok(&::entities::Number(ref y)), Ok(&::entities::Number(ref z)))
					=> Some(Vec3::new(*x as f32, *y as f32, *z as f32)),
				(Ok(&::entities::Number(ref x)), Ok(&::entities::Number(ref y)), _)
					=> Some(Vec3::new(*x as f32, *y as f32, 0.0)),
				_ => None
			})

			// add all the elements together
			.fold(Vec3::new(0.0, 0.0, 0.0), |vec, a| vec + a)
	}
}
