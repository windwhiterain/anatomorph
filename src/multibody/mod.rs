use std::ops::Mul;

use bevy::{
    math::{Quat, Vec3},
    transform::components::Transform,
    utils::default,
};
use nalgebra::{UnitQuaternion, Vector3};

use crate::IntoBevy;
pub type R3 = Vector3<f32>;
pub type SO3 = UnitQuaternion<f32>;

impl IntoBevy for R3 {
    type Bevy = Vec3;

    fn into_bevy(self) -> Self::Bevy {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl IntoBevy for SO3 {
    type Bevy = Quat;

    fn into_bevy(self) -> Self::Bevy {
        Quat::from_xyzw(self.i, self.j, self.k, self.w)
    }
}

impl IntoBevy for SE3 {
    type Bevy = Transform;

    fn into_bevy(self) -> Self::Bevy {
        Transform {
            translation: self.translation.into_bevy(),
            rotation: self.orientation.into_bevy(),
            ..default()
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SE3 {
    pub translation: R3,
    pub orientation: SO3,
}
impl Mul for SE3 {
    type Output = SE3;

    fn mul(self, rhs: Self) -> Self::Output {
        SE3 {
            translation: self.translation + self.orientation * rhs.translation,
            orientation: self.orientation * rhs.orientation,
        }
    }
}

pub const NONE_PARENT: usize = usize::MAX;
#[derive(Debug, Default)]
pub struct Dependant<T> {
    pub value: Option<T>,
    pub dirty: bool,
}
#[derive(Debug, Default)]
pub struct MultiBody {
    pub bodies: Vec<Body>,
    global_transforms: Dependant<Vec<SE3>>,
}
#[derive(Debug)]
pub struct Body {
    pub transform: SE3,
    pub parent: usize,
}

impl MultiBody {
    pub fn new(bodies: Vec<Body>) -> Self {
        Self {
            bodies,
            ..default()
        }
    }
    pub fn globbal_transforms(&mut self) -> &Vec<SE3> {
        if self.global_transforms.dirty {
            if self.global_transforms.value.is_none() {
                self.global_transforms.value = Some(Vec::new());
            }
            let global_transforms = self.global_transforms.value.as_mut().unwrap();
            global_transforms.resize(self.bodies.len(), default());
            for i in 0..self.bodies.len() {
                let body = &self.bodies[i];
                if body.parent == NONE_PARENT {
                    global_transforms[i] = body.transform;
                } else {
                    global_transforms[i] = self.bodies[body.parent].transform * body.transform
                }
            }
            self.global_transforms.dirty = false;
        }
        self.global_transforms.value.as_ref().unwrap()
    }
}
