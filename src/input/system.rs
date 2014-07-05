use super::{ Message, Element, Pressed, Released };
use entities::{ EntitiesState, EntityID, ComponentID, NativeComponentType };
use script;

pub struct InputSystem;

impl InputSystem {
    pub fn new(state: &EntitiesState)
        -> InputSystem
    {
        InputSystem
    }

    pub fn process(&mut self, state: &mut EntitiesState, elapsed: &u64, messages: &[Message])
    {
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
                script::execute(state, &component, &script.as_slice()).unwrap()
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
}
