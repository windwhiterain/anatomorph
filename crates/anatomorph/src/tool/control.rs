use anatomorph_math::{
    SE3, SO3,
    bevy::{ToAnatomorph as _, ToBevy as _},
};
use bevy::prelude::*;
use nalgebra::Unit;

use crate::{
    bevy_utils::World2Pixel,
    multibody::{self, Joint, MultiBody, MultiBodyPlugin},
    tool::ToolPlugin,
};

pub struct ControlPlugin;

#[derive(Debug, Resource, Default)]
pub struct Settings {
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
}

#[derive(Debug, Component)]
#[require(Pickable)]
pub struct ControllerVisualizer {
    pub idx: usize,
}

fn visualize(
    mut commands: Commands,
    multibody: Res<MultiBody>,
    global_transforms: Res<multibody::GlobalTransforms>,
    mut query: Query<(Entity, &mut Transform, &mut ControllerVisualizer)>,
    settings: Res<Settings>,
) {
    if multibody.is_changed() {
        let mut bodies = (0..multibody
            .bodies
            .len()
            .min(global_transforms.global_transforms.len()))
            .filter_map(|idx| {
                let body = &multibody.bodies[idx];
                if let Joint::Spherical(rotation) = body.joint {
                    let target_transform = global_transforms.global_transforms[idx];
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
            if let Some((idx, target_transform)) = bodies.next() {
                *transform = target_transform;
                visualizer.idx = idx;
            } else {
                commands.entity(entity).despawn();
            }
        }
        for (idx, target_transform) in bodies {
            commands.spawn((
                target_transform,
                ControllerVisualizer { idx },
                Mesh3d(settings.mesh.clone()),
                MeshMaterial3d(settings.material.clone()),
            ));
        }
    }
}

fn on_drag(
    mut events: MessageReader<Pointer<Drag>>,
    mut multibody: ResMut<MultiBody>,
    multibody_global_transforms: Res<multibody::GlobalTransforms>,
    controller_visualizer: Query<&ControllerVisualizer>,
    global_transform: Query<&GlobalTransform>,
    world_to_pixel: World2Pixel,
    camera_global_transform: Query<&GlobalTransform, With<Camera>>,
) {
    for event in events.read() {
        let _: Option<()> = try {
            let entity = event.entity;
            let idx = controller_visualizer.get(entity).ok()?.idx;
            let parent = multibody.bodies[idx].parent;
            let camera_translation_world = camera_global_transform
                .single()
                .unwrap()
                .translation()
                .to_anatomorph();
            let center_world = global_transform
                .get(entity)
                .unwrap()
                .translation()
                .to_anatomorph();
            let center_pixel = world_to_pixel.world2pixel(center_world)?;
            let arc = event.delta.to_anatomorph();
            let radius = event.pointer_location.position.to_anatomorph() - center_pixel;
            let angle = radius.perp(&arc) / radius.norm_squared();
            let axis = Unit::new_normalize(center_world - camera_translation_world);
            let global_rotation = SO3::from_axis_angle(&axis, angle);
            let delta_rotation = if let Some(parent) = parent {
                let parent_global_rotation =
                    multibody_global_transforms.global_transforms[parent].rotation;
                parent_global_rotation.conjugate() * global_rotation * parent_global_rotation
            } else {
                global_rotation
            };
            if let Joint::Spherical(rotation) = &mut multibody.bodies[idx].joint {
                *rotation = delta_rotation * *rotation;
            }
        };
    }
}

impl ToolPlugin for ControlPlugin {}

impl Plugin for ControlPlugin {
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
