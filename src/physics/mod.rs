use entities::{ EntitiesState, EntityID, ComponentID, NativeComponentType };
use std::collections::{ HashSet, HashMap };
use std::cell::RefCell;
use std::rc::Rc;
use nalgebra::na::{ Norm, Translation, Vec2 };
use ncollide::geom::geom::Geom;
use nphysics::world::World;
use nphysics::object::{ RigidBody };

pub struct PhysicsSystem {
	world: World,
	bodies: HashMap<EntityID, Rc<RefCell<RigidBody>>>
}

impl PhysicsSystem {
	pub fn new(_: &EntitiesState)
		-> PhysicsSystem
	{
		let mut world = World::new();
		world.set_gravity(Vec2::new(0.0f32, -9.81));

		let shape = ::ncollide::geom::Plane::new(Vec2::new(0.0f32, 1.0));
		let body = Rc::new(RefCell::new(RigidBody::new_static(shape, 0.3, 0.6)));
		world.add_body(body);

		PhysicsSystem {
			world: world,
			bodies: HashMap::new()
		}
	}

	pub fn process(&mut self, state: &mut EntitiesState, elapsed: &u64)
	{
		// getting the list of all entities that have physics activated
		let listOfEntities: HashSet<EntityID> = state.get_components_iter()
            .filter(|c| state.is_component_visible(*c).unwrap())
			.filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "physics", _ => false })
			.filter(|c| match state.get(*c, "activated") { Ok(&::entities::Boolean(ref b)) => *b, _ => false })
			.map(|c| state.get_owner(c).unwrap())
			.collect();

		// removing from the world the elements that have disappeared
		{
			let toRemove: Vec<EntityID> = self.bodies.keys().filter(|e| !listOfEntities.contains(e.clone())).map(|e| e.clone()).collect();
			for e in toRemove.move_iter() {
				//self.bodies.find(&e).		// TODO: remove body from the world
				self.bodies.remove(&e);
			}
		}

		// adding elements that are not yet in the world
		{
			let toCreate: Vec<EntityID> = listOfEntities.iter().filter(|e| !self.bodies.contains_key(e.clone())).map(|e| e.clone()).collect();
			for e in toCreate.move_iter() {
				let shape = ::ncollide::geom::Cuboid::new(Vec2::new(0.5f32, 0.5f32));
				let body = Rc::new(RefCell::new(RigidBody::new_dynamic(shape, 1.0, 0.5, 0.0)));

				// initializing body with current position and movement
				body.borrow_mut().set_translation(PhysicsSystem::get_position(state, &e));
				body.borrow_mut().set_lin_vel(PhysicsSystem::get_movement(state, &e));

				self.bodies.insert(e.clone(), body.clone());
				self.world.add_body(body.clone());
			}
		}

		// setting all positions and movements
		for (entity, body) in self.bodies.iter() {
			let pos = PhysicsSystem::get_position(state, entity);
			let requestedMovement = PhysicsSystem::get_movement(state, entity);

			let mut borrowedBody = body.borrow_mut();
			borrowedBody.set_translation(pos);

			let currentMovement = borrowedBody.lin_vel();
			borrowedBody.set_lin_acc(Norm::normalize_cpy(&(requestedMovement - currentMovement)));
		}

		// step
		self.world.step((*elapsed as f32) / 1000.0);

		//
		for (entity, body) in self.bodies.iter() {
			PhysicsSystem::set_position(state, entity, &body.borrow().translation())
		}
	}

	/// returns the position of an entity
	fn get_position(state: &EntitiesState, id: &EntityID)
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
	fn get_movement(state: &EntitiesState, id: &EntityID)
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

	/// changes the position of an entity
	fn set_position(state: &mut EntitiesState, id: &EntityID, pos: &Vec2<f32>)
	{
		let list: Vec<ComponentID> = state
			.get_components_iter()
			// take only the components owned by the entity
			.filter(|c| state.get_owner(*c).unwrap() == *id)
			// take only the "position" components
			.filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "position", _ => false })
			.map(|c| c.clone()).collect();

		for cmp in list.iter()
		{
			state.set(cmp, "x", ::entities::Number(pos.x as f64));
			state.set(cmp, "y", ::entities::Number(pos.y as f64));
		}
	}
}
