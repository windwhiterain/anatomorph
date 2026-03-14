use anatomorph_math::SE3;
use bevy::prelude::*;
use std::marker::PhantomData;

use crate::{bevy_utils::AddDendencyPlugin, multibody::{self, joint}};

pub mod pole;

pub trait Class: std::fmt::Debug + Component {
    fn child_bodies(&self) -> &[Option<Entity>];
}

#[derive(Debug, Component,Default)]
pub struct Skeleton {
    pub transform: SE3,
    pub root_body: Option<Entity>,
}

impl Skeleton {
    pub fn new(transform: SE3) -> Self {
        Self {
            transform,
            root_body: default(),
        }
    }
}

#[derive(Debug, Component)]
pub struct Parent<T: Class> {
    pub skeleton: Entity,
    pub offset: usize,
    _p: PhantomData<&'static T>,
}

impl<T: Class> Parent<T> {
    pub fn new(skeleton: Entity, offset: usize) -> Self {
        Self {
            skeleton,
            offset,
            _p: default(),
        }
    }
}

pub fn add<T: Class>(
    commands: &mut Commands,
    skeleton_class: T,
    skeleton: Skeleton,
    parent: Option<Parent<T>>,
)->Entity {
    let mut cmd = commands.spawn((skeleton_class, skeleton));
    if let Some(parent) = parent {
        cmd.insert(parent);
    }
    cmd.id()
}

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_dependency_plugin(||multibody::Plugin);
        app.add_systems(Update, (on_add_skeleton,on_change_skeleton.after(on_add_skeleton)));
        register(app, pole::Plugin);
    }
}

pub trait SkeletonPlugin:bevy::prelude::Plugin{
    type Skeleton:Class;
}

pub fn register<T:SkeletonPlugin>(app: &mut App,plugin:T){
    app.add_plugins(plugin);
    app.add_systems(Update, on_change_parent::<T::Skeleton>);
}

fn on_add_skeleton(
    mut commands: Commands,
    skeletons: Query<&mut Skeleton, Added<Skeleton>>,
) {
    for mut skeleton in skeletons {
        skeleton.root_body = Some(multibody::add(
            &mut commands,
            joint::Free::default(),
            None,
            None,
        ));
    }
}

fn on_change_parent<T: Class>(
    mut commands:Commands,
    parents: Query<(&Parent<T>, &Skeleton), Changed<Parent<T>>>,
    skeleton_classes: Query<&T>,
) {
    for (parent, skeleton) in parents {
        let skeleton_class = skeleton_classes.get(parent.skeleton).unwrap();
        let parent_body = skeleton_class.child_bodies()[parent.offset].unwrap();
        multibody::parent(&mut commands, skeleton.root_body.unwrap(), parent_body);
    }
}

fn on_change_skeleton(
    skeletons: Query<&Skeleton, Changed<Skeleton>>,
    mut free_joints: Query<&mut joint::Free>,
) {
    for skeleton in skeletons {
        let mut free_joint = free_joints.get_mut(skeleton.root_body.unwrap()).unwrap();
        free_joint.0 = skeleton.transform;
    }
}
