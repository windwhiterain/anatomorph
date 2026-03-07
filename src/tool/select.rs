use std::any::TypeId;

use bevy::prelude::*;

use crate::tool::{Tool, Tools};

pub struct SelectPlugin;

#[derive(Component, Debug)]
#[require(Pickable)]
pub struct Selectable;

#[derive(Component, Debug)]
#[require(MeshPickingCamera)]
pub struct SelectCamera;

fn on_click(mut events: MessageReader<Pointer<Click>>, q: Query<&Selectable>) {
    for ev in events.read() {
        if let Ok(entity) = q.get(ev.entity) {
            info!("Picked entity with MyComponent: {:?}", ev.entity);
        }
    }
}

impl Plugin for SelectPlugin {
    fn build(&self, app: &mut App) {
        app.world_mut()
            .resource_mut::<Tools>()
            .tools
            .insert(TypeId::of::<SelectPlugin>(), Tool { enbaled: true });
        app.add_plugins(MeshPickingPlugin);
        app.add_systems(
            Update,
            on_click.run_if(|tools: Res<Tools>| tools.tools[&TypeId::of::<SelectPlugin>()].enbaled),
        );
    }
}
