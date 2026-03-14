use std::collections::HashSet;

use bevy::{ecs::relationship::OrderedRelationshipSourceCollection, prelude::*};

#[derive(Debug, Component)]
pub struct Parent(pub Entity);

#[derive(Debug, Component, Default)]
pub struct Children(pub HashSet<Entity>);

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (on_add_parent, on_remove_parent, on_remove_children),
        );
    }
}

fn on_add_parent(
    parents: Query<(Entity, &Parent), Added<Parent>>,
    mut childrens: Query<&mut Children>,
) {
    for (entity, parent) in parents {
        let mut children = childrens.get_mut(parent.0).unwrap();
        children.0.insert(entity);
    }
}

fn on_remove_parent(
    mut removed: RemovedComponents<Parent>,
    parents: Query<&Parent>,
    mut childrens: Query<&mut Children>,
) {
    for this in removed.read() {
        let parent = parents.get(this).unwrap();
        if let Ok(mut children) = childrens.get_mut(parent.0) {
            children.0.remove(&this);
        }
    }
}

fn on_remove_children(
    mut removed: RemovedComponents<Children>,
    mut commands: Commands,
    childrens: Query<&Children>,
) {
    for this in removed.read() {
        let children = childrens.get(this).unwrap();
        for child in children.0.iter().copied() {
            commands.entity(child).despawn();
        }
    }
}
