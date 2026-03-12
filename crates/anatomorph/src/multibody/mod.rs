use std::ops::Mul;

use anatomorph_math::{Aff3, R3, SE3, SO3, bevy::ToBevy};
use bevy::{
    ecs::{query::Changed, system::Query}, log::tracing_subscriber::layer::Context, math::{Quat, Vec2, Vec3}, prelude::*, transform::components::Transform, utils::default
};
use nalgebra::{Quaternion, UnitQuaternion, Vector2, Vector3};

use crate::{Builtins, Dependant, gen_set, multibody::joint::{Idx, Joint, JointClass}, impl_set::{InSet}};

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

pub trait JointsTraverser {
    fn run<T:JointClass>(&mut self,field:&Vec<Joint<T>>);
}
pub trait JointsTraverserMut {
    fn run<T:JointClass>(&mut self,field:&mut Vec<Joint<T>>);
}

gen_set!(#[derive(Debug,Default)] pub Joints:JointsTraverser,JointsTraverserMut{free:Vec<Joint<joint::Free>>,swing_twist:Vec<Joint<joint::SwingTwist>>});

#[derive(Debug, Default, Resource)]
pub struct MultiBody {
    pub bodies: Vec<Body>,
    pub joints: Joints,
}

impl MultiBody{
    pub fn add_joint<T:JointClass>(&mut self,joint:Joint<T>)->usize where Vec<Joint<T>>:InSet<Joints>{
        let vec = self.joints.get_mut::<Vec<Joint<T>>>();
        vec.push(joint);
        vec.len()-1
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
    struct Context<'a>{
        transforms:&'a mut Vec<SE3>
    }
    impl<'a> JointsTraverser for Context<'a>{
        fn run<T:JointClass>(&mut self,field:&Vec<Joint<T>>) {
            for joint in field{
                self.transforms[joint.body] = joint.class.transform();
            }
        }
    }
    let transforms = &mut transforms.transforms;
    let len = multibody.bodies.iter().len();
    transforms.resize(len, default());
    multibody.joints.traverse(&mut Context{transforms});
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
    builtin: Res<Builtins>,
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
