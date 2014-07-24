use entities::{ EntitiesState, EntitiesHelper, EntityID, ComponentID, NativeComponentType };
use std::collections::{ HashSet, HashMap };
use std::cell::RefCell;
use std::rc::Rc;
use nalgebra::na;
use nalgebra::na::{ Norm, Translation, Vec2, Vec3 };
use ncollide::geom::geom::Geom;
use nphysics::world::World;
use nphysics::object::{ RigidBody };
use log;

pub struct PhysicsSystem {
    world: World,
    bodies: HashMap<EntityID, Rc<RefCell<RigidBody>>>,
}

impl PhysicsSystem {
    pub fn new(_: &EntitiesState, log: |log::LogRecord|)
        -> PhysicsSystem
    {
        let mut world = World::new();
        world.set_gravity(Vec2::new(1.0f32, 1.0));

        //let shape = ::ncollide::geom::Plane::new(Vec2::new(0.0f32, 1.0));
        //let body = Rc::new(RefCell::new(RigidBody::new_static(shape, 0.3, 0.6)));
        //world.add_body(body);

        PhysicsSystem {
            world: world,
            bodies: HashMap::new()
        }
    }

    pub fn process(&mut self, state: &mut EntitiesState, elapsed: &f64, log: |log::LogRecord|)
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
                body.borrow_mut().set_translation({ let p = get_entity_position(state, &e); na::Vec2::new(p.x,p.y) });
                body.borrow_mut().set_lin_vel({ let p = get_entity_movement(state, &e); na::Vec2::new(p.x,p.y) });
                body.borrow_mut().set_lin_acc_scale(na::Vec2::new(0.0, 0.0));

                self.bodies.insert(e.clone(), body.clone());
                self.world.add_body(body.clone());
            }
        }

        // setting all positions and movements
        for (entity, body) in self.bodies.iter() {
            let position = { let p = get_entity_position(state, entity); na::Vec2::new(p.x,p.y) };
            let movement = { let p = get_entity_movement(state, entity); na::Vec2::new(p.x,p.y) };
            let requestedMovement = get_requested_movement(state, entity);

            let mut borrowedBody = body.borrow_mut();
            borrowedBody.set_translation(position);
            borrowedBody.set_lin_vel(movement);
            borrowedBody.activate(100.0);       // objects tend to deactivate too often

            if requestedMovement.is_some() {
                let requestedMovement = requestedMovement.unwrap();

                let acceleration: Vec2<f32> = Vec2::new(
                    if requestedMovement.x < 0.0 && requestedMovement.x < movement.x { -1.0 }
                    else if requestedMovement.x > 0.0 && requestedMovement.x > movement.x { 1.0 }
                    else { 0.0 },
                    if requestedMovement.y < 0.0 && requestedMovement.y < movement.y { -1.0 }
                    else if requestedMovement.y > 0.0 && requestedMovement.y > movement.y { 1.0 }
                    else { 0.0 }
                );

                if acceleration != borrowedBody.lin_acc_scale() {
                    borrowedBody.set_lin_acc_scale(acceleration);
                }
            }
        }

        // step
        self.world.step(*elapsed as f32);

        //
        for (entity, body) in self.bodies.iter() {
            set_position(state, entity, &body.borrow().translation());
            set_movement(state, entity, &body.borrow().lin_vel());
        }
    }
}

/// returns the position of an entity
pub fn get_entity_position(state: &EntitiesState, id: &EntityID)
    -> na::Vec3<f32>
{
    use std::iter::AdditiveIterator;

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
        .sum()
}

/// Returns the total movement of an entity.
pub fn get_entity_movement(state: &EntitiesState, id: &EntityID)
    -> na::Vec3<f32>
{
    use std::iter::AdditiveIterator;

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
        .sum()
}

/// returns the total requested movement of an entity
pub fn get_requested_movement(state: &EntitiesState, id: &EntityID)
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
pub fn set_position(state: &mut EntitiesState, id: &EntityID, pos: &Vec2<f32>)
{
    let current = get_entity_position(state, id);
    let current = na::Vec2::new(current.x, current.y);
    adjust_position(state, id, &(pos - current));
}

/// updates the position of an entity
pub fn adjust_position(state: &mut EntitiesState, id: &EntityID, diff: &Vec2<f32>)
{
    let list: Vec<ComponentID> = state
        .get_components_iter()
        // take only the components owned by the entity
        .filter(|c| state.get_owner(*c).unwrap() == *id)
        // take only the "position" components
        .filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "position", _ => false })
        .filter(|c| state.get_as_boolean(*c, "autoupdate").unwrap_or(true))
        .map(|c| c.clone()).collect();

    if list.len() == 0 {
        let mut data = HashMap::new();
        data.insert("x".to_string(), ::entities::Number(diff.x as f64));
        data.insert("y".to_string(), ::entities::Number(diff.y as f64));
        state.create_native_component(id, "position", data);
        return
    }

    let diff = diff / (list.len() as f32);

    for cmp in list.iter() {
        let (x, y) = (
                state.get_as_number(cmp, "x").unwrap_or(0.0),
                state.get_as_number(cmp, "y").unwrap_or(0.0)
            );

        state.set(cmp, "x", ::entities::Number(x + diff.x as f64));
        state.set(cmp, "y", ::entities::Number(y + diff.y as f64));
    }
}

/// changes the position of an entity
pub fn set_movement(state: &mut EntitiesState, id: &EntityID, movement: &Vec2<f32>)
{
    let current = get_entity_movement(state, id);
    let current = na::Vec2::new(current.x, current.y);
    adjust_movement(state, id, &(movement - current));
}

/// changes the movement of an entity
pub fn adjust_movement(state: &mut EntitiesState, id: &EntityID, diff: &Vec2<f32>)
{
    let list: Vec<ComponentID> = state
        .get_visible_native_components("movement")
        .move_iter()
        // take only the components owned by the entity
        .filter(|c| state.get_owner(c).unwrap() == *id)
        .filter(|c| state.get_as_boolean(c, "autoupdate").unwrap_or(true))
        .collect();

    if list.len() == 0 {
        let mut data = HashMap::new();
        data.insert("x".to_string(), ::entities::Number(diff.x as f64));
        data.insert("y".to_string(), ::entities::Number(diff.y as f64));
        state.create_native_component(id, "movement", data);
        return
    }

    let diff = diff / (list.len() as f32);

    for cmp in list.iter() {
        let (x, y) = (
                state.get_as_number(cmp, "x").unwrap_or(0.0),
                state.get_as_number(cmp, "y").unwrap_or(0.0)
            );

        state.set(cmp, "x", ::entities::Number(x + diff.x as f64));
        state.set(cmp, "y", ::entities::Number(y + diff.y as f64));
    }
}
