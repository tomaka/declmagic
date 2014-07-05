extern crate std;

use resources::ResourcesLoader;
use serialize::json;
use std::ascii::StrAsciiExt;
use std::collections::{ HashSet, HashMap };
use super::EntitiesState;
use super::EntityID;
use super::ComponentID;

pub fn load(loader: &ResourcesLoader, resourceName: &str, output: &mut EntitiesState)
	-> Result<Vec<EntityID>, String>
{
	let mut docs = HashSet::new();
	loadImpl(loader, resourceName, output, &mut docs)
}

fn loadImpl(loader: &ResourcesLoader, resourceName: &str, output: &mut EntitiesState, loadedDocs: &mut HashSet<String>)
	-> Result<Vec<EntityID>, String>
{
	// checking that the doc has not already been loaded
	if loadedDocs.contains_equiv(&resourceName) {
		return Ok(vec!());
	}
	loadedDocs.insert(resourceName.to_string());

	// if not, loading the resource
	let mut resource = match loader.load(resourceName) {
		Ok(r) => std::io::BufferedReader::new(r),
		Err(err) => return Err(format!("{}", err))
	};

	// building the JSON object
	let data = match json::Builder::new(resource.chars().map(|c| c.unwrap())).build() {
		Ok(d) => d,
		Err(err) => return Err(format!("{}", err))
	};

	// loading the doc into the entities state
	loadAll(output, &data, loader, loadedDocs)
}

fn loadAll(output: &mut EntitiesState, doc: &json::Json, loader: &ResourcesLoader, loadedDocs: &mut HashSet<String>)
	-> Result<Vec<EntityID>, String>
{
	match doc {
		&json::List(ref entities) => {
			let mut result = Vec::new();

			for elem in entities.iter() {
				match loadEntity(output, elem, loader, loadedDocs) {
					Ok(e) => result.push(e),
					Err(err) => {
						for e in result.iter() { output.destroy_entity(e); }
						return Err(err);
					}
				}
			}

			Ok(result)
		},
		_ => return Err(format!("Wrong format for entities document, expected list of entities but got: {}", doc))
	}
}

fn loadEntity(output: &mut EntitiesState, entity: &json::Json, loader: &ResourcesLoader, loadedDocs: &mut HashSet<String>)
	-> Result<EntityID, String>
{
	match entity {
		&json::Object(ref entityData) => {
			let name = entityData.find(&"name".to_string()).and_then(|e| e.as_string()).map(|e| e.to_string());
			let entityID = output.create_entity(name);

			match entityData.find(&"components".to_string()) {
				Some(cmp) => { 
					match loadComponentsList(output, &entityID, cmp, loader, loadedDocs) {
						Ok(_) => (),
						Err(err) => {
							output.destroy_entity(&entityID);
							return Err(err);
						}
					}
				},
				_ => ()
			};

			Ok(entityID)
		},
		_ => return Err(format!("Wrong format for entity, expected object but got: {}", entity))
	}
}

fn loadComponentsList(output: &mut EntitiesState, entity: &EntityID, componentsList: &json::Json, loader: &ResourcesLoader, loadedDocs: &mut HashSet<String>)
	-> Result<Vec<ComponentID>, String>
{
	match componentsList {
		&json::List(ref components) => {
			let mut result = Vec::new();

			for elem in components.iter() {
				match loadComponent(output, entity, elem, loader, loadedDocs) {
					Ok(e) => result.push(e),
					Err(err) => {
						for e in result.iter() { output.destroy_component(e); }
						return Err(err);
					}
				}
			}

			Ok(result)
		},
		_ => return Err(format!("Wrong format for components list, expected list but got: {}", componentsList))
	}
}

fn loadComponent(output: &mut EntitiesState, entity: &EntityID, component: &json::Json, loader: &ResourcesLoader, loadedDocs: &mut HashSet<String>)
	-> Result<ComponentID, String>
{
	match component {
		&json::Object(ref componentInfos) => {
			let cmptype = match componentInfos.find(&"type".to_string()).and_then(|e| e.as_string()) {
				Some(t) => t,
				None => return Err(format!("Component does not have a \"type\" field: {}", component))
			};

			let data = match componentInfos.find(&"data".to_string()) {
				Some(cmp) => { try!(loadComponentData(output, cmp, loader, loadedDocs)) },
				_ => HashMap::new()
			};

			output.create_native_component(entity, cmptype, data)
		},
		_ => return Err(format!("Wrong format for component, expected object but got {}", component))
	}
}

fn loadComponentData(output: &mut EntitiesState, componentData: &json::Json, loader: &ResourcesLoader, loadedDocs: &mut HashSet<String>)
	-> Result<HashMap<String, super::Data>, String>
{
	match componentData {
		&json::Object(ref data) => {
			let mut result = HashMap::new();

			for (key, val) in data.iter() {
				result.insert(key.clone(), try!(loadComponentDataElement(output, val, loader, loadedDocs)));
			}

			Ok(result)
		},
		_ => return Err(format!("Wrong format for component data, expected object but got {}", componentData))
	}
}

fn loadComponentDataElement(output: &mut EntitiesState, element: &json::Json, loader: &ResourcesLoader, loadedDocs: &mut HashSet<String>)
	-> Result<super::Data, String>
{
	Ok(match element {
		&json::String(ref data) => {
			super::String(data.clone())
		},
		&json::Number(ref data) => {
			super::Number(data.clone())
		},
		&json::Boolean(ref data) => {
			super::Boolean(data.clone())
		},
		&json::List(ref elems) => {
			let mut result = Vec::new();
			for elem in elems.iter() {
				let val = try!(loadComponentDataElement(output, elem, loader, loadedDocs));
				result.push(val);
			}
			super::List(result)
		},
		&json::Object(ref data) => {
			let (key, val) = match data.iter().next() {
				None => return Err(format!("Empty object found for component data element")),
				Some(a) => a
			};

			if key.as_slice().eq_ignore_ascii_case("prototype") {
				let entityID = output.create_entity(None);

				match loadComponentsList(output, &entityID, val, loader, loadedDocs) {
					Ok(_) => (),
					Err(err) => {
						output.destroy_entity(&entityID);
						return Err(err);
					}
				}

				super::Entity(entityID)

			} else {
				return Err(format!("Got invalid key for component data element object: {}", key));
			}
		},
		_ => return Err(format!("Wrong format for component data element, got {}", element))
	})
}
