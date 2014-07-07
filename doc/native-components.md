## Native components


### Common

#### position

Determines the position of the entity.

If there are multiple `position` components, the position of the entity is the sum of all the components.

The values in this component are updated when the game runs.

```rust
{
	"type": "position",
	"data": {
		"x": <number (optional)>,
		"y": <number (optional)>,
		"z": <number (optional)>
	}
}
```


### Display

#### camera

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


#### spriteDisplay

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


### Input

#### inputHandler

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


### Physics

#### movement

Determines the requested movement of the entity.

If there are multiple `movement` components, the movement of the entity is the sum of all the components.

```rust
{
	"type": "movement",
	"data": {
		"x": <number (optional)>,
		"y": <number (optional)>,
		"z": <number (optional)>
	}
}
```


#### physics

WIP

```rust
{
	"type": "physics",
	"data": {
		"activated": <boolean (default: false)>
	}
}
```


### Mechanics

#### executeNow

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

#### externContent

**(not implemented)**

As long as this component is alive, the entities defined in the given resource are loaded into the game.

If the content of `resource` is changed, the entities are unloaded.

```rust
{
	"type": "externContent",
	"data": {
		"resource": <string (optional)>
	}
}
```

#### property

Defines a custom property

```rust
{
	"type": "property",
	"data": {
		"property": <string>,
		"value": <anything>
	}
}
```

#### propertyRange

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
