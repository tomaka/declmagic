use entities::{ EntitiesState, EntitiesHelper, EntityID, ComponentID, NativeComponentType };
use std::collections::{ HashSet, HashMap };

use resources::ResourcesLoader;
use self::extern_content::ExternContentSystem;

use log;

mod extern_content;

pub struct MechanicsSystem {
    externContentSystem: ExternContentSystem,
}

impl MechanicsSystem {
    pub fn new<RL: ResourcesLoader + Send + Share>(state: &EntitiesState, loader: RL, log: |log::LogRecord|)
        -> MechanicsSystem
    {
        MechanicsSystem {
            externContentSystem: ExternContentSystem::new(state, loader, |l| log(l))
        }
    }

    pub fn process(&mut self, state: &mut EntitiesState, elapsed: &f64, log: |log::LogRecord|)
    {
        self.externContentSystem.process(state, |l| log(l));
        self.update_spawners(state, elapsed, |l| log(l));
    }

    fn update_spawners(&mut self, state: &mut EntitiesState, elapsed: &f64, log: |log::LogRecord|)
    {
        // getting the list of all sprite displayer components
        let listOfComponents = state.get_visible_native_components("spawner");

        for cmp in listOfComponents.move_iter() {
            let mut nextSpawn = match state.get_as_number(&cmp, "nextSpawn") { Some(v) => v, None => continue };
            let mut interval = match state.get_as_number(&cmp, "interval") { Some(v) => v, None => continue };
            let mut limit = state.get_as_number(&cmp, "limit");

            // detecting infinity
            if interval <= 0.0 && limit.is_none() {
                // TODO: error
                continue;
            }

            nextSpawn -= *elapsed as f64;

            while nextSpawn <= 0.0 && match limit { Some(l) => l >= 1.0, None => true } {
                MechanicsSystem::trigger_spawner(state, &cmp);

                // adding interval
                nextSpawn += interval;

                // reducing limit
                match &mut limit { &Some(ref mut l) => (*l) -= 1.0, &None => () };
            }

            // TODO: detect if FromProperty

            // updating nextSpawn
            state.set(&cmp, "nextSpawn", ::entities::Number(nextSpawn));

            // updating limit
            match limit {
                Some(limit) => { state.set(&cmp, "limit", ::entities::Number(limit)).ok(); },
                None => ()
            };

            // if limit is 0, destroying
            match limit {
                Some(v) if v <= 0.0 => {
                    state.destroy_component(&cmp);
                },
                _ => ()
            };
        }
    }

    /// Spawns an entity on a spawner.
    /// Does not update any of the component's properties.
    fn trigger_spawner(state: &mut EntitiesState, cmp: &ComponentID)
    {
        let prototype = match state.get_as_entity(cmp, "prototype") { Some(v) => v, None => return };

        let newEntity = state.create_entity(None, true);
        state.create_component_from_entity(&newEntity, &prototype, ::std::collections::HashMap::new());
    }
}
