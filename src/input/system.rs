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
        }
    }
}
