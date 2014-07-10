extern crate declmagic;

use declmagic::resources::dir_loader::DirLoader;

fn main() {
	let loader = DirLoader::new(::std::os::self_exe_path().unwrap().join("resources"));

	declmagic::exec_game(loader);
}
