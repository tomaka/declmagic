use entities::{ EntitiesState, EntityID, ComponentID, NativeComponentType };
use std::collections::{ HashSet, HashMap };

use resources::ResourcesLoader;
use self::extern_content::ExternContentSystem;

mod extern_content;

pub struct MechanicsSystem {
	externContentSystem: ExternContentSystem
}

impl MechanicsSystem {
	pub fn new<RL: ResourcesLoader + Send + Share>(state: &EntitiesState, loader: RL)
		-> MechanicsSystem
	{
		MechanicsSystem {
			externContentSystem: ExternContentSystem::new(state, loader)
		}
	}

	pub fn process(&mut self, state: &mut EntitiesState, _: &u64)
	{
		self.externContentSystem.process(state);
	}
}
