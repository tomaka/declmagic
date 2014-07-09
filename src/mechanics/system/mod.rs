use entities::{ EntitiesState, EntitiesHelper, EntityID, ComponentID, NativeComponentType };
use std::collections::{ HashSet, HashMap };

use resources::ResourcesLoader;
use self::extern_content::ExternContentSystem;

mod extern_content;

pub struct MechanicsSystem {
	externContentSystem: ExternContentSystem,
	logger: Box<::log::Logger>
}

impl MechanicsSystem {
	pub fn new<RL: ResourcesLoader + Send + Share, L: ::log::Logger + 'static>(state: &EntitiesState, loader: RL, logger: L)
		-> MechanicsSystem
	{
		MechanicsSystem {
			externContentSystem: ExternContentSystem::new(state, loader, logger.clone()),
			logger: box logger
		}
	}

	pub fn process(&mut self, state: &mut EntitiesState, _: &u64)
	{
		self.externContentSystem.process(state);
	}
}
