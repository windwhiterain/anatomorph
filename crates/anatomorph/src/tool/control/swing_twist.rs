use anatomorph_math::{
    R2, R3, SE3, SO3,
    bevy::{ToAnatomorph as _, ToBevy as _},
};
use bevy::{camera::visibility::RenderLayers, prelude::*};
use nalgebra::Unit;

use crate::{
    Builtins, MainCamera, UICamera,
    bevy_utils::World2Screen,
    multibody::{self, MultiBody, MultiBodyPlugin, joint::JointClass},
    tool::ToolPlugin,
};

pub struct Plugin;

#[derive(Debug, Component)]
#[require(Pickable)]
pub struct Visualizer {
    pub idx: usize,
}

fn visualize(
    mut commands: Commands,
    multibody: Res<MultiBody>,
    global_transforms: Res<multibody::GlobalTransforms>,
    mut controller: Query<(Entity, &mut Transform, &mut Visualizer)>,
    settings: Res<Builtins>,
    world2screen: World2Screen,
) {
    let mut joints = (0..multibody.joints.swing_twist.len()).filter_map(|idx| {
        let body_idx = multibody.joints.swing_twist[idx].body;
        if let Some(target_transform) = global_transforms.global_transforms.get(body_idx) {
            if let Some(screen_position) = world2screen.world2screen(target_transform.translation) {
                Some((
                    idx,
                    Transform {
                        translation: screen_position.push(1.0).to_bevy(),
                        scale: R3::repeat(16.0).to_bevy(),
                        ..Default::default()
                    },
                ))
            } else {
                None
            }
        } else {
            None
        }
    });
    for (entity, mut transform, mut visualizer) in controller.iter_mut() {
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
            Mesh2d(settings.rect.clone()),
            MeshMaterial2d(settings.yellow.clone()),
            RenderLayers::layer(1),
        ));
    }
}

fn on_drag(
    mut events: MessageReader<Pointer<Drag>>,
    mut multibody: ResMut<MultiBody>,
    multibody_global_transforms: Res<multibody::GlobalTransforms>,
    controller_visualizer: Query<&Visualizer>,
    camera_global_transform: Query<&GlobalTransform, With<MainCamera>>,
) {
    for event in events.read() {
        let _: Option<()> = try {
            let entity = event.entity;
            let idx = controller_visualizer.get(entity).ok()?.idx;
            let body_idx = multibody.joints.swing_twist[idx].body;
            let parent = multibody.bodies[body_idx].parent;
            let camera_transform = camera_global_transform.single().unwrap();
            let camera_translation = camera_transform.translation().to_anatomorph();
            let camera_rotation = camera_transform.rotation().to_anatomorph();
            let translation = multibody_global_transforms
                .global_transforms
                .get(body_idx)?
                .translation;
            let look_direction = Unit::new_normalize(translation - camera_translation);
            let camera_rotation =
                SO3::rotation_between_axis(&(camera_rotation * (-R3::z_axis())), &look_direction)?
                    * camera_rotation;
            let (camera2joint, joint_look_direction) = if let Some(parent) = parent {
                let world2joint = multibody_global_transforms
                    .global_transforms
                    .get(parent)?
                    .rotation
                    .inverse();

                (world2joint * camera_rotation, world2joint * look_direction)
            } else {
                (camera_rotation, look_direction)
            };
            let delta = event.delta.to_anatomorph();
            let arc = Unit::new_normalize(
                camera2joint * delta.component_mul(&R2::new(1.0, -1.0)).push(0.0),
            );
            let joint = &mut multibody.joints.swing_twist[idx];
            let radius = joint.class.swing;
            let axis = Unit::new_normalize(radius.cross(&arc));
            let angle = delta.norm() / 256.0;
            let swing_rotation = SO3::from_axis_angle(&axis, angle);
            let swing = &mut joint.class.swing;
            let forward = swing.dot(&joint_look_direction) > 0.0;
            *swing = swing_rotation * *swing;
            let look_component = swing.dot(&joint_look_direction);
            let safe_look_component = if forward {
                look_component.max(0.1)
            } else {
                look_component.min(-0.1)
            };
            if safe_look_component != look_component {
                let perp_component =
                    swing.into_inner() - look_component * joint_look_direction.into_inner();
                *swing = Unit::new_unchecked(
                    safe_look_component * joint_look_direction.into_inner()
                        + (1.0 - safe_look_component * safe_look_component).sqrt()
                            / perp_component.norm()
                            * perp_component,
                )
            }
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
        app.add_systems(
            Update,
            (
                visualize.run_if(Self::enbale_condition),
                on_drag.run_if(Self::enbale_condition),
            ),
        );
    }
}
