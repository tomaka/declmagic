use entities::{ EntitiesState, EntityID, ComponentID, NativeComponentType };
use std::collections::{ HashSet, HashMap };
use resources::ResourcesLoader;

pub struct ExternContentSystem {
	loader: Box<ResourcesLoader + Send + Share>,
	logger: Box<::log::Logger>
}

impl ExternContentSystem {
	pub fn new<RL: ResourcesLoader + Send + Share, L: ::log::Logger + 'static>(_: &EntitiesState, loader: RL, logger: L)
		-> ExternContentSystem
	{
		ExternContentSystem {
			loader: box loader as Box<ResourcesLoader+Send+Share>,
			logger: box logger
		}
	}

	pub fn process(&mut self, state: &mut EntitiesState)
	{
		// getting the list of all "externContent" components
		let listOfComponents = state.get_components_iter()
            .filter(|c| state.is_component_visible(*c).unwrap())
			.filter(|c| match state.get_type(*c) { Ok(NativeComponentType(t)) => t.as_slice() == "externContent", _ => false })
			.map(|c| c.clone())
			.collect::<HashSet<ComponentID>>();

	}
}
