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
	load_impl(loader, resourceName, output, &mut docs)
}

fn load_impl(loader: &ResourcesLoader, resourceName: &str, output: &mut EntitiesState, loadedDocs: &mut HashSet<String>)
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
	load_all(output, resourceName, &data, loader, loadedDocs)
}

fn load_all(output: &mut EntitiesState, resourceName: &str, doc: &json::Json, loader: &ResourcesLoader, loadedDocs: &mut HashSet<String>)
	-> Result<Vec<EntityID>, String>
{
	match doc {
		&json::List(ref entities) => {
			let mut result = Vec::new();

			for elem in entities.iter() {
				match load_entity(output, resourceName, elem, loader, loadedDocs) {
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

fn load_entity(output: &mut EntitiesState, resourceName: &str, entity: &json::Json, loader: &ResourcesLoader, loadedDocs: &mut HashSet<String>)
	-> Result<EntityID, String>
{
	match entity {
		&json::Object(ref entityData) => {
			let name = entityData
				.find(&"name".to_string())
				.and_then(|e| e.as_string())
				.map(|e| e.to_string())
				.map(|name| Path::new(resourceName).join(name).as_str().expect("non-utf8 entity name!").to_string());

			let visible = entityData.find(&"visible".to_string()).and_then(|e| e.as_boolean()).unwrap_or(true);
			let entityID = output.create_entity(name, visible);

			match entityData.find(&"components".to_string()) {
				Some(cmp) => { 
					match load_components_list(output, &entityID, cmp, loader, loadedDocs) {
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

fn load_components_list(output: &mut EntitiesState, entity: &EntityID, componentsList: &json::Json, loader: &ResourcesLoader, loadedDocs: &mut HashSet<String>)
	-> Result<Vec<ComponentID>, String>
{
	match componentsList {
		&json::List(ref components) => {
			let mut result = Vec::new();

			for elem in components.iter() {
				match load_component(output, entity, elem, loader, loadedDocs) {
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

fn load_component(output: &mut EntitiesState, entity: &EntityID, component: &json::Json, loader: &ResourcesLoader, loadedDocs: &mut HashSet<String>)
	-> Result<ComponentID, String>
{
	match component {
		&json::Object(ref componentInfos) => {
			let cmptype = match componentInfos.find(&"type".to_string()) {
				Some(t) => t,
				None => return Err(format!("Component does not have a \"type\" field: {}", component))
			};

			let data = match componentInfos.find(&"data".to_string()) {
				Some(cmp) => { try!(load_component_data(output, cmp, loader, loadedDocs)) },
				_ => HashMap::new()
			};

			match cmptype {
				&json::String(ref t) => output.create_native_component(entity, t.as_slice(), data).map_err(|err| format!("{}", err)),
				&json::Object(_) => {
					match load_data_entry(output, cmptype, loader, loadedDocs) {
						Ok(super::Entity(id)) => output.create_component_from_entity(entity, &id, data).map_err(|err| format!("{}", err)),
						Ok(_) => return Err(format!("Wrong type for component \"type\" field object, expected entity")),
						Err(err) => return Err(err)
					}
				},
				_ => Err(format!("Wrong format for component \"type\" field, expected string or object"))
			}
		},
		_ => return Err(format!("Wrong format for component, expected object but got {}", component))
	}
}

fn load_component_data(output: &mut EntitiesState, componentData: &json::Json, loader: &ResourcesLoader, loadedDocs: &mut HashSet<String>)
	-> Result<HashMap<String, super::Data>, String>
{
	match componentData {
		&json::Object(ref data) => {
			let mut result = HashMap::new();

			for (key, val) in data.iter() {
				result.insert(key.clone(), try!(load_data_entry(output, val, loader, loadedDocs)));
			}

			Ok(result)
		},
		_ => return Err(format!("Wrong format for component data, expected object but got {}", componentData))
	}
}

fn load_data_entry(output: &mut EntitiesState, element: &json::Json, loader: &ResourcesLoader, loadedDocs: &mut HashSet<String>)
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
				let val = try!(load_data_entry(output, elem, loader, loadedDocs));
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
				let entityID = output.create_entity(None, false);

				match load_components_list(output, &entityID, val, loader, loadedDocs) {
					Ok(_) => (),
					Err(err) => {
						output.destroy_entity(&entityID);
						return Err(err);
					}
				}

				super::Entity(entityID)

			} else if key.as_slice().eq_ignore_ascii_case("entity") {
				let requestedName = match val.as_string() { Some(a) => a, None => return Err(format!("Component data element object of type Entity expects a string")) };
				super::Entity(try!(load_entity_from_name(output, requestedName, loader, loadedDocs)))
				
			} else if key.as_slice().eq_ignore_ascii_case("property") {
				let requestedProp = match val.as_string() { Some(a) => a, None => return Err(format!("Component data element object of type Property expects a string")) };
				super::FromProperty(requestedProp.to_string())
				
			} else {
				return Err(format!("Got invalid key for component data element object: {}", key));
			}
		},
		_ => return Err(format!("Wrong format for component data element, got {}", element))
	})
}

fn load_entity_from_name(output: &mut EntitiesState, entityName: &str, loader: &ResourcesLoader, loadedDocs: &mut HashSet<String>)
	-> Result<EntityID, String>
{
	let entityNameRefined = {
		let path = Path::new(entityName);
		path.join(path.filename().unwrap()).as_str().expect("non-utf8 entity name!").to_string()
	};

	// first, we check if there is an existing entity with this name
	{
		let entities = output.get_entities_by_name(entityName);
		if entities.len() >= 2 {
			return Err(format!("Found multiple entities with the same name: {}", entityName))
		}
		if entities.len() == 1 {
			return Ok(entities.get(0).clone())
		}
	}

	// first, we check if there is an existing entity with this name
	{
		let entities = output.get_entities_by_name(entityNameRefined.as_slice());
		if entities.len() >= 2 {
			return Err(format!("Found multiple entities with the same name: {}", entityNameRefined))
		}
		if entities.len() == 1 {
			return Ok(entities.get(0).clone())
		}
	}


	// trying to load the file with the same name as the entity
	{
		let mut nameAsPath = ::std::path::posix::Path::new(entityName);
		loop {
			match nameAsPath.as_str() {
				 Some(path) => 
					match load_impl(loader, path, output, loadedDocs) {
						Ok(l) => break,
						Err(e) => ()		// TODO: check for error type! If anything else than "resource doesn't exist", return
					},
				 None => ()
			};

			let newPath = nameAsPath.dir_path();
			if match newPath.as_str() { Some(s) => s == ".", None => false } || newPath == nameAsPath { break };
			nameAsPath = newPath;
		}
	}

	// first, we check if there is an existing entity with this name
	{
		let entities = output.get_entities_by_name(entityName);
		if entities.len() >= 2 {
			return Err(format!("Found multiple entities with the same name: {}", entityName))
		}
		if entities.len() == 1 {
			return Ok(entities.get(0).clone())
		}
	}

	// first, we check if there is an existing entity with this name
	{
		let entities = output.get_entities_by_name(entityNameRefined.as_slice());
		if entities.len() >= 2 {
			return Err(format!("Found multiple entities with the same name: {}", entityNameRefined))
		}
		if entities.len() == 1 {
			return Ok(entities.get(0).clone())
		}
	}

	Err(format!("Unable to load entity named \"{}\"", entityName))
}
