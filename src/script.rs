use entities::{ EntitiesState, EntityID, ComponentID };
use rust_hl_lua::{ Lua, LuaError };

pub fn execute<S: ::std::str::Str + ::std::fmt::Show>(entities: &mut EntitiesState, component: &ComponentID, code: &S)
	-> Result<(), LuaError>
{
	let mut lua = Lua::new();

	println!("executing script {}", code);

	//lua.set("Entities", );

	lua.execute(code.as_slice())
}
