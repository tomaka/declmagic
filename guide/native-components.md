# Native components

## Common

### position

Determines the position of the entity.

If there are multiple `position` components, the position of the entity is the sum of all the components.

The values in this component are updated when the game runs if `autoupdate` is true.

```rust
{
	"type": "position",
	"data": {
		"x": <number (optional)>,
		"y": <number (optional)>,
		"z": <number (optional)>,
		"autoupdate": <boolean (optional, default true)>
	}
}
```


## Display

### camera

Specifies a camera to be used.

```rust
{
	"type": "camera",
	"data": {
		"matrix": [ <m11>, <m12>, <m13>, <m14>, <m21>, <m22>, <m23>, <m24>, <m31>, <m32>, <m33>, <m34>, <m41>, <m42>, <m43>, <m44> ],
		"priority": <number (optional, default 1)>
	}
}
```


 - `matrix`: the matrix to apply to the whole scene when rendered with this camera
 - `priority`: if there are multiple camera components accross the whole state, the one with the highest priority wins


The `position` of the camera entity is also taken into account.
If you move the camera entity, all the elements on the scene will move.

When a scene is being rendered, the `camera` component with the highest priority is chosen.
Its position is then applied to all the elements, and then its matrix is applied


### spriteDisplay

Displays a 2D sprite at the entity's position.

```rust
{
	"type": "spriteDisplay",
	"data": {
		"texture": <string>,
		"topY": <number (optional)>,
		"leftX": <number (optional)>,
		"bottomY": <number (optional)>,
		"rightX": <number (optional)>
	}
}
```


 - `texture`: name of the resource to display
 - `topY`: y coordinate of the top of the sprite, relative to the entity's position
 - `leftX`: x coordinate of the left of the sprite, relative to the entity's position
 - `bottomY`: y coordinate of the bottom of the sprite, relative to the entity's position
 - `rightX`: x coordinate of the right of the sprite, relative to the entity's position


If some of the coordinates are not specified, they will be automatically determined depending on the others.


## Input

### clickBox

Adds a box where this entity will interact with the mouse.

```rust
{
	"type": "clickBox",
	"data": {
		"topY": <number>,
		"leftX": <number)>,
		"bottomY": <number>,
		"rightX": <number>
	}
}
```

The coordinates are added to the entity's position.

### hoverHandler

Handles the user's mouse hovering the entity.

```rust
{
	"type": "hoverHandler",
	"data": {
		"scriptOnEnter": <string (optional)>,
		"scriptOnLeave": <string (optional)>,
		"prototype": <entity (optional)>
	}
}
```

 - `scriptOnEnter`: script to execute when the mouse starts hovering the element
 - `scriptOnLeave`: script to execute when the mouse leaves the clickbox or starts hovering another element
 - `prototype`: entity to inherit from when the element is being hovered

### inputHandler

Allows handling of a user input.

```rust
{
	"type": "inputHandler",
	"data": {
		"element": <string>,
		"script": <string (optional)>,
		"prototypeWhilePressed": <entity (optional)>
	}
}
```

 - `element`: name of the element that is to be handled (eg. "A", "B", "Button0", etc.)
 - `script`: script to execute every time the element is pressed, released or moved
 - `prototypeWhilePressed`: entity to inherit from when the element is down


## Physics

### movement

The movement of the entity.

If there are multiple `movement` components, the movement of the entity is the sum of all the components.

The values in this component are updated when the game runs.

```rust
{
	"type": "movement",
	"data": {
		"x": <number (optional)>,
		"y": <number (optional)>,
		"z": <number (optional)>,
		"autoupdate": <boolean (optional, default true)>
	}
}
```

### physics

WIP

```rust
{
	"type": "physics",
	"data": {
		"activated": <boolean (default: false)>
	}
}
```

### requestedMovement

Determines the requested movement of the entity.
If there are multiple `requestedMovement` components, the requested movement of the entity is the sum of all the components.

As long as the entity's movement is not equal to its requestedMovement, an acceleration will be applied in order to compensate.

```rust
{
	"type": "requestedMovement",
	"data": {
		"x": <number (optional)>,
		"y": <number (optional)>,
		"z": <number (optional)>
	}
}
```

Units movement should be done by manipulating `requestedMovement`.


## Mechanics

### childEntity

**(not implemented)**

Link to another entity.
If the component is destroyed, the entity gets destroyed with it.
If the entity is destroyed, the component gets destroyed with it.

```rust
{
	"type": "childEntity",
	"data": {
		"entity": <entity>
	}
}
```

### destination

**(not implemented)**

Entity will try to move (via a `requestedMovement` component) to the destination point.

```rust
{
	"type": "destination",
	"data": {
		"x": <number>,
		"y": <number>,
		"z": <number>
	}
}
```

### executeNow

**(not implemented)**

When this component is created, its script will be executed as soon as possible and the component is destroyed.

```rust
{
	"type": "executeNow",
	"data": {
		"script": <string>
	}
}
```

### externContent

**(not implemented)**

As long as this component is alive, the entities defined in the given resource are loaded into the game.

If the content of `resource` is changed, the entities are unloaded.

```rust
{
	"type": "externContent",
	"data": {
		"resource": <string (optional)>,
		"whileLoading": <entity (optional)>,
		"destroyAfterLoading": <boolean (optional, default false)>
	}
}
```

 - `resource`: resources to load
 - `whileLoading`: entity that should become visible while the resource is loading
 - `destroyAfterLoading`: if true, the loaded entities will be merged into the state and this component will be destroyed

This is useful to load levels or menus.

### property

Defines a custom property.

```rust
{
	"type": "property",
	"data": {
		"property": <string>,
		"value": <anything>,
		"priority": <number (optional, default 1)>
	}
}
```

 - `name`: name of the property
 - `value`: value of the property
 - `priority`: if multiple properties of the same name exist on the same entity, the one with the highest priority will take precedence

### propertyView

Defines a custom property whose value is an alias of something else.

```rust
{
	"type": "propertyView",
	"data": {
		"property": <string>,
		"value": <string>,
		"priority": <number (optional, default 1)>
	}
}
```

 - `name`: name of the property
 - `value`: Lua script that gets executed each time the value of this property needs to be obtained, **the script must return a value by calling `return` manually**
 - `priority`: if multiple properties of the same name exist on the same entity, the one with the highest priority will take precedence

For example, you can define a property view named `dead` whose value is `return This.properties["health"] <= 0`

### propertyRange

**(not implemented)**

Script to execute when the value of a property is in a certain range.

```rust
{
	"type": "propertyRange",
	"data": {
		"property": <string>,
		"minValue": <number (optional)>,
		"maxValue": <number (optional)>,
		"scriptOnEnter": <string (optional)>,
		"scriptOnLeave": <string (optional)>,
		"prototypeInRange": <entity (optional)>
	}
}
```

### spawner

Spawns entities.

```rust
{
	"type": "spawner",
	"data": {
		"nextSpawn": <number>,
		"interval": <number>,
		"limit": <number (optional)>,
		"prototype": <entity (optional)>,
		"detach": <boolean (optional, default false)>,
		"destroyAfterExpiration": <boolean (option, default true)>
	}
}
```

**note: detach false is not supported, currently it is only true**

- `nextSpawn`: time in seconds until the next entity is spawned
- `interval`: time between two spawns
- `limit`: number of entities to spawn (default: infinite)
- `prototype`: prototype that the new entities will inherit from
- `detach`: if true, the new entities will be detached from the spawner ; if false, the new entities will be destroyed if the spawner is destroyed
- `destroyAfterExpiration`: if true, the spawner component will be destroyed after the limit reaches 0

### timeModifier

**(not implemented)**

Applies a modifier on all chronometer in the world

```
{
	"type": "timeModifier",
	"data": {
		"modifier": <number>
	}
}
```

 - `time`: multiplies the rate at which the time elapses by this number, cannot be native

Add this component for bullet-time effets.

To pause the game, add this component and set `modifier` to 0.
Pausing the game only affects the physics engine and the animations in the display engine. All scripts, input handlers, etc. will still be executed.

### timedDestruction

**(not implemented)**

This component will self-destruct after a certain time has passed

```
{
	"type": "timedDestruction",
	"data": {
		"time": <number>,
		"prototype": <entity>
	}
}
```

 - `time`: number of seconds before destruction
 - `prototype`: prototype to inherit from, as long as this component exists

This component is useful to add temporary effects on entities.
