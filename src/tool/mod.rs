use std::{any::TypeId, collections::HashMap};

use bevy::prelude::*;

use crate::tool::control::ControlPlugin;

pub mod select;
pub mod control;

#[derive(Debug)]
pub struct Tool {
    enbaled: bool,
}

pub trait ToolPlugin where Self: 'static {
    const ENABLED: bool = true;
    fn register(app:&mut App){
        app.world_mut()
            .resource_mut::<Tools>()
            .tools
            .insert(TypeId::of::<Self>(), Tool { enbaled: Self::ENABLED });
    }
    fn enbale_condition(tools: Res<Tools>)->bool{
        tools.tools[&TypeId::of::<ControlPlugin>()].enbaled
    }
}

#[derive(Debug, Resource, Default)]
pub struct Tools {
    tools: HashMap<TypeId, Tool>,
}
