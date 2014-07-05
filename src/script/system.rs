use entities::{ EntitiesState, EntityID, ComponentID, NativeComponentType };

pub struct ScriptSystem;

impl ScriptSystem {
	pub fn new(state: &EntitiesState)
		-> ScriptSystem
	{
		ScriptSystem
	}

	pub fn process(&mut self, state: &mut EntitiesState, elapsed: &u64)
	{
	}
}
