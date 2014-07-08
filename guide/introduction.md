# Introduction

## Entities and components

Entities represent elements of the game, and are described with components. Usually a game is made of a lot of entities with around one or two to several dozens components.

Regular components are of [one of the native component types](native-components.md).

For example :
```
|- Entity 1 (name: character)
| | - spriteDisplay
| | - position
| | - movement
|
|- Entity 2 (name: camera)
| | - position
| | - camera
| | - inputHandler
```

But components can also be from an **entity type**, in which case they inherit from all the target entity's components.

For example :
```
|- Entity 1 (name: character1)
| | - Entity: templates/character
| | | - spriteDisplay (inherited)
| | - position
| | - movement
|
|- Entity 2 (name: character2)
| | - Entity: templates/character
| | | - spriteDisplay (inherited)
| | - position
| | - movement
|
|- Entity 3 (name: templates/character)
| | - spriteDisplay
```

In this example, the game will consider that entities 1 and 2 have the components `templates/character`, `spriteDisplay`, `position` and `movement`. There is no difference between a regular component and a component inherited from another entity.

If the `templates/character` component gets destroyed, then all its inherited components are destroyed too.

In order to avoid the `spriteDisplay` component of entity 3 to be displayed on the screen, we need to set the **visibility** of this entity to false. The visibility is not inherited.

## The JSON format

All the game's logic are written in JSON. When you save the game, the current state of the game is entirely saved in the same format, including all the game's logic.

This means that in case of a patch in the game's logic, the existing save games and replays will not be patched. This is both a good thing (retro-compatibility is guaranteed) and a bad thing (bugs won't be fixed if the player doesn't start a new game).

Here is an example of a game logic resource:

```json
[
    {
        "name": "character1",
        "components": [
            {
                "type": { "Entity": "templates/unit" }
            },
            {
                "type": "position",
                "data": { "x": 12, "y": 6 }
            }
        ]
    },
    {
        "name": "character2",
        "components": [
            {
                "type": { "Entity": "templates/unit" }
            },
            {
                "type": "position",
                "data": { "x": -5.4, "y": 5 }
            }
        ]
    },
    {
        "name": "templates/unit",
        "visible": false,
        "components": [
            {
                "type": "spriteDisplay",
                "data": { "texture": "textures/orc", "bottomY": 0, "topY": 1.8 }
            },
            {
                "type": "physics",
                "data": { "activated": true }
            }
        ]
    }
]
```

There are several *special* data types in this format:

 - **Entity**: attempts to load an entity
 - **Prototype**: creates a non-visible entity with the given components
 - **Resource**: string corresponding to the content of another resource

The `Entity` type obeys the following rules. First the loader will check if there is an existing entity of the given name in the state. Then it will check if there is an existing entity whose name is the given string with its last part duplicated (eg. if you request "templates/unit", the loader will look for "templates/unit/unit"). Then it will try to load resources whose names correspond to the different parts of the string (eg. if you request "templates/unit", the loader will try to load the "templates/unit" resource and the "templates" resource). After loading, it will check again for entities with the given name and name whose last part is duplicated.

## Inheritance

Inherited components have their data linked to their origin. If the original component is modified, then all the linked components are modified too.

This link breaks if you modify the inherited component itself.

However, the link is never *totally* broken. When you destroy the entity type component, the inherited component will always be destroyed even if it was no longer linked to its origin.
