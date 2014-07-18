use std::collections::HashMap;
use std::fmt::Show;

pub trait ConfigValue: Send + Clone + Show {
}

pub struct Config {
    params: HashMap<String, Box<ConfigValue>>
}

impl ConfigValue for int {}
impl ConfigValue for String {}
impl ConfigValue for bool {}
