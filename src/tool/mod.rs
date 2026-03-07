use std::{any::TypeId, collections::HashMap};

use bevy::prelude::*;

pub mod select;

#[derive(Debug)]
pub struct Tool {
    enbaled: bool,
}

#[derive(Debug, Resource, Default)]
pub struct Tools {
    tools: HashMap<TypeId, Tool>,
}
