use anatomorph_math::{Aff3, GL3, R3, SE3};
use bevy::utils::default;

use crate::{
    multibody::{
        self, Body, BodyMesh,
        joint::{self, Joint},
    },
    skeleton::{Bone, BoneClass},
};

pub const BODY_END: usize = 1;
#[derive(Debug)]
pub struct Pole {
    pub length: f32,
    pub mesh: BodyMesh,
    free_joint_offset: usize,
}
impl Pole {
    pub fn new(length: f32, mesh: BodyMesh) -> Self {
        Self {
            length,
            mesh,
            free_joint_offset: default(),
        }
    }
}
impl BoneClass for Pole {
    fn bodies_len(&self) -> usize {
        2
    }

    fn init(&mut self, bone: &Bone, multibody: &mut multibody::MultiBody) {
        multibody.add_joint(joint::Desc {
            class: joint::SwingTwist::default(),
            body: bone.body_offset + 0,
        });
        multibody.bodies[bone.body_offset + 0] = Body {
            mesh: Some(BodyMesh {
                scale: R3::new(1.0, 1.0, self.length).component_mul(&self.mesh.scale),
                ..self.mesh.clone()
            }),
            ..Default::default()
        };
        self.free_joint_offset = multibody.joint_classes.free.len();
        multibody.add_joint(joint::Desc {
            class: joint::Free::default(),
            body: bone.body_offset + BODY_END,
        });
        multibody.bodies[bone.body_offset + BODY_END] = Body {
            parent: Some(bone.body_offset + 0),
            ..Default::default()
        };
    }

    fn update(&self, bone: &Bone, multibody: &mut multibody::MultiBody) {
        multibody.joint_classes.free[self.free_joint_offset].class.0.translation = R3::z() * self.length;
    }
}
