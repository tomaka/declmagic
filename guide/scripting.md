# Script

**Note: most of this is not implemented**

Many components allow you to specify a **script**. These scripts are in Lua language.

## API

The API manipulate values of custom types:
 - `Entity`
 - `Component`
 - `EntitiesList`

The `Entity` API:
 - `name`: name of the entity (read-only)
 - `visible`: true if the entity is visible (read-only)
 - `components`: 
 - `properties`: read-only array which contains the values of all properties ([see the `property` and `propertyView` native components](native-components.md))

The `Component`:
 - `data`: array where each element represents some data of the component

Some global values are always defined:
 - `This` (type `Entity`) is the owner of the component who is executing the script
 - `Caller` (type `Component`) is the component who is executing the script
 - `Entities` (type `EntitiesList`) represents all the entities of the state

## Examples

```lua

```
