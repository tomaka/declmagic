extern crate std;

pub use self::state::{ EntitiesState, Data, EntityID, ComponentID, Number, String, Boolean, List, Entity, FromProperty, Empty };
pub use self::state::{ ComponentType, NativeComponentType, EntityComponentType };
pub use self::state::{ StateError };

use std::collections::HashMap;
use lua::any;

pub mod loader;
mod state;

// TODO: totally rework this trait once associated types are implemented
pub trait EntitiesHelper {
    /// Creates a new empty entity in the state.
    fn create_entity(&mut self, name: Option<String>, visible: bool) -> EntityID;

    /// Destroys an entity.
    ///
    /// This operation will also destroy all the components owned by
    /// this entity and all components whose type is this entity.
    fn destroy_entity(&mut self, id: &EntityID) -> Result<(), StateError>;

    /// Creates a new component of a native type.
    fn create_native_component(&mut self, owner: &EntityID,
                               typename: &str, data: HashMap<String, Data>)
        -> Result<ComponentID, StateError>;

    /// Creates a new component of an entity type.
    fn create_component_from_entity(&mut self, owner: &EntityID, typename: &EntityID,
                                    data: HashMap<String, Data>)
                                    -> Result<ComponentID, StateError>;

    /// Destroys a component.
    fn destroy_component(&mut self, id: &ComponentID) -> Result<(), StateError>;

    /// Modifies an element of a component.
    fn set(&mut self, id: &ComponentID, field: &str, data: Data) -> Result<(), StateError>;

    /// Returns the type of the component.
    fn get_type(&self, id: &ComponentID)
        -> Result<ComponentType, StateError>;

    /// Returns the list of all the components in the state.
    fn get_components_list(&self)
        -> Vec<ComponentID>;

    /// Returns the list of all the components which are visible
    /// and are of the requested native type.
    fn get_visible_native_components(&self, nativetype: &str)
        -> Vec<ComponentID>;
    
    /// Returns the owner of the component.
    fn get_owner(&self, id: &ComponentID)
        -> Result<EntityID, StateError>;

    /// Returns the name of an entity.
    fn get_entity_name<'a>(&'a self, id: &EntityID)
        -> Result<Option<String>, StateError>;

    /// Returns the list of all entities with the given name.
    fn get_entities_by_name<'a>(&'a self, name: &str)
        -> Vec<EntityID>;

    /// Returns true if the entity is visible.
    fn is_entity_visible(&self, id: &EntityID)
        -> Result<bool, StateError>;

    /// Returns true if the component is visible.
    fn is_component_visible(&self, id: &ComponentID)
        -> Result<bool, StateError>;

    /// Sets an entity as parent of another one.
    /// When a component is destroyed, all its children are destroyed it.
    /// If the component already has a parent, it's removed.
    fn set_component_parent(&mut self, component: &ComponentID, parent: &ComponentID)
        -> Result<(), StateError>;

    /// Resets the parent of an entity, breaking the parent-child link.
    fn clear_component_parent(&mut self, component: &ComponentID);

    /// Returns the list of children of a component.
    fn get_component_children(&self, component: &ComponentID)
        -> Result<Vec<ComponentID>, StateError>;

    /// Returns an element of a component.
    fn get<'a>(&'a self, id: &ComponentID, field: &str)
        -> Result<&'a Data, StateError>;

    /// Returns the value of a property of an entity.
    /// Reads the appropriate "property" or "propertyView" component.
    /// Returns Ok(Empty) if the property is not found.
    fn get_property_value(&self, id: &EntityID, propname: &str)
        -> Result<Data, StateError>
    {
        let value = self
            .get_visible_native_components("property").move_iter()
            .chain(self.get_visible_native_components("propertyView").move_iter())
            .filter(|c| &self.get_owner(c).unwrap() == id)
            .filter(|c|
                match self.get(c, "property").map(|c| c.as_string()) {
                    Ok(Some(n)) => n.as_slice() == propname,
                    _ => false
                })
            .max_by(|c|
                match self.get(c, "priority").map(|c| c.as_number()) {
                    Ok(Some(n)) => ((n * 1000f64) as int),
                    _ => 1000
                })
            .and_then(|c| {
                let cmpType = match self.get_type(&c) {
                    Ok(NativeComponentType(t)) => t.clone(),
                    _ => fail!()
                };

                match cmpType.as_slice() {
                    "property" =>
                        match self.get(&c, "value") {
                            Ok(&FromProperty(_)) => None,
                            Ok(n) => Some(n.clone()),
                            _ => None
                        },
                    "propertyView" => {
                        let script = match self.get(&c, "script") {
                            Ok(&FromProperty(_)) => return None,
                            Ok(&String(ref n)) => n.clone(),
                            _ => return None
                        };
                        match ::script::execute(self, &c, &script) {
                            Ok(any::Number(val)) => Some(Number(val)),
                            Ok(any::String(val)) => Some(String(val)),
                            Ok(any::Boolean(val)) => Some(Boolean(val)),
                            Ok(_) => unimplemented!(),
                            Err(e) => fail!("{}", e)
                        }
                    },
                    _ => fail!()
                }
            });

        match value {
            Some(v) => Ok(v),
            None => Ok(Empty)
        }
    }

    /// Gets the value of a field of the component.
    /// Resolves it if the field is "FromProperty".
    fn get_and_resolve(&self, id: &ComponentID, field: &str)
        -> Result<Data, StateError>
    {
        let propname = match try!(self.get(id, field)) {
            &FromProperty(ref p) => p,
            a => return Ok(a.clone())
        };

        let owner = try!(self.get_owner(id));

        self.get_property_value(&owner, propname.as_slice())
    }

    fn get_as_number(&self, id: &ComponentID, field: &str)
        -> Option<f64>
    {
        match self.get_and_resolve(id, field) {
            Ok(Number(n)) => Some(n),
            _ => None
        }
    }

    fn get_as_string(&self, id: &ComponentID, field: &str)
        -> Option<String>
    {
        match self.get_and_resolve(id, field) {
            Ok(String(s)) => Some(s),
            _ => None
        }
    }

    fn get_as_boolean(&self, id: &ComponentID, field: &str)
        -> Option<bool>
    {
        match self.get_and_resolve(id, field) {
            Ok(Boolean(b)) => Some(b),
            _ => None
        }
    }

    fn get_as_entity(&self, id: &ComponentID, field: &str)
        -> Option<EntityID>
    {
        match self.get_and_resolve(id, field) {
            Ok(Entity(e)) => Some(e),
            _ => None
        }
    }
}
