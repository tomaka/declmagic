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

	pub fn create_entity(&mut self, name: Option<String>, visible: bool)
		-> EntityID
	{
		let id = self.nextEntityID;

		let entity = EntityData {
			name: name,
			visible: visible,

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
		-> Result<(), String>
	{
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
			self.destroy_component(cmp).ok();
		}

		self.entities.remove(id);
		Ok(())
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

	// TODO: better error handling
	pub fn create_component_from_entity(&mut self, owner: &EntityID, typename: &EntityID, data: HashMap<String, Data>)
		-> Result<ComponentID, String>
	{
		let newID = self.nextComponentID;

		let newComponent = Component {
			owner: owner.clone(),
			data: ComponentDataNative(data),
			linkedFrom: Vec::new(),
			parent: None,
			children: Vec::new(),
			cmpType: EntityComponentType(typename.clone())
		};

		(try!(self.get_entity_by_id_mut(owner))).components.push(newID);
		(try!(self.get_entity_by_id_mut(typename))).componentsOfType.push(newID);

		// creating the list of components to inherit
		let componentsToInherit: Vec<ComponentID> = (try!(self.get_entity_by_id(typename))).components.iter().map(|c| c.clone()).collect();

		self.components.insert(newID, newComponent);
		self.nextComponentID = self.nextComponentID + 1;

		// inheriting components
		for cmp in componentsToInherit.move_iter() {
			match self.create_inherited_component(owner, &newID, &cmp) {
				Ok(_) => (),
				Err(err) => {
					self.destroy_component(&newID);
					return Err(err);
				}
			}
		}

		Ok(newID)
	}

	pub fn destroy_component(&mut self, id: &ComponentID)
		-> Result<(), String>
	{
		let children = (try!(self.get_component_by_id(id))).children.clone();
		let linked = (try!(self.get_component_by_id(id))).linkedFrom.clone();

		let parent = (try!(self.get_component_by_id(id))).parent;
		if parent.is_some() {
			let mut p = (try!(self.get_component_by_id_mut(&parent.unwrap())));
			let pos = p.children.iter().position(|c| *c == *id).unwrap();
			p.children.remove(pos);
		}

		for child in children.iter() {
			self.destroy_component(child).ok();
		}

		for cmp in linked.iter() {
			self.destroy_component(cmp).ok();
		}

		// removing from entity
		{
			let owner = (try!(self.get_component_by_id(id))).owner;
			let mut entity = try!(self.get_entity_by_id_mut(&owner));
			let pos = entity.components.iter().position(|e| *e == *id).unwrap();
			entity.components.remove(pos);
		}

		// TODO: remove from componentsOfNativeType

		// removing from components list
		self.components.remove(id);

		Ok(())
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
		match (try!(self.get_component_by_id(id))).data {
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
		let component = match self.components.find(id) {
			None => return Err(format!("Component with ID {} doesn't exist", id)),
			Some(e) => e
		};

		Ok(component.owner)
	}

	pub fn get_type(&self, id: &ComponentID)
		-> Result<ComponentType, String>
	{
		let component = match self.components.find(id) {
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

	pub fn get_entity_name<'a>(&'a self, id: &EntityID)
		-> Result<Option<String>, String>
	{
		Ok((try!(self.get_entity_by_id(id))).name.clone())
	}

	pub fn get_entities_by_name<'a>(&'a self, name: &str)
		-> Vec<EntityID>
	{
		self.entities.iter().filter(|&(_, ref e)| e.name == Some(name.to_string())).map(|(id, _)| id.clone()).collect()
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

	pub fn set_component_parent(&mut self, component: &ComponentID, parent: &ComponentID)
		-> Result<(), String>
	{
		if (try!(self.get_component_by_id_mut(component))).parent.is_some() {
			self.clear_component_parent(component);
		}

		(try!(self.get_component_by_id_mut(component))).parent = Some(parent.clone());
		(try!(self.get_component_by_id_mut(parent))).children.push(component.clone());

		Ok(())
	}

	fn clear_component_parent(&mut self, component: &ComponentID) {
		unimplemented!()
	}

	pub fn get_component_children(&self, component: &ComponentID)
		-> Result<Vec<ComponentID>, String>
	{
		Ok((try!(self.get_component_by_id(component))).children.clone())
	}

	/**
	 * Creates a component inherited from another
	 */
	// TODO: better error handling
	fn create_inherited_component(&mut self, owner: &EntityID, parent: &ComponentID, inherit: &ComponentID)
		-> Result<ComponentID, String>
	{
		let newID = self.nextComponentID;

		let newComponent = Component {
			owner: owner.clone(),
			data: ComponentDataLink(inherit.clone()),
			linkedFrom: Vec::new(),
			parent: Some(parent.clone()),
			children: Vec::new(),
			cmpType: (try!(self.get_component_by_id(inherit))).cmpType.clone()
		};

		(try!(self.get_component_by_id_mut(inherit))).linkedFrom.push(newID);
		(try!(self.get_component_by_id_mut(parent))).children.push(newID);
		(try!(self.get_entity_by_id_mut(owner))).components.push(newID);

		self.components.insert(newID, newComponent);

		self.nextComponentID = self.nextComponentID + 1;

		Ok(newID)
	}

	fn get_entity_by_id<'a>(&'a self, id: &ComponentID)
		-> Result<&'a EntityData, String>
	{
		match self.entities.find(id) {
			None => Err(format!("Entity with ID {} not found", id)),
			Some(c) => Ok(c)
		}
	}

	fn get_entity_by_id_mut<'a>(&'a mut self, id: &ComponentID)
		-> Result<&'a mut EntityData, String>
	{
		match self.entities.find_mut(id) {
			None => Err(format!("Entity with ID {} not found", id)),
			Some(c) => Ok(c)
		}
	}

	fn get_component_by_id<'a>(&'a self, id: &ComponentID)
		-> Result<&'a Component, String>
	{
		match self.components.find(id) {
			None => Err(format!("Component with ID {} not found", id)),
			Some(c) => Ok(c)
		}
	}

	fn get_component_by_id_mut<'a>(&'a mut self, id: &ComponentID)
		-> Result<&'a mut Component, String>
	{
		match self.components.find_mut(id) {
			None => Err(format!("Component with ID {} not found", id)),
			Some(c) => Ok(c)
		}
	}

	pub fn is_component_visible(&self, id: &ComponentID)
		-> Result<bool, String>
	{
		let owner = try!(self.get_owner(id));
		Ok((try!(self.get_entity_by_id(&owner))).visible)
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
