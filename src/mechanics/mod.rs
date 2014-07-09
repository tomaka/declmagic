use entities::{ EntitiesState, EntitiesHelper, EntityID, ComponentID, NativeComponentType };
use nalgebra::na::Vec2;

pub use self::system::MechanicsSystem;

mod system;

/// returns the position of an entity
pub fn get_position(state: &EntitiesState, id: &EntityID)
	-> Vec2<f32>
{
	state
		.get_components_iter()

		// take only the components owned by the entity
		.filter(|c| state.get_owner(*c).unwrap() == *id)

		// take only the "position" components
		.filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "position", _ => false })

		// build a vector from each of the component
		.filter_map(|cmp| match (state.get(cmp, "x"), state.get(cmp, "y"), state.get(cmp, "z")) {
			(Ok(&::entities::Number(ref x)), Ok(&::entities::Number(ref y)), _)
				=> Some(Vec2::new(*x as f32, *y as f32)),
			_ => None
		})

		// add all the elements together
		.fold(Vec2::new(0.0, 0.0), |vec, a| Vec2::new(vec.x + a.x, vec.y + a.y))
}

/// returns the total movement of an entity
pub fn get_movement(state: &EntitiesState, id: &EntityID)
	-> Vec2<f32>
{
	state
		.get_components_iter()

		// take only the components owned by the entity
		.filter(|c| state.get_owner(*c).unwrap() == *id)

		// take only the "position" components
		.filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "movement", _ => false })

		// build a vector from each of the component
		.filter_map(|cmp| match (state.get(cmp, "x"), state.get(cmp, "y"), state.get(cmp, "z")) {
			(Ok(&::entities::Number(ref x)), Ok(&::entities::Number(ref y)), _)
				=> Some(Vec2::new(*x as f32, *y as f32)),
			_ => None
		})

		// add all the elements together
		.fold(Vec2::new(0.0, 0.0), |vec, a| Vec2::new(vec.x + a.x, vec.y + a.y))
}
