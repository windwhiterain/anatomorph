use core::f32;

use anatomorph_math::{PI, R1, R3, SE3, SO3, UnitR3};

#[derive(Debug)]
pub struct Joint<T: JointClass> {
    pub class: T,
    pub body: usize,
}

pub trait JointClass: Clone+Copy {
    fn transform(self) -> SE3;
}

#[derive(Debug, Clone, Copy)]
pub enum Idx {
    Free(usize),
    SwingTwist(usize),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Free(pub SE3);

impl JointClass for Free {
    fn transform(self) -> SE3 {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SwingTwist {
    pub swing: UnitR3,
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

impl JointClass for SwingTwist {
    fn transform(self) -> SE3 {
        SE3 {
            rotation: SO3::rotation_between_axis(&R3::z_axis(), &self.swing)
                .unwrap_or_else(|| SO3::from_axis_angle(&R3::x_axis(), PI))
                * SO3::from_axis_angle(&R3::z_axis(), self.twist),
            ..Default::default()
        }
    }
}
