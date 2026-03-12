use anatomorph_math::{
    R1, R2, R3, SE3, SO3, bevy::{ToAnatomorph as _, ToBevy as _}
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
            let delta_reject = 
                camera2joint * delta.component_mul(&R2::new(1.0, -1.0)).push(0.0)/256.0;
            let joint = &mut multibody.joints.swing_twist[idx];
            let swing = joint.class.swing;
            let project_coef = swing.dot(&joint_look_direction);
            let project = project_coef * joint_look_direction.into_inner();
            let reject = swing.into_inner() - project + delta_reject;
            let (reject,reject_norm_squared) = {
                const SAFE_NORM:R1 = 0.9;
                const SAFE_NORM_SQUARED:R1 = SAFE_NORM*SAFE_NORM;
                let norm_squared = reject.norm_squared();
                if norm_squared<SAFE_NORM_SQUARED{
                    (reject,norm_squared)
                }else{
                    (reject/norm_squared.sqrt()*SAFE_NORM,SAFE_NORM_SQUARED)
                }
            };
            let project = if project_coef > 0.0 {1.0}else{-1.0}*(1.0-reject_norm_squared).sqrt()*joint_look_direction.into_inner();
            joint.class.swing = Unit::new_unchecked(project + reject);
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
