use entities::{ EntitiesState, EntitiesHelper, EntityID, ComponentID, NativeComponentType };
use std::collections::{ HashSet, HashMap };
use std::cell::RefCell;
use std::rc::Rc;
use nalgebra::na;
use nalgebra::na::{ Norm, Translation, Vec2, Vec3 };
use ncollide::geom::geom::Geom;
use nphysics::world::World;
use nphysics::object::{ RigidBody };

pub struct PhysicsSystem {
    world: World,
    bodies: HashMap<EntityID, Rc<RefCell<RigidBody>>>,
    logger: Box<::log::Logger>
}

impl PhysicsSystem {
    pub fn new<L: ::log::Logger + 'static>(_: &EntitiesState, mut logger: L)
        -> PhysicsSystem
    {
        let mut world = World::new();
        world.set_gravity(Vec2::new(0.0f32, -9.8));

        let shape = ::ncollide::geom::Plane::new(Vec2::new(0.0f32, 1.0));
        let body = Rc::new(RefCell::new(RigidBody::new_static(shape, 0.3, 0.6)));
        world.add_body(body);

        PhysicsSystem {
            world: world,
            bodies: HashMap::new(),
            logger: box logger
        }
    }

    pub fn process(&mut self, state: &mut EntitiesState, elapsed: &f64)
    {
        // getting the list of all entities that have physics activated
        let listOfEntities: HashSet<EntityID> = state.get_components_iter()
            .filter(|c| state.is_component_visible(*c).unwrap())
            .filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "physics", _ => false })
            .filter(|c| match state.get_as_boolean(*c, "activated") { Some(b) => b, _ => false })
            .map(|c| state.get_owner(c).unwrap())
            .collect();

        // removing from the world the elements that have disappeared
        {
            let toRemove: Vec<EntityID> = self.bodies.keys().filter(|e| !listOfEntities.contains(e.clone())).map(|e| e.clone()).collect();
            for e in toRemove.move_iter() {
                //self.bodies.find(&e).     // TODO: remove body from the world
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
                body.borrow_mut().set_translation({ let p = PhysicsSystem::get_entity_position(state, &e); na::Vec2::new(p.x,p.y) });
                body.borrow_mut().set_lin_vel({ let p = PhysicsSystem::get_entity_movement(state, &e); na::Vec2::new(p.x,p.y) });

                self.bodies.insert(e.clone(), body.clone());
                self.world.add_body(body.clone());
            }
        }

        // setting all positions and movements
        for (entity, body) in self.bodies.iter() {
            let position = { let p = PhysicsSystem::get_entity_position(state, entity); na::Vec2::new(p.x,p.y) };
            let movement = { let p = PhysicsSystem::get_entity_movement(state, entity); na::Vec2::new(p.x,p.y) };
            let requestedMovement = PhysicsSystem::get_requested_movement(state, entity);

            let mut borrowedBody = body.borrow_mut();
            borrowedBody.set_translation(position);
            borrowedBody.set_lin_vel(movement);

            if requestedMovement.is_some() && requestedMovement != Some(movement) {
                let acceleration = na::normalize(&(requestedMovement.unwrap() - movement)) * 5.0f32;
                borrowedBody.set_lin_acc(acceleration);
            }
        }

        // step
        self.world.step(*elapsed as f32);

        //
        for (entity, body) in self.bodies.iter() {
            PhysicsSystem::set_position(state, entity, &body.borrow().translation());
            PhysicsSystem::set_movement(state, entity, &body.borrow().lin_vel());
        }
    }

    /// returns the position of an entity
    pub fn get_entity_position(state: &EntitiesState, id: &EntityID)
        -> na::Vec3<f32>
    {
        state
            .get_visible_native_components("position")
            .move_iter()

            // take only the components owned by the entity
            .filter(|c| state.get_owner(c).unwrap() == *id)

            // build a vector from each of the component
            .filter_map(|cmp| match (state.get_as_number(&cmp, "x"), state.get_as_number(&cmp, "y"), state.get_as_number(&cmp, "z")) {
                (Some(x), Some(y), Some(z))
                    => Some(na::Vec3::new(x as f32, y as f32, z as f32)),
                (Some(x), Some(y), _)
                    => Some(na::Vec3::new(x as f32, y as f32, 0.0)),
                _ => None
            })

            // add all the elements together
            .fold(na::Vec3::new(0.0, 0.0, 0.0), |vec, a| vec + a)
    }

    /// Returns the total movement of an entity.
    pub fn get_entity_movement(state: &EntitiesState, id: &EntityID)
        -> na::Vec3<f32>
    {
        state
            .get_visible_native_components("movement")
            .move_iter()

            // take only the components owned by the entity
            .filter(|c| state.get_owner(c).unwrap() == *id)

            // build a vector from each of the component
            .filter_map(|cmp| match (state.get_as_number(&cmp, "x"), state.get_as_number(&cmp, "y"), state.get_as_number(&cmp, "z")) {
                (Some(x), Some(y), Some(z))
                    => Some(na::Vec3::new(x as f32, y as f32, z as f32)),
                (Some(x), Some(y), _)
                    => Some(na::Vec3::new(x as f32, y as f32, 0.0)),
                _ => None
            })

            // add all the elements together
            .fold(na::Vec3::new(0.0, 0.0, 0.0), |vec, a| vec + a)
    }

    /// returns the total requested movement of an entity
    fn get_requested_movement(state: &EntitiesState, id: &EntityID)
        -> Option<Vec2<f32>>
    {
        state
            .get_visible_native_components("requestedMovement")
            .move_iter()

            // take only the components owned by the entity
            .filter(|c| state.get_owner(c).unwrap() == *id)

            // build a vector from each of the component
            .filter_map(|cmp| match (state.get_as_number(&cmp, "x"), state.get_as_number(&cmp, "y"), state.get_as_number(&cmp, "z")) {
                (Some(x), Some(y), _)
                    => Some(Vec2::new(x as f32, y as f32)),
                _ => None
            })

            // add all the elements together
            .fold(None, |vec: Option<Vec2<f32>>, a| match vec { None => Some(a), Some(v) => Some(v + a) })
    }

    /// changes the position of an entity
    /// TODO: this is wrong
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

    /// changes the movement of an entity
    /// TODO: this is wrong
    fn set_movement(state: &mut EntitiesState, id: &EntityID, pos: &Vec2<f32>)
    {
        let list: Vec<ComponentID> = state
            .get_components_iter()
            // take only the components owned by the entity
            .filter(|c| state.get_owner(*c).unwrap() == *id)
            // take only the "position" components
            .filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "movement", _ => false })
            .map(|c| c.clone()).collect();

        for cmp in list.iter()
        {
            state.set(cmp, "x", ::entities::Number(pos.x as f64));
            state.set(cmp, "y", ::entities::Number(pos.y as f64));
        }
    }
}
