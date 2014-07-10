use entities::{ EntitiesHelper, EntitiesState, EntityID, ComponentID };
use rust_hl_lua::{ Lua, LuaError };
use rust_hl_lua::any;

pub fn execute_mut<E: EntitiesHelper, S: ::std::str::Str + ::std::fmt::Show>(entities: &mut E, component: &ComponentID, code: &S)
	-> Result<any::AnyLuaValue, LuaError>
{
	let mut lua = Lua::new();

	println!("executing script {}", code);

	//lua.set("Entities", );

	lua.execute(code.as_slice())
}

pub fn execute<E: EntitiesHelper, S: ::std::str::Str + ::std::fmt::Show>(entities: &E, component: &ComponentID, code: &S)
	-> Result<any::AnyLuaValue, LuaError>
{
	let mut lua = Lua::new();

	println!("executing script {}", code);

	//lua.set("Entities", );

	lua.execute(code.as_slice())
}
