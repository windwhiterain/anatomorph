use std::ops::Mul;

use bevy::{
    math::{Quat, Vec3,Vec2},
    transform::components::Transform,
    utils::default,
};
use nalgebra::{Quaternion, UnitQuaternion, Vector2, Vector3};

use crate::{IntoAnatomorph, IntoBevy};
pub type R3 = Vector3<f32>;
pub type R2 = Vector2<f32>;
pub type SO3 = UnitQuaternion<f32>;

impl IntoBevy for R3 {
    type Bevy = Vec3;
    fn into_bevy(self) -> Self::Bevy {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl IntoAnatomorph for Vec3 {
    type Anatomorph = R3;
    fn into_anatomorph(self) -> Self::Anatomorph {
        R3::new(self.x,self.y,self.z)
    }
}

impl IntoBevy for R2 {
    type Bevy = Vec2;
    fn into_bevy(self) -> Self::Bevy {
        Vec2::new(self.x, self.y)
    }
}

impl IntoAnatomorph for Vec2 {
    type Anatomorph = R2;
    fn into_anatomorph(self) -> Self::Anatomorph {
        R2::new(self.x,self.y)
    }
}

impl IntoBevy for SO3 {
    type Bevy = Quat;

    fn into_bevy(self) -> Self::Bevy {
        Quat::from_xyzw(self.i, self.j, self.k, self.w)
    }
}

impl IntoAnatomorph for Quat {
    type Anatomorph = SO3;
    fn into_anatomorph(self) -> Self::Anatomorph {
        SO3::from_quaternion(Quaternion::new(self.w, self.x, self.y, self.z))
    }
}

impl IntoBevy for SE3 {
    type Bevy = Transform;

    fn into_bevy(self) -> Self::Bevy {
        Transform {
            translation: self.translation.into_bevy(),
            rotation: self.rotation.into_bevy(),
            ..default()
        }
    }
}

impl IntoAnatomorph for Transform {
    type Anatomorph = SE3;
    fn into_anatomorph(self) -> Self::Anatomorph {
        SE3{translation:self.translation.into_anatomorph(),rotation:self.rotation.into_anatomorph()}
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SE3 {
    pub translation: R3,
    pub rotation: SO3,
}
impl Mul for SE3 {
    type Output = SE3;

    fn mul(self, rhs: Self) -> Self::Output {
        SE3 {
            translation: self.translation + self.rotation * rhs.translation,
            rotation: self.rotation * rhs.rotation,
        }
    }
}

pub const NONE_PARENT: usize = usize::MAX;
#[derive(Debug)]
pub struct Dependant<T> {
    pub value: Option<T>,
    pub dirty: bool,
}
impl<T: Default> Default for Dependant<T> {
    fn default() -> Self {
        Self {
            value: Default::default(),
            dirty: true,
        }
    }
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
    pub fn transform_of(&self,idx:usize)->SE3{
        self.bodies[idx].transform
    }
    pub fn set_transform_of(&mut self,idx:usize,transform:SE3){
        self.bodies[idx].transform = transform;
        self.global_transforms.dirty = true;
    }
    pub fn parent_of(&self,idx:usize)->Option<usize>{
        let parent = self.bodies[idx].parent;
        if parent == NONE_PARENT{
            return None
        }else{
            Some(parent)
        }
    }
    pub fn update_globbal_transforms(&mut self){
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
                    global_transforms[i] = global_transforms[body.parent] * body.transform
                }
            }
            self.global_transforms.dirty = false;
        }
    }
    pub fn global_transform_of(&self,idx:usize) -> SE3 {
        self.global_transforms.value.as_ref().unwrap()[idx]
    }
}
