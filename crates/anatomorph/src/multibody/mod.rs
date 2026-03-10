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

use crate::{Builtin, Dependant};

pub struct MultiBodyPlugin;

impl Plugin for MultiBodyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MultiBody>();
        app.init_resource::<GlobalTransforms>();
        app.add_systems(Update, (update_global_transforms, visualize));
    }
}

#[derive(Debug, Default, Resource)]
pub struct MultiBody {
    pub bodies: Vec<Body>,
}
#[derive(Debug, Default, Resource)]
pub struct GlobalTransforms {
    pub global_transforms: Vec<SE3>,
}
#[derive(Debug, Clone, Copy)]
pub enum Joint {
    Fixed(SE3),
    Free(SE3),
    Spherical(SO3),
}
impl Joint {
    pub fn to_transform(self) -> SE3 {
        match self {
            Joint::Fixed(transform) => transform,
            Joint::Free(transform) => transform,
            Joint::Spherical(rotation) => SE3 {
                rotation,
                ..Default::default()
            },
        }
    }
}
impl Default for Joint {
    fn default() -> Self {
        Joint::Fixed(default())
    }
}
#[derive(Debug, Default, Clone)]
pub struct Body {
    pub joint: Joint,
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
pub fn update_global_transforms(
    multibody: Res<MultiBody>,
    mut global_transforms: ResMut<GlobalTransforms>,
) {
    if multibody.is_changed() {
        let global_transforms = &mut global_transforms.global_transforms;
        global_transforms.resize(multibody.bodies.len(), default());
        for i in 0..multibody.bodies.len() {
            let body = &multibody.bodies[i];
            global_transforms[i] = if let Some(parent) = body.parent {
                global_transforms[parent] * body.joint.to_transform()
            } else {
                body.joint.to_transform()
            };
        }
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
