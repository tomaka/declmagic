use super::{ Message, Element, Pressed, Released };
use entities::{ EntitiesState, EntitiesHelper, EntityID, ComponentID, NativeComponentType };
use nalgebra::na;
use script;

pub struct InputSystem {
    logger: Box<::log::Logger>,
    current_hover: Option<EntityID>
}

impl InputSystem {
    pub fn new<L: ::log::Logger + 'static>(_: &EntitiesState, logger: L)
        -> InputSystem
    {
        InputSystem {
            logger: box logger,
            current_hover: None
        }
    }

    pub fn process(&mut self, state: &mut EntitiesState, elapsed: &u64, messages: &[Message])
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
                                .filter(|c| match state.get(*c, "element") { Ok(&::entities::String(ref s)) => s == &elementStr, _ => false })
                                // obtain the script and the component id
                                .filter_map(|c| match state.get(c, "script") { Ok(&::entities::String(ref s)) => Some((c.clone(), s.clone())), _ => None })

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
                                .filter(|c| match state.get(*c, "element") { Ok(&::entities::String(ref s)) => s == &elementStr, _ => false })
                                // obtain the script and the component id
                                .filter_map(|c| match state.get(c, "prototypeWhilePressed") { Ok(&::entities::Entity(ref id)) => Some((c.clone(), id.clone())), _ => None })

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

    fn process_hover(&mut self, state: &mut EntitiesState, _: &u64, messages: &[Message])
    {
        let hovered_entity: Option<EntityID> = state
            .get_components_iter()
            .filter(|c| state.is_component_visible(*c).unwrap())
            .filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "clickBox", _ => false })
            .filter_map(|component| {
                let entity_position = ::physics::PhysicsSystem::get_entity_position(state, &match state.get_owner(component) { Ok(t) => t, _ => return None });

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

                // TODO: 
                None
            })
            .next();

        // we have left the current hovered entity, executing script and removing prototype
        if self.current_hover.is_some() && hovered_entity != self.current_hover {
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

    }
}
