use core::f32;

use anatomorph_math::{PI, R1, R3, SE3, SO3};
use bevy::ecs::component::Component;
use nalgebra::Unit;
use bevy::prelude::*;

use crate::multibody;

pub struct Desc<T:Class>{
    pub body:usize,
    pub class:T,
}

pub trait Class: Clone+Copy+Component {
    fn transform(self) -> SE3;
}

pub trait JointPlugin: bevy::prelude::Plugin{
    type Joint: Class;
}

fn on_change_joints<T: Class>(
    joints: Query<(Entity, &T)>,
    mut transforms: Query<&mut multibody::Transform>,
) {
    for (entity, joint) in joints {
        transforms.get_mut(entity).unwrap().0 = joint.transform();
    }
}

pub fn register<T:Class>(app:&mut App){
    app.add_systems(Update, on_change_joints::<T>);
}

#[derive(Debug, Clone, Copy, Default,Component)]
pub struct Free(pub SE3);

impl Class for Free {
    fn transform(self) -> SE3 {
        self.0
    }
}

#[derive(Debug, Clone, Copy,Component)]
pub struct SwingTwist {
    pub swing: Unit<R3>,
    pub twist: f32,
}

impl Default for SwingTwist {
    fn default() -> Self {
        Self {
            swing: R3::z_axis(),
            twist: Default::default(),
        }
    }
}

impl Class for SwingTwist {
    fn transform(self) -> SE3 {
        SE3 {
            rotation: SO3::rotation_between_axis(&R3::z_axis(), &self.swing)
                .unwrap_or_else(|| SO3::from_axis_angle(&R3::x_axis(), PI))
                * SO3::from_axis_angle(&R3::z_axis(), self.twist),
            ..Default::default()
        }
    }
}
