use anatomorph_math::{Aff3, GL3, R3, SE3};
use bevy::prelude::*;
use bevy::utils::default;

use crate::{
    multibody::{
        self,
        joint::{self},
    },
    skeleton::{self, Class, Skeleton, SkeletonPlugin},
};

pub const CHILD_END: usize = 0;
#[derive(Debug, Component)]
pub struct Pole {
    pub length: f32,
    pub mesh: multibody::Mesh,
    child_bodies: [Option<Entity>; 1],
    free_joint: Option<Entity>,
}
impl Pole {
    pub fn new(length: f32, mesh: multibody::Mesh) -> Self {
        Self {
            length,
            mesh,
            child_bodies: default(),
            free_joint: default(),
        }
    }
}
impl Class for Pole {
    fn child_bodies(&self) -> &[Option<Entity>] {
        &self.child_bodies
    }
}

pub struct Plugin;
impl SkeletonPlugin for Plugin {
    type Skeleton = Pole;
}

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                on_add
                    .before(skeleton::on_change_parent::<Pole>)
                    .after(skeleton::on_add_skeleton),
                on_change.after(on_add),
            ),
        );
    }
}

fn on_add(mut commands: Commands, poles: Query<(&mut Pole, &Skeleton), Added<Skeleton>>) {
    for (mut pole, skeleton) in poles {
        commands
            .entity(skeleton.root_body.unwrap())
            .insert(pole.mesh.clone());
        pole.free_joint = Some(multibody::add(
            &mut commands,
            joint::Free::default(),
            Some(skeleton.root_body.unwrap()),
            None,
            false,
        ));
        pole.child_bodies[CHILD_END] = Some(multibody::add(
            &mut commands,
            joint::SwingTwist {
                swing: R3::z_axis(),
                twist: 0.0,
            },
            Some(pole.free_joint.unwrap()),
            None,
            true
        ));
    }
}

fn on_change(
    poles: Query<(&Pole, &Skeleton), Changed<Pole>>,
    mut free_joints: Query<&mut joint::Free>,
    mut meshes: Query<&mut multibody::Mesh>,
) {
    for (pole, skeleton) in poles {
        let mut free_joint = free_joints.get_mut(pole.free_joint.unwrap()).unwrap();
        free_joint.0.translation = R3::z() * pole.length;
        let mut mesh = meshes.get_mut(skeleton.root_body.unwrap()).unwrap();
        mesh.scale = R3::new(1.0, 1.0, pole.length).component_mul(&pole.mesh.scale);
    }
}
