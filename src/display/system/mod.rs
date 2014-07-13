use super::managed_display::ManagedDisplay;
use entities::{ EntitiesState, EntitiesHelper, EntityID, ComponentID, NativeComponentType };
use nalgebra::na;
use nalgebra::na::{ Vec3, Eye };
use std::collections::{ HashSet, HashMap };
use std::sync::Arc;
use super::sprite_displayer::SpriteDisplayer;
use super::Drawable;
use physics;
use log;

mod custom_display_system;

pub struct DisplaySystem {
    display: Arc<ManagedDisplay>,
    customDisplay: custom_display_system::CustomDisplaySystem,
    sprites: HashMap<ComponentID, (SpriteDisplayer, String)>
}

impl DisplaySystem {
    pub fn new(display: Arc<ManagedDisplay>, state: &EntitiesState, log: |log::LogRecord|)
        -> DisplaySystem
    {
        //declmagic_info!(logger, "created display system");

        let customDisplaySystem =
            custom_display_system::CustomDisplaySystem::new(display.clone(), state, |l| log(l));

        DisplaySystem {
            display: display.clone(),
            customDisplay: customDisplaySystem,
            sprites: HashMap::new()
        }
    }

    pub fn draw(&mut self, state: &EntitiesState, _: &f64, log: |log::LogRecord|)
    {
        self.update_sprite_displayers(state, |l| log(l));

        let camera = DisplaySystem::get_camera(state).unwrap_or_else(|| {
            log(log::LogRecord::new(log::Warning, format!("No active camera on the scene")));
            Eye::new_identity(4)
        });

        for (cmp, &(ref sprite, _)) in self.sprites.iter() {
            let pos = physics::PhysicsSystem::get_entity_position(state,
                &state.get_owner(cmp).unwrap());
            let translationMatrix = na::Mat4::new(
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                pos.x, pos.y, pos.z, 1.0
            );
            sprite.draw(&(translationMatrix * camera));
        }

        self.customDisplay.draw(state, |l| log(l));
    }

    fn update_sprite_displayers(&mut self, state: &EntitiesState, log: |log::LogRecord|)
    {
        // getting the list of all sprite displayer components
        let listOfComponents = state
            .get_visible_native_components("spriteDisplay").move_iter()
            .collect::<HashSet<ComponentID>>();

        // removing from the list the elements that have disappeared
        {
            let toRemove = self.sprites.keys()
                .filter(|e| !listOfComponents.contains(e.clone()))
                .map(|e| e.clone())
                .collect::<Vec<ComponentID>>();

            for c in toRemove.move_iter() {
                self.sprites.remove(&c);
            }
        }

        // adding elements that are not yet created
        {
            let to_create = listOfComponents.iter()
                .filter(|e| !self.sprites.contains_key(e.clone()))
                .map(|e| e.clone())
                .collect::<Vec<ComponentID>>();

            for component in to_create.move_iter() {
                // getting the name of the texture
                let textureName = match state.get_as_string(&component, "texture") {
                    Some(s) => s,
                    _ => {
                        // TODO: 
                        //declmagic_error!(self.logger,
                        //   "component {} has no valid \"texture\" element", component)
                        continue
                    }
                };

                // inserting in sprites list
                self.sprites.insert(component.clone(), (
                    SpriteDisplayer::new(self.display.clone(), textureName.as_slice())
                        .unwrap(),
                    textureName
                ));
            }
        }

        // updating everything
        for (component, &(ref mut sprite, ref mut currTexName)) in self.sprites.mut_iter() {
            // getting the name of the texture
            let textureName = match state.get_as_string(component, "texture") {
                Some(s) => s,
                _ => {
                    // TODO: 
                    //declmagic_error!(self.logger,
                    //   "component {} has no valid \"texture\" element", component)
                    continue
                }
            };

            if currTexName.as_slice() != textureName.as_slice() {
                sprite.set_resource(textureName.as_slice());
                *currTexName = textureName;
            }

            // getting coordinates
            sprite.set_rectangle_coords(
                state.get_as_number(component, "leftX").map(|n| n as f32),
                state.get_as_number(component, "topY").map(|n| n as f32),
                state.get_as_number(component, "rightX").map(|n| n as f32),
                state.get_as_number(component, "bottomY").map(|n| n as f32)
            );
        }
    }

    /// Returns the camera matrix of the scene.
    pub fn get_camera(state: &EntitiesState)
        -> Option<na::Mat4<f32>>
    {
        let cameraInfos = state
            .get_visible_native_components("camera")
            .move_iter()
            .max_by(|c|
                match state.get_as_number(c, "priority") {
                    Some(n) => (n * 1000f64) as int,
                    _ => 1000
                })
            .and_then(|c| 
                match state.get(&c, "matrix") {
                    Ok(&::entities::List(ref data)) => Some((c, data)),
                    _ => None
                })
            .map(|(c, data)| {
                (c, data.iter()
                    .filter_map(|elem|
                        match elem {
                            &::entities::Number(ref n) => Some(n.clone() as f32),
                            _ => None
                        })
                    .collect::<Vec<f32>>())
            });

        if cameraInfos.is_none() {
            return None;
        }

        let (cameraComponent, matrixData) = cameraInfos.unwrap();
        let matrix = na::Mat4::new(*matrixData.get(0), *matrixData.get(1), *matrixData.get(2),
            *matrixData.get(3), *matrixData.get(4), *matrixData.get(5), *matrixData.get(6),
            *matrixData.get(7), *matrixData.get(8), *matrixData.get(9), *matrixData.get(10),
            *matrixData.get(11), *matrixData.get(12), *matrixData.get(13), *matrixData.get(14),
            *matrixData.get(15));

        let position = physics::PhysicsSystem::get_entity_position(state,
            &state.get_owner(&cameraComponent).unwrap());

        let positionMatrix = na::Mat4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            -position.x, -position.y, -position.z, 1.0
        );

        Some(positionMatrix * matrix)
    }
}
