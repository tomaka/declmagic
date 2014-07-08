extern crate std;

use std::collections::HashMap;

/**
 * Identifier of an entity
 */
pub type EntityID = uint;

/**
 * Identifier of a component
 */
pub type ComponentID = uint;

/**
 * An error while doing an operation on the state
 */
#[deriving(Show)]
pub enum StateError {
    EntityNotFound(EntityID),
    ComponentNotFound(ComponentID),
    FieldDoesNotExist(ComponentID, String)
}

/**
 * 
 */
#[unstable]
pub struct EntitiesState {
    components: HashMap<ComponentID, Component>,
    entities: HashMap<EntityID, EntityData>,

    next_component_id: ComponentID,
    next_entity_id: EntityID,

    components_of_native_type: HashMap<String, ComponentID>
}

struct EntityData {
    // name of the entity with its path as prefix
    name: Option<String>,
    visible: bool,

    // components owned by the entity
    components: Vec<ComponentID>,

    // components whose type is the entity
    components_of_type: Vec<ComponentID>,

    // list of parameters of the current entity
    default_parameters: HashMap<String, Data>
}

struct Component {
    owner: EntityID,

    cmp_type: ComponentType,

    data: ComponentData,

    // all components in this list have their data as a ComponentDataLink to this one
    linked_from: Vec<ComponentID>,

    parent: Option<ComponentID>,
    // when a component is destroyed, all children are destroyed too
    children: Vec<ComponentID>
}

enum ComponentData {
    ComponentDataNative(HashMap<String, Data>),
    ComponentDataLink(ComponentID)
}

/**
 * Type of a component
 */
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
    FromProperty(String),
    Empty
}

impl EntitiesState {
    /**
     * Builds a new empty state
     */
    pub fn new() -> EntitiesState
    {
        EntitiesState {
            components: HashMap::new(),
            entities: HashMap::new(),
            next_component_id: 1,
            next_entity_id: 1,
            components_of_native_type: HashMap::new()
        }
    }

    /**
     * Creates a new empty entity in the state
     */
    pub fn create_entity(&mut self, name: Option<String>, visible: bool)
        -> EntityID
    {
        let id = self.next_entity_id;

        let entity = EntityData {
            name: name,
            visible: visible,

            components: Vec::new(),

            components_of_type: Vec::new(),
            default_parameters: std::collections::HashMap::new()
        };

        self.entities.insert(id, entity);
        self.next_entity_id += 1;
        id
    }

    /**
     * Destroys an entity
     * This operation will also destroy all the components owned by this entity and all components whose type is this entity
     */
    pub fn destroy_entity(&mut self, id: &EntityID)
        -> Result<(), StateError>
    {
        let components_list = {
            let entity = try!(self.get_entity_by_id(id));

            if entity.components_of_type.len() != 0 {
                unimplemented!()
                //return Err(format!("Cannot destroy entity with ID {} (name: {}) because it has components of its type", id, entity.name));
            }

            entity.components.clone()
        };

        for cmp in components_list.iter() {
            // ignoring error from destroying component because we don't want to recreate them
            self.destroy_component(cmp).ok();
        }

        self.entities.remove(id);
        Ok(())
    }

    /**
     * Creates a new component of native type
     */
    pub fn create_native_component(&mut self, owner: &EntityID, typename: &str, data: HashMap<String, Data>)
        -> Result<ComponentID, StateError>
    {
        let newID = self.next_component_id;

        let newComponent = Component {
            owner: owner.clone(),
            data: ComponentDataNative(data),
            linked_from: Vec::new(),
            parent: None,
            children: Vec::new(),
            cmp_type: NativeComponentType(typename.to_string())
        };

        (try!(self.get_entity_by_id_mut(owner))).components.push(newID);

        self.components.insert(newID, newComponent);
        self.next_component_id = self.next_component_id + 1;

        self.components_of_native_type.insert(typename.to_string(), newID);

        Ok(newID)
    }

    /**
     * Creates a new component of an entity type
     */
    /// TODO: better error handling
    pub fn create_component_from_entity(&mut self, owner: &EntityID, typename: &EntityID, data: HashMap<String, Data>)
        -> Result<ComponentID, StateError>
    {
        let newID = self.next_component_id;

        let newComponent = Component {
            owner: owner.clone(),
            data: ComponentDataNative(data),
            linked_from: Vec::new(),
            parent: None,
            children: Vec::new(),
            cmp_type: EntityComponentType(typename.clone())
        };

        (try!(self.get_entity_by_id_mut(owner))).components.push(newID);
        (try!(self.get_entity_by_id_mut(typename))).components_of_type.push(newID);

        // creating the list of components to inherit
        let components_to_inherit: Vec<ComponentID> = (try!(self.get_entity_by_id(typename))).components.iter().map(|c| c.clone()).collect();

        self.components.insert(newID, newComponent);
        self.next_component_id = self.next_component_id + 1;

        // inheriting components
        for cmp in components_to_inherit.move_iter() {
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

    /**
     * Destroys a component
     */
    pub fn destroy_component(&mut self, id: &ComponentID)
        -> Result<(), StateError>
    {
        let children = (try!(self.get_component_by_id(id))).children.clone();
        let linked = (try!(self.get_component_by_id(id))).linked_from.clone();

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

        // TODO: remove from components_of_native_type

        // removing from components list
        self.components.remove(id);

        Ok(())
    }

    pub fn set(&mut self, id: &ComponentID, field: &str, data: Data)
        -> Result<(), StateError>
    {
        let mut idIter = id.clone();

        loop {
            let mut component = match self.components.find_mut(&idIter) {
                None => return Err(ComponentNotFound(idIter)),
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
        -> Result<&'a Data, StateError>
    {
        match (try!(self.get_component_by_id(id))).data {
            ComponentDataNative(ref data) => {
                match data.find_equiv(&field) {
                    Some(a) => Ok(a),
                    None => Err(FieldDoesNotExist(id.clone(), field.to_string()))
                }
            },
            ComponentDataLink(c) => {
                self.get(&c, field)
            }
        }
    }

    pub fn get_owner(&self, id: &ComponentID)
        -> Result<EntityID, StateError>
    {
        Ok((try!(self.get_component_by_id(id))).owner)
    }

    pub fn get_type(&self, id: &ComponentID)
        -> Result<ComponentType, StateError>
    {
        Ok((try!(self.get_component_by_id(id))).cmp_type.clone())
    }

    pub fn get_entities_iter<'a>(&'a self)
        -> std::collections::hashmap::Keys<'a, EntityID, EntityData>
    {
        self.entities.keys()
    }

    pub fn get_entity_name<'a>(&'a self, id: &EntityID)
        -> Result<Option<String>, StateError>
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
    
    pub fn is_component_visible(&self, id: &ComponentID)
        -> Result<bool, StateError>
    {
        let owner = try!(self.get_owner(id));
        Ok((try!(self.get_entity_by_id(&owner))).visible)
    }

    pub fn set_component_parent(&mut self, component: &ComponentID, parent: &ComponentID)
        -> Result<(), StateError>
    {
        if (try!(self.get_component_by_id_mut(component))).parent.is_some() {
            self.clear_component_parent(component);
        }

        (try!(self.get_component_by_id_mut(component))).parent = Some(parent.clone());
        (try!(self.get_component_by_id_mut(parent))).children.push(component.clone());

        Ok(())
    }

    pub fn clear_component_parent(&mut self, component: &ComponentID) {
        unimplemented!()
    }

    pub fn get_component_children(&self, component: &ComponentID)
        -> Result<Vec<ComponentID>, StateError>
    {
        Ok((try!(self.get_component_by_id(component))).children.clone())
    }

    /**
     * Creates a component inherited from another
     */
    // TODO: better error handling
    fn create_inherited_component(&mut self, owner: &EntityID, parent: &ComponentID, inherit: &ComponentID)
        -> Result<ComponentID, StateError>
    {
        let newID = self.next_component_id;

        let newComponent = Component {
            owner: owner.clone(),
            data: ComponentDataLink(inherit.clone()),
            linked_from: Vec::new(),
            parent: Some(parent.clone()),
            children: Vec::new(),
            cmp_type: (try!(self.get_component_by_id(inherit))).cmp_type.clone()
        };

        (try!(self.get_component_by_id_mut(inherit))).linked_from.push(newID);
        (try!(self.get_component_by_id_mut(parent))).children.push(newID);
        (try!(self.get_entity_by_id_mut(owner))).components.push(newID);

        self.components.insert(newID, newComponent);

        self.next_component_id = self.next_component_id + 1;

        Ok(newID)
    }

    fn get_entity_by_id<'a>(&'a self, id: &ComponentID)
        -> Result<&'a EntityData, StateError>
    {
        match self.entities.find(id) {
            None => Err(EntityNotFound(id.clone())),
            Some(c) => Ok(c)
        }
    }

    fn get_entity_by_id_mut<'a>(&'a mut self, id: &ComponentID)
        -> Result<&'a mut EntityData, StateError>
    {
        match self.entities.find_mut(id) {
            None => Err(EntityNotFound(id.clone())),
            Some(c) => Ok(c)
        }
    }

    fn get_component_by_id<'a>(&'a self, id: &ComponentID)
        -> Result<&'a Component, StateError>
    {
        match self.components.find(id) {
            None => Err(ComponentNotFound(id.clone())),
            Some(c) => Ok(c)
        }
    }

    fn get_component_by_id_mut<'a>(&'a mut self, id: &ComponentID)
        -> Result<&'a mut Component, StateError>
    {
        match self.components.find_mut(id) {
            None => Err(ComponentNotFound(id.clone())),
            Some(c) => Ok(c)
        }
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::EntitiesState;

    #[test]
    fn basic() {
        let mut state = EntitiesState::new();

        let eID = state.create_entity(Some(format!("myname")), true);

        let cmpID = state.create_native_component(&eID, "test", HashMap::new());
    }
}
