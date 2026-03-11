use anatomorph_math::{
    R2, R3, SE3, SO3,
    bevy::{ToAnatomorph as _, ToBevy as _},
};
use bevy::prelude::*;
use nalgebra::Unit;

use crate::{
    bevy_utils::World2Pixel,
    multibody::{self, MultiBody, MultiBodyPlugin, joint::JointClass},
    tool::ToolPlugin,
};

pub struct Plugin;

#[derive(Debug, Resource, Default)]
pub struct Settings {
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
}

#[derive(Debug, Component)]
#[require(Pickable)]
pub struct Visualizer {
    pub idx: usize,
}

fn visualize(
    mut commands: Commands,
    multibody: Res<MultiBody>,
    global_transforms: Res<multibody::GlobalTransforms>,
    mut query: Query<(Entity, &mut Transform, &mut Visualizer)>,
    settings: Res<Settings>,
) {
    if !multibody.is_changed() && !global_transforms.is_changed() {
        return;
    }
    let mut joints = (0..multibody.swing_twist_joints.len()).filter_map(|idx| {
        let body_idx = multibody.swing_twist_joints[idx].body;
        if let Some(target_transform) = global_transforms.global_transforms.get(body_idx) {
            Some((
                idx,
                Transform {
                    translation: target_transform.translation.to_bevy(),
                    rotation: target_transform.rotation.to_bevy(),
                    ..Default::default()
                },
            ))
        } else {
            None
        }
    });
    for (entity, mut transform, mut visualizer) in query.iter_mut() {
        if let Some((idx, target_transform)) = joints.next() {
            *transform = target_transform;
            visualizer.idx = idx;
        } else {
            commands.entity(entity).despawn();
        }
    }
    for (idx, target_transform) in joints {
        commands.spawn((
            target_transform,
            Visualizer { idx },
            Mesh3d(settings.mesh.clone()),
            MeshMaterial3d(settings.material.clone()),
        ));
    }
}

fn on_drag(
    mut events: MessageReader<Pointer<Drag>>,
    mut multibody: ResMut<MultiBody>,
    multibody_global_transforms: Res<multibody::GlobalTransforms>,
    controller_visualizer: Query<&Visualizer>,
    camera_global_transform: Query<&GlobalTransform, With<Camera>>,
) {
    for event in events.read() {
        let _: Option<()> = try {
            let entity = event.entity;
            let idx = controller_visualizer.get(entity).ok()?.idx;
            let body_idx = multibody.swing_twist_joints[idx].body;
            let parent = multibody.bodies[body_idx].parent;
            let camera_transform = camera_global_transform.single().unwrap();
            let camera_translation = camera_transform.translation().to_anatomorph();
            let camera_rotation = camera_transform.rotation().to_anatomorph();
            let translation = multibody_global_transforms
                .global_transforms
                .get(body_idx)?
                .translation;
            let direction = Unit::new_normalize(translation - camera_translation);
            let camera_rotation =
                SO3::rotation_between_axis(&(camera_rotation * (-R3::z_axis())), &direction)?
                    * camera_rotation;

            let arc = event
                .delta
                .to_anatomorph()
                .component_mul(&R2::new(1.0, -1.0))
                .push(0.0);

            let radius = R3::z();
            let axis = Unit::new_normalize(radius.cross(&arc));
            let angle = arc.norm() / 256.0;
            let target_rotation =
                camera_rotation * SO3::from_axis_angle(&axis, angle) * camera_rotation.conjugate();
            let delta_rotation = if let Some(parent) = parent {
                let parent_global_rotation = multibody_global_transforms
                    .global_transforms
                    .get(parent)?
                    .rotation;
                parent_global_rotation.conjugate() * target_rotation * parent_global_rotation
            } else {
                target_rotation
            };
            let joint = &mut multibody.swing_twist_joints[idx];
            joint.class.swing = delta_rotation * joint.class.swing;
        };
    }
}

impl ToolPlugin for Plugin {}

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        Self::register(app);
        if !app.is_plugin_added::<MultiBodyPlugin>() {
            app.add_plugins(MultiBodyPlugin);
        }
        app.init_resource::<Settings>();
        app.add_systems(
            Update,
            (
                visualize.run_if(Self::enbale_condition),
                on_drag.run_if(Self::enbale_condition),
            ),
        );
    }
}
