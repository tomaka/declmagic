## declmagic

**Note: library in heavy development**

Game engine written in Rust with declarative programming mind

[How to use it?](guide/introduction.md)


### Quick usage guide

#### Installation
Add this to the `Cargo.toml` file of your project

    [dependencies.declmagic]
    git = "https://github.com/Tomaka17/declmagic"

#### The Rust code

Your Rust code will mainly consist in giving the hand to the `declmagic` library.

```rust
extern crate declmagic;

use std::path::Path;
use declmagic::resources::dir_loader::DirLoader;

fn main() {
	let loader = DirLoader::new(Path::new("resources"));
	declmagic::exec_game(loader);
}
```

We crate a new `Loader` pointing to the location of the game's data, and start the engine with it.

Your Rust executable can also be used to customize the engine.

#### Game resources

Most of your game will consist of the game's logic, which are in JSON.

See the [guide](guide/introduction.md) to learn how to use it, or check the example bundled with the library.
