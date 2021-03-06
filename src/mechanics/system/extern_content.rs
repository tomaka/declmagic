use entities::{ EntitiesState, EntitiesHelper, EntityID, ComponentID, NativeComponentType };
use std::collections::{ HashSet, HashMap };
use resources::ResourcesLoader;
use log;

pub struct ExternContentSystem {
	loader: Box<ResourcesLoader + Send + Share>
}

impl ExternContentSystem {
	pub fn new<RL: ResourcesLoader + Send + Share>(_: &EntitiesState, loader: RL, log: |log::LogRecord|)
		-> ExternContentSystem
	{
		ExternContentSystem {
			loader: box loader as Box<ResourcesLoader+Send+Share>
		}
	}

	pub fn process(&mut self, state: &mut EntitiesState, log: |log::LogRecord|)
	{
		// getting the list of all "externContent" components
		let listOfComponents = state.get_components_iter()
            .filter(|c| state.is_component_visible(*c).unwrap())
			.filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "externContent", _ => false })
			.map(|c| c.clone())
			.collect::<HashSet<ComponentID>>();

	}
}
