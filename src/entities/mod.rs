extern crate std;

pub use self::state::{ EntitiesState, Data, EntityID, ComponentID, Number, String, Boolean, List, Entity, FromProperty, Empty };
pub use self::state::{ ComponentType, NativeComponentType, EntityComponentType };
pub use self::state::{ StateError };

use rust_hl_lua::any;

pub mod loader;
mod state;

pub trait EntitiesHelper {
    /**
     * Returns the type of the component
     */
    fn get_type(&self, id: &ComponentID)
        -> Result<ComponentType, StateError>;

    /**
     * Returns the list of all the components in the state
     */
    fn get_components_list(&self)
        -> Vec<ComponentID>;
    
    /**
     * Returns the owner of the component
     */
    fn get_owner(&self, id: &ComponentID)
        -> Result<EntityID, StateError>;

    /**
     * Returns an element of a component
     */
    fn get<'a>(&'a self, id: &ComponentID, field: &str)
        -> Result<&'a Data, StateError>;

    fn get_property_value(&self, id: &EntityID, propname: &str)
        -> Result<Data, StateError>
    {
        let value = self
            .get_components_list().move_iter()
            .filter(|c| &self.get_owner(c).unwrap() == id)
            .filter(|c| match self.get_type(c) { Ok(NativeComponentType(t)) => t.as_slice() == "property" || t.as_slice() == "propertyView", _ => false })
            .filter(|c| match self.get(c, "property") { Ok(&String(ref n)) => n.as_slice() == propname, _ => false })
            .max_by(|c| match self.get(c, "priority") { Ok(&::entities::Number(ref n)) => (((*n) * 1000f64) as int), _ => 1000 })
            .and_then(|c| {
                let cmpType = match self.get_type(&c) {
                    Ok(NativeComponentType(t)) => t.clone(),
                    _ => fail!()
                };

                match cmpType.as_slice() {
                    "property" =>
                        match self.get(&c, "value") { Ok(&FromProperty(_)) => None, Ok(n) => Some(n.clone()), _ => None },
                    "propertyView" => {
                        let script = match self.get(&c, "script") { Ok(&FromProperty(_)) => return None, Ok(&String(ref n)) => n.clone(), _ => return None };
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
