use bevy::prelude::*;

use crate::tool::ToolPlugin;

pub struct SelectPlugin;

#[derive(Component, Debug)]
#[require(Pickable)]
pub struct Selectable;

#[derive(Resource, Default)]
pub struct Selected {
    pub entity: Option<Entity>,
}

fn on_click(
    mut events: MessageReader<Pointer<Click>>,
    q: Query<&Selectable>,
    mut selected: ResMut<Selected>,
) {
    if let Some(event) = events.read().last() {
        selected.entity = Some(event.entity)
    }
}

impl ToolPlugin for SelectPlugin {}

impl Plugin for SelectPlugin {
    fn build(&self, app: &mut App) {
        Self::register(app);
        app.init_resource::<Selected>();
        app.add_plugins(MeshPickingPlugin);
        app.add_systems(Update, on_click.run_if(Self::enbale_condition));
    }
}
