use super::managed_display::ManagedDisplay;
use entities::{ EntitiesState, EntityID, ComponentID, NativeComponentType };
use nalgebra::na::{ Mat4, Vec3, Eye };
use std::collections::{ HashSet, HashMap };
use std::sync::Arc;
use super::sprite_displayer::SpriteDisplayer;
use super::Drawable;

pub struct DisplaySystem {
	display: Arc<ManagedDisplay>,
	sprites: HashMap<ComponentID, SpriteDisplayer>
}

impl DisplaySystem {
	pub fn new(display: Arc<ManagedDisplay>, state: &EntitiesState)
		-> DisplaySystem
	{
		DisplaySystem {
			display: display.clone(),
			sprites: HashMap::new()
		}
	}

	pub fn draw(&mut self, state: &EntitiesState, elapsed: &u64)
	{
		self.update_sprite_displayers(state);

		let camera = DisplaySystem::get_camera(state);

	 	for (cmp, sprite) in self.sprites.iter() {
	 		let pos = DisplaySystem::get_position(state, &state.get_owner(cmp).unwrap());
	 		let translationMatrix = Mat4::new(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, pos.x, pos.y, pos.z, 1.0);
			sprite.draw(&(translationMatrix * camera));
		}
	}

	fn update_sprite_displayers(&mut self, state: &EntitiesState)
	{
		// getting the list of all sprite displayer components
		let listOfComponents = state.get_components_iter()
            .filter(|c| state.is_component_visible(*c).unwrap())
			.filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "spriteDisplay", _ => false })
			.map(|c| c.clone())
			.collect::<HashSet<EntityID>>();

		// removing from the list the elements that have disappeared
		{	let toRemove = self.sprites.keys().filter(|e| !listOfComponents.contains(e.clone())).map(|e| e.clone()).collect::<Vec<EntityID>>();
			for c in toRemove.move_iter() {
				self.sprites.remove(&c);
			}
		}

		// adding elements that are not yet created
		{	let toCreate = listOfComponents.iter().filter(|e| !self.sprites.contains_key(e.clone())).map(|e| e.clone()).collect::<Vec<EntityID>>();
			for spriteDisplayComponent in toCreate.move_iter() {
				// getting the name of the texture
				let textureName = match state.get(&spriteDisplayComponent, "texture") { Ok(&::entities::String(ref s)) => s, _ => continue };

				let mut spriteDisplayer = SpriteDisplayer::new(self.display.clone(), textureName.as_slice()).unwrap();

				// getting coordinates
				spriteDisplayer.set_rectangle_coords(
					match state.get(&spriteDisplayComponent, "leftX") { Ok(&::entities::Number(ref nb)) => *nb as f32, _ => -0.5 },
					match state.get(&spriteDisplayComponent, "topY") { Ok(&::entities::Number(ref nb)) => *nb as f32, _ => 1.0 },
					match state.get(&spriteDisplayComponent, "rightX") { Ok(&::entities::Number(ref nb)) => *nb as f32, _ => 0.5 },
					match state.get(&spriteDisplayComponent, "bottomY") { Ok(&::entities::Number(ref nb)) => *nb as f32, _ => 0.0 }
				);

				// inserting in sprites list
				self.sprites.insert(spriteDisplayComponent.clone(), spriteDisplayer);
			}
		}
	}

	/// returns the camera matrix of the scene
	fn get_camera(state: &EntitiesState)
		-> Mat4<f32>
	{
		state
			.get_components_iter()

			// filter out non-visible components
			.filter(|c| state.is_component_visible(*c).unwrap())

			// take only the "camera" components
			.filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "camera", _ => false })

			// get the component with the highest priority
			.max_by(|c| match state.get(*c, "priority") { Ok(&::entities::Number(ref n)) => (((*n) * 1000f64) as int), _ => -99999999 })

			// get the matrix of this component and check its type
			.and_then(|e| match state.get(e, "matrix") { Ok(&::entities::List(ref data)) => Some(data), _ => None })

			// check the type of each element in the matrix and creates an array
			.map(|data| data.iter().filter_map(|elem| match elem { &::entities::Number(ref n) => Some(n.clone() as f32), _ => None }).collect() )

			// turn into matrix
			.map(|array: Vec<f32>| Mat4::new(*array.get(0), *array.get(1), *array.get(2), *array.get(3), *array.get(4), *array.get(5), *array.get(6), *array.get(7), *array.get(8), *array.get(9), *array.get(10), *array.get(11), *array.get(12), *array.get(13), *array.get(14), *array.get(15)))

			// return identity if the option is empty
			.unwrap_or_else(|| Eye::new_identity(4))
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
