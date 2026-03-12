use anatomorph_math::{Aff3, GL3, R3, SE3};
use bevy::utils::default;

use crate::{
    multibody::{
        self, Body, BodyMesh,
        joint::{self, Joint},
    },
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

    fn add(&self, bone: &Bone, multibody: &mut multibody::MultiBody) {
        let swing_twist_joint = multibody.add_joint(Joint {
            class: joint::SwingTwist::default(),
            body: bone.body_offset + 0,
        });
        multibody.bodies[bone.body_offset + 0] = Body {
            joint: Some(joint::Idx::SwingTwist(swing_twist_joint)),
            mesh: Some(BodyMesh {
                scale: R3::new(1.0, 1.0, self.length).component_mul(&self.mesh.scale),
                ..self.mesh.clone()
            }),
            ..Default::default()
        };
        let free_joint = multibody.add_joint(Joint {
            class: joint::Free::default(),
            body: bone.body_offset + END,
        });
        multibody.bodies[bone.body_offset + END] = Body {
            joint: Some(joint::Idx::Free(free_joint)),
            parent: Some(bone.body_offset + 0),
            ..Default::default()
        };
    }

    fn update(&self, bone: &Bone, multibody: &mut multibody::MultiBody) {
        let joint::Idx::Free(idx) = multibody.bodies[bone.body_offset + END].joint.unwrap() else {
            unreachable!()
        };
        multibody.joints.free[idx].class.0.translation = R3::z() * self.length;
    }
}
