use anatomorph_math::{Aff3, GL3, R3, SE3};
use bevy::utils::default;

use crate::{
    multibody::{self, Body, BodyMesh, Joint},
    skeleton::{Bone, BoneClass},
};

pub const END: usize = 1;
#[derive(Debug)]
pub struct Pole {
    pub length: f32,
    pub mesh: BodyMesh,
}
impl BoneClass for Pole {
    fn bodies_len(&self) -> usize {
        2
    }

    fn update(&self, bone: &Bone, bodies: &mut [crate::multibody::Body]) {
        bodies[0] = Body {
            joint: Joint::Spherical(default()),
            mesh: Some(BodyMesh {
                scale: R3::new(1.0, 1.0, self.length),
                ..self.mesh.clone()
            }),
            ..Default::default()
        };
        bodies[END] = Body {
            joint: Joint::Fixed(SE3 {
                translation: R3::new(0.0, 0.0, self.length),
                ..Default::default()
            }),
            parent: Some(bone.body_offset),
            ..Default::default()
        };
    }
}
