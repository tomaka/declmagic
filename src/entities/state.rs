extern crate std;

use std::collections::HashMap;

pub type EntityID = uint;
pub type ComponentID = uint;

pub struct EntitiesState {
	components: std::collections::HashMap<ComponentID, Component>,
	entities: std::collections::HashMap<EntityID, EntityData>,
	nextComponentID: ComponentID,
	nextEntityID: EntityID,
	componentsOfNativeType: HashMap<String, ComponentID>
}

struct EntityData {
	name: Option<String>,
	visible: bool,

	components: Vec<ComponentID>,

	componentsOfType: Vec<ComponentID>,
	parameters: std::collections::HashMap<String, Box<std::any::Any>>,
	defaultParameters: std::collections::HashMap<String, Box<std::any::Any>>
}

struct Component {
	owner: EntityID,

	data: ComponentData,

	linkedFrom: Vec<ComponentID>,

	parent: Option<ComponentID>,
	children: Vec<ComponentID>,

	cmpType: ComponentType
}

enum ComponentData {
	ComponentDataNative(HashMap<String, Data>),
	ComponentDataLink(ComponentID)
}

#[deriving(Clone,Show)]
pub enum ComponentType {
	NativeComponentType(String),
	EntityComponentType(EntityID)
}

#[deriving(Clone,Show)]
pub enum Data {
	Number(f64),
	String(String),
	Boolean(bool),
	List(Vec<Data>),
	Entity(EntityID),
	Empty
}

impl EntitiesState {
	pub fn new() -> EntitiesState
	{
		EntitiesState {
			components: HashMap::new(),
			entities: HashMap::new(),
			nextComponentID: 1,
			nextEntityID: 1,
			componentsOfNativeType: HashMap::new()
		}
	}

	pub fn create_entity(&mut self, name: Option<String>)
		-> EntityID
	{
		let id = self.nextEntityID;

		let entity = EntityData {
			name: name,
			visible: true,

			components: Vec::new(),

			componentsOfType: Vec::new(),
			parameters: std::collections::HashMap::new(),
			defaultParameters: std::collections::HashMap::new()
		};

		self.entities.insert(id, entity);
		self.nextEntityID += 1;
		return id;
	}

	pub fn destroy_entity(&mut self, id: &EntityID)
		-> Result<Vec<ComponentID>, String>
	{
		let mut result = Vec::new();

		let componentsList = {
			let entity = match self.entities.find(id) {
				None => return Err(format!("Entity with ID {} is not valid", id)),
				Some(e) => e
			};

			if entity.componentsOfType.len() != 0 {
				return Err(format!("Cannot destroy entity with ID {} (name: {}) because it has components of its type", id, entity.name));
			}

			entity.components.clone()
		};

		for cmp in componentsList.iter() {
			let l = self.destroy_component(cmp);
			for e in l.move_iter() {
				result.push(e);
			}
		}

		self.entities.remove(id);
		Ok(vec!())
	}

	fn clone_entity(&mut self, id: &EntityID)
		-> EntityID
	{
		unimplemented!()
	}

	fn add_entity_parameter(&mut self, id: &EntityID, name: &String)
	{
		unimplemented!()
	}

	pub fn create_native_component(&mut self, owner: &EntityID, typename: &str, data: HashMap<String, Data>)
		-> Result<ComponentID, String>
	{
		let newID = self.nextComponentID;

		let newComponent = Component {
			owner: owner.clone(),
			data: ComponentDataNative(data),
			linkedFrom: Vec::new(),
			parent: None,
			children: Vec::new(),
			cmpType: NativeComponentType(typename.to_string())
		};

		let mut entity = match self.entities.find_mut(owner) {
			None => return Err(format!("Entity with ID {} is not valid", owner)),
			Some(e) => e
		};

		entity.components.push(newID);
		self.components.insert(newID, newComponent);
		self.nextComponentID = self.nextComponentID + 1;

		self.componentsOfNativeType.insert(typename.to_string(), newID);

		Ok(newID)
	}

	pub fn destroy_component(&mut self, id: &ComponentID)
		-> Vec<ComponentID>
	{
		let mut retValue: Vec<ComponentID> = Vec::new();

		let children = {
			let component = self.components.find(id).unwrap();
			component.children.clone()
		};

		for child in children.iter() {
			for e in self.destroy_component(child).move_iter() {
				retValue.push(e);
			}
		}

		let linked = {
			let component = self.components.find(id).unwrap();
			component.linkedFrom.clone()
		};

		for cmp in linked.iter() {
			for e in self.destroy_component(cmp).move_iter() {
				retValue.push(e);
			}
		}

		// removing from entity
		{
			let owner = self.components.find(id).unwrap().owner;
			let mut entity = self.entities.find_mut(&owner).unwrap();
			let pos = entity.components.iter().position(|e| *e == owner).unwrap();
			entity.components.remove(pos);
		}

		// TODO: remove from componentsOfNativeType

		// removing from components list
		self.components.remove(id);

		retValue.push(id.clone());
		return retValue;
	}

	pub fn set(&mut self, id: &ComponentID, field: &str, data: Data)
		-> Result<(), String>
	{
		let mut idIter = id.clone();

		loop {
			let mut component = match self.components.find_mut(&idIter) {
				None => return Err(format!("Component with ID {} not found", idIter)),
				Some(c) => c
			};

			match &mut component.data {
				&ComponentDataNative(ref mut val) => {
					val.insert(field.to_string(), data);
					return Ok(());
				},
				&ComponentDataLink(c) => {
					idIter = c.clone();
					continue;
				}
			}
		}

		unreachable!();
	}

	pub fn get<'a>(&'a self, id: &ComponentID, field: &str)
		-> Result<&'a Data, String>
	{
		let component = match self.components.find(id) {
			None => return Err(format!("Component with ID {} not found", id)),
			Some(c) => c
		};

		match component.data {
			ComponentDataNative(ref data) => {
				match data.find_equiv(&field) {
					Some(a) => Ok(a),
					None => Err(format!("No field named {} in the component", field))
				}
			},
			ComponentDataLink(c) => {
				self.get(&c, field)
			}
		}
	}

	pub fn get_owner(&self, id: &ComponentID)
		-> Result<EntityID, String>
	{
		let mut component = match self.components.find(id) {
			None => return Err(format!("Component with ID {} doesn't exist", id)),
			Some(e) => e
		};

		Ok(component.owner)
	}

	pub fn get_type(&self, id: &ComponentID)
		-> Result<ComponentType, String>
	{
		let mut component = match self.components.find(id) {
			None => return Err(format!("Component with ID {} doesn't exist", id)),
			Some(e) => e
		};

		Ok(component.cmpType.clone())
	}

	pub fn get_entities_iter<'a>(&'a self)
		-> std::collections::hashmap::Keys<'a, EntityID, EntityData>
	{
		self.entities.keys()
	}

	pub fn get_components_iter<'a>(&'a self)
		-> std::collections::hashmap::Keys<'a, ComponentID, Component>
	{
		self.components.keys()
	}

	/*pub fn get_name<'a>(&'a self, id: &EntityID)
		-> Result<Option<&'a str>, String>
	{
		let mut entity = match self.entities.find(id) {
			None => return Err(format!("Entity with ID {} doesn't exist", id)),
			Some(e) => e
		};

		Ok(entity.name.map(|e| e.as_slice()))
	}*/

	/*pub fn get_components_of_native_type(&self, typename: &str) {

	}*/

	fn link_component_data(&mut self, component: &ComponentID, link_to: &ComponentID) {
		unimplemented!()
	}

	fn unlink(&mut self, component: &ComponentID) {
		unimplemented!()
	}

	fn set_component_parent(&mut self, component: &ComponentID, parent: &ComponentID) {
		unimplemented!()
	}

	fn clear_component_parent(&mut self, component: &ComponentID) {
		unimplemented!()
	}
}


#[cfg(test)]
mod tests {
	#[test]
	fn basic() {
		let es = super::EntitiesState::new();

		let id = es.create_entity();
	}
}
