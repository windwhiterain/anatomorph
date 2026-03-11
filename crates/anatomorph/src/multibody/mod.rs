use std::ops::Mul;

use anatomorph_math::{Aff3, R3, SE3, SO3, bevy::ToBevy};
use bevy::{
    ecs::{query::Changed, system::Query},
    math::{Quat, Vec2, Vec3},
    prelude::*,
    transform::components::Transform,
    utils::default,
};
use nalgebra::{Quaternion, UnitQuaternion, Vector2, Vector3};

use crate::{Builtin, Dependant, multibody::joint::{Idx, Joint, JointClass}};

pub mod joint;

pub struct MultiBodyPlugin;

impl Plugin for MultiBodyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MultiBody>();
        app.init_resource::<Transforms>();
        app.init_resource::<GlobalTransforms>();
        app.add_systems(Update, (update_transforms,update_global_transforms, visualize));
    }
}

#[derive(Debug, Default, Resource)]
pub struct MultiBody {
    pub bodies: Vec<Body>,
    pub free_joints: Vec<Joint<joint::Free>>,
    pub swing_twist_joints: Vec<Joint<joint::SwingTwist>>,
}

impl MultiBody{
    pub fn add_free_joint(&mut self,joint:Joint<joint::Free>)->usize{
        self.free_joints.push(joint);
        self.free_joints.len()-1
    }
    pub fn add_swing_twist_joint(&mut self,joint:Joint<joint::SwingTwist>)->usize{
        self.swing_twist_joints.push(joint);
        self.swing_twist_joints.len()-1
    }
}

#[derive(Debug, Default, Resource)]
pub struct Transforms {
    pub transforms: Vec<SE3>,
}

#[derive(Debug, Default, Resource)]
pub struct GlobalTransforms {
    pub global_transforms: Vec<SE3>,
}

#[derive(Debug, Default, Clone)]
pub struct Body {
    pub joint: Option<joint::Idx>,
    pub parent: Option<usize>,
    pub mesh: Option<BodyMesh>,
}

#[derive(Debug, Clone)]
pub struct BodyMesh {
    pub handle: Handle<Mesh>,
    pub translation: R3,
    pub scale: R3,
    pub rotation: SO3,
}

impl Default for BodyMesh {
    fn default() -> Self {
        Self {
            handle: Default::default(),
            translation: Default::default(),
            scale: R3::repeat(1.0),
            rotation: Default::default(),
        }
    }
}

#[derive(Component)]
pub struct BodyVisualizer {
    pub idx: usize,
}

impl MultiBody {
    pub fn new(bodies: Vec<Body>) -> Self {
        Self {
            bodies,
            ..default()
        }
    }
}
pub fn update_transforms(
    multibody: Res<MultiBody>,
    mut transforms: ResMut<Transforms>,
){
    if !multibody.is_changed() {return}
    let transforms = &mut transforms.transforms;
    let len = multibody.bodies.iter().len();
    transforms.resize(len, default());
    for joint in &multibody.free_joints{
        transforms[joint.body] = joint.class.transform();
    }
    for joint in &multibody.swing_twist_joints{
        transforms[joint.body] = joint.class.transform();
    }
}
pub fn update_global_transforms(
    multibody: Res<MultiBody>,
    transforms: Res<Transforms>,
    mut global_transforms: ResMut<GlobalTransforms>,
) {
    if !transforms.is_changed() {return}
    let global_transforms = &mut global_transforms.global_transforms;
    let transforms = &transforms.transforms;
    let len = transforms.len();
    global_transforms.resize(len, default());
    for i in 0..len {
        let body = &multibody.bodies[i];
        global_transforms[i] = if let Some(parent) = body.parent {
            global_transforms[parent] * transforms[i]
        } else {
            transforms[i]
        };
    }
}
fn visualize(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut BodyVisualizer, &mut Mesh3d)>,
    multibody: Res<MultiBody>,
    global_transforms: Res<GlobalTransforms>,
    builtin: Res<Builtin>,
) {
    if !multibody.is_changed() && !global_transforms.is_changed() {
        return;
    }
    let mut bodies = (0..multibody
        .bodies
        .len()
        .min(global_transforms.global_transforms.len()))
        .filter_map(|idx| {
            let body = &multibody.bodies[idx];
            let mesh = &body.mesh;
            if let Some(mesh) = mesh {
                let target_transform = global_transforms.global_transforms[idx]
                    * SE3 {
                        translation: mesh.scale.component_mul(&mesh.translation),
                        rotation: mesh.rotation,
                    };
                let target_transform = Transform {
                    translation: target_transform.translation.to_bevy(),
                    rotation: target_transform.rotation.to_bevy(),
                    scale: mesh.scale.to_bevy(),
                };
                Some((idx, target_transform, &mesh.handle))
            } else {
                None
            }
        });
    for (entity, mut transform, mut visualizer, mut mesh3d) in query.iter_mut() {
        if let Some((idx, target_transform, mesh)) = bodies.next() {
            *transform = target_transform;
            visualizer.idx = idx;
            if mesh3d.0 != *mesh {
                mesh3d.0 = mesh.clone();
            }
        } else {
            commands.entity(entity).despawn();
        }
    }
    for (idx, target_transform, mesh) in bodies {
        commands.spawn((
            target_transform,
            BodyVisualizer { idx },
            Mesh3d(mesh.clone()),
            MeshMaterial3d(builtin.default_material.clone()),
        ));
    }
}
