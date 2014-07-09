extern crate std;

pub use self::state::{ EntitiesState, Data, EntityID, ComponentID, Number, String, Boolean, List, Entity, FromProperty, Empty };
pub use self::state::{ ComponentType, NativeComponentType, EntityComponentType };
pub use self::state::{ StateError };

pub mod loader;
mod state;

pub trait EntitiesHelper {
    fn get<'a>(&'a self, id: &ComponentID, field: &str)
        -> Result<&'a Data, StateError>;

    fn get_and_resolve(&self, id: &ComponentID, field: &str)
        -> Result<Data, StateError>
    {
        match try!(self.get(id, field)) {
            &FromProperty(ref propname) => {
                unimplemented!()
            },
            a => Ok(a.clone())
        }
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
