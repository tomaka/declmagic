extern crate std;

pub trait ConfigValue: Send + Clone + std::fmt::Show {
}

pub struct Config {
	params: std::collections::HashMap<String, Box<ConfigValue>>
}

impl ConfigValue for i32 {}
impl ConfigValue for String {}
impl ConfigValue for bool {}
