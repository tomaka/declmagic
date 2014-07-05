#![feature(macro_rules)]
#![feature(globs)]
#![allow(non_camel_case_types)]

extern crate collections;
extern crate gl;
extern crate libc;
extern crate log;
extern crate glob;
extern crate nalgebra;
extern crate ncollide = "ncollide2df32";
extern crate nphysics = "nphysics2df32";
extern crate rust_hl_lua;
extern crate serialize;
extern crate stb_image;
extern crate sync;

use std::sync::{ Arc, Mutex };
use display::Drawable;

pub mod resources;

mod config;
mod display;
mod entities;
mod input;
mod physics;
mod script;
mod threaded_executer;

pub fn exec_game<RL: resources::ResourcesLoader + Send>(resources: RL) {
	let resources = Arc::new(Mutex::new(box resources as Box<resources::ResourcesLoader+Send>));

	let display = Arc::new(display::managed_display::ManagedDisplay::new(display::raw::Display::new(1024, 768, "Game"), resources.clone()));

	let mut timer = ::std::io::timer::Timer::new().unwrap();
	let period = 1000 / 60;
	let timerPeriod = timer.periodic(period);

	let mut state = entities::EntitiesState::new();

	let lock = resources.lock();
	entities::loader::load(*lock, "main", &mut state).unwrap();

	let mut displaySystem = display::DisplaySystem::new(display.clone(), &state);
	let mut inputSystem = input::InputSystem::new(&state);
	let mut physicsSystem = physics::PhysicsSystem::new(&state);
	let mut scriptSystem = script::ScriptSystem::new(&state);

	'mainLoop: loop {
		let mut inputMessages = Vec::new();

		loop {
			match display.recv() {
				Some(display::raw::Closed) => break 'mainLoop,
				Some(display::raw::Input(msg)) => inputMessages.push(msg),
				Some(_) => continue,
				None => break
			};
		}

		inputSystem.process(&mut state, &period, inputMessages.as_slice());
		physicsSystem.process(&mut state, &period);
		scriptSystem.process(&mut state, &period);
		displaySystem.draw(&state, &period);

		display.swap_buffers();
		timerPeriod.recv();
	}

}
