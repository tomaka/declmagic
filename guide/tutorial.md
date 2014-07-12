### Tutorial

#### Installation

Add this to the `Cargo.toml` file of your project:

```toml
[dependencies.declmagic]
git = "https://github.com/Tomaka17/declmagic"
```

#### The Rust code

Contrary to other game engines, you don't code your game logic in Rust.

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

In the future it will be possible to extend the engine with Rust code.

#### Game resources

The main file which contains your game is `resources/main.json`.

Let's look at an example file:

```json
[
    {
        "name": "mainCamera",
        "components": [
            {
                "type": "camera",
                "data": { "matrix": [ 0.1, 0, 0, 0, 0, 0.1, 0, 0, 0, 0, 1, 0, -0.333, -1, 0, 1 ], "priority": 1 }
            }
        ]
    },
    {
        "name": "character",
        "components": [
            {
                "type": "position",
                "data": { "x": 12, "y": 12 }
            }
        ]
    }
]
```

The declmagic engine is based upon the classic entities-components architecture.
