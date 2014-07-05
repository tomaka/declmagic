## declmagic

**Note: library in heavy development, the example below doesn't work entirely**

Game engine written in Rust with declarative programming mind


### How to install it?

Add this to the `Cargo.toml` file of your project

    [dependencies.declmagic]
    git = "https://github.com/Tomaka17/declmagic"


### Example usage

This is the code in your main executable:

	extern crate declmagic;

	use std::path::Path;
	use declmagic::resources::dir_loader::DirLoader;

	fn main() {
		let loader = DirLoader::new(Path::new("resources"));

		declmagic::exec_game(loader);
	}

Now all you need to do is populate the `resources` directory.

The main file of your game is `resources/main.json`. For example:

```json
[
    {
        "name": "mainCamera",
        "components": [
            {
                "type": "camera",
                "data": {
                	"matrix": [ 0.1, 0, 0, 0, 0, 0.1, 0, 0, 0, 0, 1, 0, -0.333, -1, 0, 1 ],
                	"priority": 1
                }
            }
        ]
    },
    {
        "name": "character",
        "components": [
            {
                "type": "inputHandler",
                "data": {
                    "element": "Right",
                    "prototypeWhilePressed": {
                    	"Prototype": [
					        {
					            "type": "movement",
					            "data": { "x": 1, "y": 0 }
					        },
                    	]
                    }
                }
            },
            {
                "type": "inputHandler",
                "data": {
                    "element": "Left",
                    "prototypeWhilePressed": {
                    	"Prototype": [
					        {
					            "type": "movement",
					            "data": { "x": -1, "y": 0 }
					        },
                    	]
                    }
                }
            },
            {
                "type": "position",
                "data": { "x": 8, "y": 10 }
            },
            {
                "type": "spriteDisplay",
                "data": { "texture": "character.png", "bottomY": 0, "topY": 1, "leftX": -0.5, "rightX": 0.5 }
            },
            {
                "type": "physics",
                "data": { "activated": true }
            }
        ]
    },
]
```

You also need to add a `resources/character.png` file or you won't see anything.
