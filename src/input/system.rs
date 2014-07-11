use super::{ Message, Element, Pressed, Released, MouseMoved };
use entities::{ EntitiesState, EntitiesHelper, EntityID, ComponentID, NativeComponentType };
use nalgebra::na;
use nalgebra::na::Inv;
use script;

pub struct InputSystem {
    logger: Box<::log::Logger>,
    current_hover: Option<EntityID>,
    last_known_mouse_position: na::Vec2<f64>
}

impl InputSystem {
    pub fn new<L: ::log::Logger + 'static>(_: &EntitiesState, logger: L)
        -> InputSystem
    {
        InputSystem {
            logger: box logger,
            current_hover: None,
            last_known_mouse_position: na::Vec2::new(0.0, 0.0)
        }
    }

    pub fn process(&mut self, state: &mut EntitiesState, elapsed: &f64, messages: &[Message])
    {
        self.process_hover(state, elapsed, messages);

        let mut filteredMessagesIter = messages.iter().filter_map(|msg| match msg {
                                                                            &Pressed(ref e) => Some((e.clone(), true)),
                                                                            &Released(ref e) => Some((e.clone(), false)),
                                                                            _ => None
                                                                        });

        for (element, pressed) in filteredMessagesIter {
            let elementStr = format!("{}", element);

            for (component, script) in state
                                .get_components_iter()
                                .filter(|c| state.is_component_visible(*c).unwrap())
                                // take only the "inputHandler" components
                                .filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "inputHandler", _ => false })
                                // filter if they have the right element
                                .filter(|c| match state.get_as_string(*c, "element") { Some(s) => s == elementStr, _ => false })
                                // obtain the script and the component id
                                .filter_map(|c| match state.get_as_string(c, "script") { Some(s) => Some((c.clone(), s)), _ => None })

                                .collect::<Vec<(ComponentID, String)>>().move_iter()
            {
                script::execute_mut(state, &component, &script.as_slice()).unwrap();
            }

            for (component, entity) in state
                                .get_components_iter()
                                .filter(|c| state.is_component_visible(*c).unwrap())
                                // take only the "inputHandler" components
                                .filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "inputHandler", _ => false })
                                // filter if they have the right element
                                .filter(|c| match state.get_as_string(*c, "element") { Some(s) => s == elementStr, _ => false })
                                // obtain the script and the component id
                                .filter_map(|c| match state.get_as_entity(c, "prototypeWhilePressed") { Some(id) => Some((c.clone(), id)), _ => None })
                                .collect::<Vec<(ComponentID, EntityID)>>().move_iter()
            {
                let children = state.get_component_children(&component).unwrap();

                match (children.len() != 0, pressed) {
                    (false, true) =>
                        {
                            let owner = match state.get_owner(&component) { Ok(o) => o, _ => continue };
                            let newCmp = state.create_component_from_entity(&owner, &entity, ::std::collections::HashMap::new()).unwrap();
                            state.set_component_parent(&newCmp, &component);
                        },

                    (true, false) =>
                        {
                            for c in children.iter() {
                                state.destroy_component(c);
                            }
                        },

                    _ => ()
                }
            }
        }
    }

    fn process_hover(&mut self, state: &mut EntitiesState, _: &f64, messages: &[Message])
    {
        // getting the mouse position between (-1, -1) and (1, 1)
        let mouse_position = messages
            .iter().rev()
            .filter_map(|msg| match msg {
                &MouseMoved(x, y) => Some(na::Vec2::new(x, y)),
                _ => None
            })
            .next().unwrap_or(self.last_known_mouse_position);
        self.last_known_mouse_position = mouse_position;

        // getting the mouse position in camera units
        let mouse_position = {
            let mousePosVector = na::Vec4::new(mouse_position.x as f32, mouse_position.y as f32, 0.0, 1.0);
            let matrix = { let mut m = ::display::DisplaySystem::get_camera(state).unwrap(); m.inv(); m };
            let result = mousePosVector * matrix;
            na::Vec2::new(result.x / result.w, result.y / result.w)
        };

        // getting which entity is being hovered
        let hovered_entity: Option<EntityID> = state
            .get_components_iter()
            .filter(|c| state.is_component_visible(*c).unwrap())
            .filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "clickBox", _ => false })
            .filter_map(|component| {
                let entity = match state.get_owner(component) { Ok(t) => t, _ => return None };
                let entity_position = ::physics::PhysicsSystem::get_entity_position(state, &entity);

                let coord1 = match (state.get_as_number(component, "leftX"), state.get_as_number(component, "bottomY"))
                {
                    (Some(x), Some(y)) => na::Vec2::new(x as f32 + entity_position.x, y as f32 + entity_position.y),
                    _ => return None
                };

                let coord2 = match (state.get_as_number(component, "rightX"), state.get_as_number(component, "topY"))
                {
                    (Some(x), Some(y)) => na::Vec2::new(x as f32 + entity_position.x, y as f32 + entity_position.y),
                    _ => return None
                };

                if (coord1.x < coord2.x) && coord2.x < mouse_position.x { return None }
                if (coord2.x < coord1.x) && coord1.x < mouse_position.x { return None }
                if (coord1.x > coord2.x) && coord2.x > mouse_position.x { return None }
                if (coord2.x > coord1.x) && coord1.x > mouse_position.x { return None }
                if (coord1.y < coord2.y) && coord2.y < mouse_position.y { return None }
                if (coord2.y < coord1.y) && coord1.y < mouse_position.y { return None }
                if (coord1.y > coord2.y) && coord2.y > mouse_position.y { return None }
                if (coord2.y > coord1.y) && coord1.y > mouse_position.y { return None }

                Some(entity)
            })
            .next();

        // if the hovered entity has not changed, we have finished
        if hovered_entity == self.current_hover {
            return
        }

        // we have left the current hovered entity, executing script and removing prototype
        if self.current_hover.is_some() {
            // looping through each "hoverHandler" of the current_hover entity
            for cmp in state
                .get_components_iter()
                .filter(|c| state.is_component_visible(*c).unwrap())
                .filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "hoverHandler", _ => false })
                .filter(|c| state.get_owner(*c).ok() == self.current_hover)
                .map(|c| c.clone())
                .collect::<Vec<ComponentID>>().move_iter()
            {
                // removing all its children (ie. the prototype)
                for c in state.get_component_children(&cmp).unwrap().move_iter() {
                    state.destroy_component(&c).ok();
                }

                // executing onLeave script
                match state.get_as_string(&cmp, "scriptOnLeave") {
                    None => (),
                    Some(script) => { script::execute_mut(state, &cmp, &script.as_slice()).unwrap(); }
                };
            }
        }

        // updating self
        self.current_hover = hovered_entity;

        // processing the new entity
        match hovered_entity {
            None => (),
            Some(hovered_entity) => {
                // looping through each "hoverHandler" of the new entity
                for cmp in state
                    .get_components_iter()
                    .filter(|c| state.is_component_visible(*c).unwrap())
                    .filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "hoverHandler", _ => false })
                    .filter(|c| state.get_owner(*c).ok() == Some(hovered_entity))
                    .map(|c| c.clone())
                    .collect::<Vec<ComponentID>>().move_iter()
                {
                    // adding prototype
                    match state.get_as_entity(&cmp, "prototype") {
                        Some(prototype) => {
                            let newCmp = state.create_component_from_entity(&hovered_entity, &prototype, ::std::collections::HashMap::new()).unwrap();
                            state.set_component_parent(&newCmp, &cmp);
                        },
                        None => ()
                    };

                    // executing onLeave script
                    match state.get_as_string(&cmp, "scriptOnEnter") {
                        None => (),
                        Some(script) => { script::execute_mut(state, &cmp, &script.as_slice()).unwrap(); }
                    };
                }
            }
        }
    }
}
