use core::f32;
use std::ops::Mul;

use nalgebra::{Matrix3, Unit, UnitQuaternion, Vector2, Vector3};

pub type R1 = f32;
pub const PI:R1 = f32::consts::PI;
pub type R3 = Vector3<f32>;
pub type R2 = Vector2<f32>;
pub type SO3 = UnitQuaternion<f32>;
pub type GL3 = Matrix3<f32>;

#[derive(Debug, Clone, Copy, Default)]
pub struct SE3 {
    pub translation: R3,
    pub rotation: SO3,
}
impl Mul for SE3 {
    type Output = SE3;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            translation: self.translation + self.rotation * rhs.translation,
            rotation: self.rotation * rhs.rotation,
        }
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub struct Aff3 {
    pub translation: R3,
    pub linear: GL3,
}

impl Mul for Aff3 {
    type Output = Aff3;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            translation: self.translation + self.linear * rhs.translation,
            linear: self.linear * rhs.linear,
        }
    }
}

#[cfg(feature = "bevy")]
pub mod bevy;
