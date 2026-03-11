use crate::tool::ToolPlugin;
use bevy::prelude::*;

pub mod swing_twist;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(swing_twist::Plugin);
    }
}
