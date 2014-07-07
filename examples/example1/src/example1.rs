extern crate declmagic;

use declmagic::resources::dir_loader::DirLoader;

fn main() {
	let loader = DirLoader::new(Path::new("resources"));

	declmagic::exec_game(loader);
}
