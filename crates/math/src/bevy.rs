use bevy_math::{Quat, Vec2, Vec3};
use nalgebra::Quaternion;

use crate::{R2, R3, SO3};

pub trait ToBevy {
    type Bevy;
    fn to_bevy(self) -> Self::Bevy;
}
pub trait ToAnatomorph {
    type Anatomorph;
    fn to_anatomorph(self) -> Self::Anatomorph;
}

impl ToBevy for R3 {
    type Bevy = Vec3;
    fn to_bevy(self) -> Self::Bevy {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl ToAnatomorph for Vec3 {
    type Anatomorph = R3;
    fn to_anatomorph(self) -> Self::Anatomorph {
        R3::new(self.x, self.y, self.z)
    }
}

impl ToBevy for R2 {
    type Bevy = Vec2;
    fn to_bevy(self) -> Self::Bevy {
        Vec2::new(self.x, self.y)
    }
}

impl ToAnatomorph for Vec2 {
    type Anatomorph = R2;
    fn to_anatomorph(self) -> Self::Anatomorph {
        R2::new(self.x, self.y)
    }
}

impl ToBevy for SO3 {
    type Bevy = Quat;

    fn to_bevy(self) -> Self::Bevy {
        Quat::from_xyzw(self.i, self.j, self.k, self.w)
    }
}

impl ToAnatomorph for Quat {
    type Anatomorph = SO3;
    fn to_anatomorph(self) -> Self::Anatomorph {
        SO3::from_quaternion(Quaternion::new(self.w, self.x, self.y, self.z))
    }
}
