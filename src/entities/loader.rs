use resources::ResourcesLoader;
use serialize::json;
use std::ascii::StrAsciiExt;
use std::collections::{ HashSet, HashMap };
use super::EntitiesState;
use super::EntitiesHelper;
use super::EntityID;
use super::ComponentID;

/// Loads entities into an EntitiesState
pub fn load(loader: &ResourcesLoader, resourceName: &str, output: &mut EntitiesState)
    -> Result<Vec<EntityID>, LoaderError>
{
    let mut context = LoadContext {
        loader: loader,
        output: output,
        loadedDocs: HashSet::new(),
    };

    load_impl(&mut context, resourceName)
}

struct LoadContext<'a> {
    loader: &'a ResourcesLoader,
    output: &'a mut EntitiesState,
    loadedDocs: HashSet<String>,
}

enum LoaderError {
    IoError(::std::io::IoError),
    SyntaxError(::serialize::json::ParserError),
    StateError(super::StateError),
    WrongDataStructure(String),
}

impl ::std::fmt::Show for LoaderError {
    fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::FormatError> {
        match self {
            &IoError(ref err) => err.fmt(formatter),
            &SyntaxError(ref err) => err.fmt(formatter),
            &StateError(ref err) => err.fmt(formatter),
            &WrongDataStructure(ref err) => format!("WrongDataStructure({})", err).fmt(formatter),
        }
    }
}

fn load_impl(context: &mut LoadContext, resourceName: &str)
    -> Result<Vec<EntityID>, LoaderError>
{
    // checking that the doc has not already been loaded
    if context.loadedDocs.contains_equiv(&resourceName) {
        return Ok(vec!());
    }
    context.loadedDocs.insert(resourceName.to_string());

    // if not, loading the resource
    let mut resource = match context.loader.load(resourceName) {
        Ok(r) => ::std::io::BufferedReader::new(r),
        Err(err) => return Err(IoError(err))
    };

    // building the JSON object
    let data = match json::Builder::new(resource.chars().map(|c| c.unwrap())).build() {
        Ok(d) => d,
        Err(err) => return Err(SyntaxError(err))
    };

    // loading the doc into the entities state
    load_all(context, resourceName, &data)
}

fn load_all(context: &mut LoadContext, resourceName: &str, doc: &json::Json)
    -> Result<Vec<EntityID>, LoaderError>
{
    match doc {
        &json::List(ref entities) => {
            let mut result = Vec::new();

            for elem in entities.iter() {
                match load_entity(context, resourceName, elem) {
                    Ok(e) => result.push(e),
                    Err(err) => {
                        for e in result.iter() { context.output.destroy_entity(e); }
                        return Err(err);
                    }
                }
            }

            Ok(result)
        },
        _ => return Err(WrongDataStructure(format!("Wrong format for entities document, expected list of entities but got: {}", doc)))
    }
}

fn load_entity(context: &mut LoadContext, resourceName: &str, entity: &json::Json)
    -> Result<EntityID, LoaderError>
{
    match entity {
        &json::Object(ref entityData) => {
            let name = entityData
                .find(&"name".to_string())
                .and_then(|e| e.as_string())
                .map(|e| e.to_string())
                .map(|name| Path::new(resourceName).join(name).as_str().expect("non-utf8 entity name!").to_string());

            let visible = entityData.find(&"visible".to_string()).and_then(|e| e.as_boolean()).unwrap_or(true);
            let entityID = context.output.create_entity(name, visible);

            match entityData.find(&"components".to_string()) {
                Some(cmp) => { 
                    match load_components_list(context, &entityID, cmp) {
                        Ok(_) => (),
                        Err(err) => {
                            context.output.destroy_entity(&entityID);
                            return Err(err);
                        }
                    }
                },
                _ => ()
            };

            Ok(entityID)
        },
        _ => return Err(WrongDataStructure(format!("Wrong format for entity, expected object but got: {}", entity)))
    }
}

fn load_components_list(context: &mut LoadContext, entity: &EntityID, componentsList: &json::Json)
    -> Result<Vec<ComponentID>, LoaderError>
{
    match componentsList {
        &json::List(ref components) => {
            let mut result = Vec::new();

            for elem in components.iter() {
                match load_component(context, entity, elem) {
                    Ok(e) => result.push(e),
                    Err(err) => {
                        for e in result.iter() { context.output.destroy_component(e); }
                        return Err(err);
                    }
                }
            }

            Ok(result)
        },
        _ => return Err(WrongDataStructure(format!("Wrong format for components list, expected list but got: {}", componentsList)))
    }
}

fn load_component(context: &mut LoadContext, entity: &EntityID, component: &json::Json)
    -> Result<ComponentID, LoaderError>
{
    match component {
        &json::Object(ref componentInfos) => {
            let cmptype = match componentInfos.find(&"type".to_string()) {
                Some(t) => t,
                None => return Err(WrongDataStructure(format!("Component does not have a \"type\" field: {}", component)))
            };

            let data = match componentInfos.find(&"data".to_string()) {
                Some(cmp) => { try!(load_component_data(context, cmp)) },
                _ => HashMap::new()
            };

            match cmptype {
                &json::String(ref t) => context.output.create_native_component(entity, t.as_slice(), data).map_err(|err| StateError(err)),
                &json::Object(_) => {
                    match load_data_entry(context, cmptype) {
                        Ok(super::Entity(id)) => context.output.create_component_from_entity(entity, &id, data).map_err(|err| StateError(err)),
                        Ok(_) => return Err(WrongDataStructure(format!("Wrong type for component \"type\" field object, expected entity"))),
                        Err(err) => return Err(err)
                    }
                },
                _ => Err(WrongDataStructure(format!("Wrong format for component \"type\" field, expected string or object")))
            }
        },
        _ => return Err(WrongDataStructure(format!("Wrong format for component, expected object but got {}", component)))
    }
}

fn load_component_data(context: &mut LoadContext, componentData: &json::Json)
    -> Result<HashMap<String, super::Data>, LoaderError>
{
    match componentData {
        &json::Object(ref data) => {
            let mut result = HashMap::new();

            for (key, val) in data.iter() {
                result.insert(key.clone(), try!(load_data_entry(context, val)));
            }

            Ok(result)
        },
        _ => return Err(WrongDataStructure(format!("Wrong format for component data, expected object but got {}", componentData)))
    }
}

fn load_data_entry(context: &mut LoadContext, element: &json::Json)
    -> Result<super::Data, LoaderError>
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
                let val = try!(load_data_entry(context, elem));
                result.push(val);
            }
            super::List(result)
        },
        &json::Object(ref data) => {
            let (key, val) = match data.iter().next() {
                None => return Err(WrongDataStructure(format!("Empty object found for component data element"))),
                Some(a) => a
            };

            if key.as_slice().eq_ignore_ascii_case("prototype") {
                let entityID = context.output.create_entity(None, false);

                match load_components_list(context, &entityID, val) {
                    Ok(_) => (),
                    Err(err) => {
                        context.output.destroy_entity(&entityID);
                        return Err(err);
                    }
                }

                super::Entity(entityID)

            } else if key.as_slice().eq_ignore_ascii_case("entity") {
                let requestedName = match val.as_string() { Some(a) => a, None => return Err(WrongDataStructure(format!("Component data element object of type Entity expects a string"))) };
                super::Entity(try!(load_entity_from_name(context, requestedName)))
                
            } else if key.as_slice().eq_ignore_ascii_case("property") {
                let requestedProp = match val.as_string() { Some(a) => a, None => return Err(WrongDataStructure(format!("Component data element object of type Property expects a string"))) };
                super::FromProperty(requestedProp.to_string())
                
            } else if key.as_slice().eq_ignore_ascii_case("script") {
                let script = match val.as_string() { Some(a) => a, None => return Err(WrongDataStructure(format!("Component data element object of type Script expects a string"))) };
                super::FromScript(script.to_string())
                
            } else {
                return Err(WrongDataStructure(format!("Got invalid key for component data element object: {}", key)));
            }
        },
        _ => return Err(WrongDataStructure(format!("Wrong format for component data element, got {}", element)))
    })
}

fn load_entity_from_name(context: &mut LoadContext, entityName: &str)
    -> Result<EntityID, LoaderError>
{
    let entityNameRefined = {
        let path = Path::new(entityName);
        path.join(path.filename().unwrap()).as_str().expect("non-utf8 entity name!").to_string()
    };

    // first, we check if there is an existing entity with this name
    {
        let entities = context.output.get_entities_by_name(entityName);
        if entities.len() >= 2 {
            return Err(WrongDataStructure(format!("Found multiple entities with the same name: {}", entityName)))
        }
        if entities.len() == 1 {
            return Ok(entities.get(0).clone())
        }
    }

    // first, we check if there is an existing entity with this name
    {
        let entities = context.output.get_entities_by_name(entityNameRefined.as_slice());
        if entities.len() >= 2 {
            return Err(WrongDataStructure(format!("Found multiple entities with the same name: {}", entityNameRefined)))
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
                    match load_impl(context, path) {
                        Ok(l) => break,
                        Err(IoError(_)) => (),
                        Err(err) => return Err(err)
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
        let entities = context.output.get_entities_by_name(entityName);
        if entities.len() >= 2 {
            return Err(WrongDataStructure(format!("Found multiple entities with the same name: {}", entityName)))
        }
        if entities.len() == 1 {
            return Ok(entities.get(0).clone())
        }
    }

    // first, we check if there is an existing entity with this name
    {
        let entities = context.output.get_entities_by_name(entityNameRefined.as_slice());
        if entities.len() >= 2 {
            return Err(WrongDataStructure(format!("Found multiple entities with the same name: {}", entityNameRefined)))
        }
        if entities.len() == 1 {
            return Ok(entities.get(0).clone())
        }
    }

    Err(WrongDataStructure(format!("Unable to load entity named \"{}\"", entityName)))
}
