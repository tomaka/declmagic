#![feature(macro_rules)]
#![feature(globs)]
#![allow(non_camel_case_types)]

extern crate collections;
extern crate gl;
extern crate libc;
extern crate stdlog = "log";
extern crate glob;
extern crate nalgebra;
extern crate ncollide = "ncollide2df32";
extern crate nphysics = "nphysics2df32";
extern crate lua = "rust-hl-lua";
extern crate serialize;
extern crate stb_image;
extern crate sync;

use std::sync::Arc;
use display::Drawable;

pub mod entities;
pub mod log;
pub mod resources;

mod config;
mod display;
mod input;
mod mechanics;
mod physics;
mod script;
mod threaded_executer;

pub trait GameSystem {
    fn process(&mut self, state: &mut entities::EntitiesState, elapsed: &f64);
}

pub struct Game {
    display: Arc<display::managed_display::ManagedDisplay>,

    state: entities::EntitiesState,
    loader: Box<resources::ResourcesLoader + Send + Share>,

    displaySystem: display::DisplaySystem,
    inputSystem: input::InputSystem,
    physicsSystem: physics::PhysicsSystem,
    mechanicsSystem: mechanics::MechanicsSystem,

    thirdPartySystems: Vec<Box<GameSystem>>
}

impl Game {
    pub fn new<RL: resources::ResourcesLoader+Send+Share>(resources: RL)
        -> Game
    {
        let display = Arc::new(display::managed_display::ManagedDisplay::new(display::raw::Display::new(1024, 768, "Game"), box resources.clone() as Box<resources::ResourcesLoader+Send+Share>));

        let mut state = entities::EntitiesState::new();
        entities::loader::load(&resources, "main", &mut state).unwrap();

        let displaySystem = display::DisplaySystem::new(display.clone(), &state, |_|{});
        let inputSystem = input::InputSystem::new(&state, |_|{});
        let physicsSystem = physics::PhysicsSystem::new(&state, |_|{});
        let mechanicsSystem = mechanics::MechanicsSystem::new(&state, resources.clone(), |_|{});

        Game {
            display: display.clone(),

            state: state,
            loader: box resources as Box<resources::ResourcesLoader + Send + Share>,

            displaySystem: displaySystem,
            inputSystem: inputSystem,
            physicsSystem: physicsSystem,
            mechanicsSystem: mechanicsSystem,

            thirdPartySystems: Vec::new()
        }
    }

    pub fn add_system<S: GameSystem + 'static>(&mut self, system: S)
    {
        self.thirdPartySystems.push(box system as Box<GameSystem>)
    }

    pub fn exec(mut self) {
        let mut timer = ::std::io::timer::Timer::new().unwrap();
        let period: f64 = 1.0 / 60.0;
        let timerPeriod = timer.periodic((period * 1000.0) as u64);

        'mainLoop: loop {
            let mut inputMessages = Vec::new();

            loop {
                match self.display.recv() {
                    Some(display::raw::Closed) => break 'mainLoop,
                    Some(display::raw::Input(msg)) => inputMessages.push(msg),
                    Some(_) => continue,
                    None => break
                };
            }

            self.inputSystem.process(&mut self.state, &period, inputMessages.as_slice(), |_|{});
            self.physicsSystem.process(&mut self.state, &period, |_|{});
            self.mechanicsSystem.process(&mut self.state, &period, |_|{});
            self.displaySystem.draw(&mut self.state, &period, |_|{});

            for system in self.thirdPartySystems.mut_iter() {
                system.process(&mut self.state, &period)
            }

            self.display.swap_buffers();
            timerPeriod.recv();
        }
    }
}

pub fn exec_game<RL: resources::ResourcesLoader+Send+Share>(resources: RL) {
    let game = Game::new(resources);
    game.exec();
}
