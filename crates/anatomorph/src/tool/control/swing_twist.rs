use anatomorph_math::{
    R1, R2, R3, SE3, SO3,
    bevy::{ToAnatomorph as _, ToBevy as _},
};
use bevy::{camera::visibility::RenderLayers, prelude::*};
use nalgebra::Unit;

use crate::{
    Builtins, MainCamera,
    bevy_utils::{AddDendencyPlugin, World2Screen},
    hierarchy,
    multibody::{
        self,
        joint::{self},
    },
    tool::{ToolPlugin, control::swing_twist},
};

#[derive(Debug, Component)]
#[require(Pickable)]
pub struct Visualizer {
    pub entity: Entity,
}

pub struct Plugin;

impl ToolPlugin for Plugin {}

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        Self::register(app);
        app.add_dependency_plugin(|| multibody::Plugin);
        app.add_systems(
            Update,
            (
                update_visual.run_if(Self::enbale_condition),
                on_drag.run_if(Self::enbale_condition),
            ),
        );
    }
}

fn update_visual(
    mut commands: Commands,
    bodies: Query<(Entity, &multibody::GlobalTransform, &joint::SwingTwist)>,
    mut visualizers: Query<(Entity, &mut Transform, &mut Visualizer)>,
    builtins: Res<Builtins>,
    world2screen: World2Screen,
) {
    let mut joints = bodies
        .iter()
        .filter_map(|(entity, global_transform, swing_twist)| {
            if let Some(screen_position) = world2screen.world2screen(global_transform.0.translation)
            {
                Some((
                    entity,
                    Transform {
                        translation: screen_position.push(0.0).to_bevy(),
                        scale: R3::repeat(16.0).to_bevy(),
                        ..Default::default()
                    },
                ))
            } else {
                None
            }
        });
    for (entity, mut transform, mut visualizer) in visualizers.iter_mut() {
        if let Some((idx, target_transform)) = joints.next() {
            *transform = target_transform;
            visualizer.entity = idx;
        } else {
            commands.entity(entity).despawn();
        }
    }
    for (idx, target_transform) in joints {
        commands.spawn((
            target_transform,
            Visualizer { entity: idx },
            Mesh2d(builtins.rect.clone()),
            MeshMaterial2d(builtins.yellow.clone()),
            RenderLayers::layer(1),
        ));
    }
}

fn on_drag(
    mut events: MessageReader<Pointer<Drag>>,
    mut swing_twists: Query<(
        &mut joint::SwingTwist,
        Option<&hierarchy::Parent>,
        &multibody::GlobalTransform,
    )>,
    visualiers: Query<&Visualizer>,
    global_transforms: Query<&multibody::GlobalTransform>,
    camera_global_transform: Query<&GlobalTransform, With<MainCamera>>,
) {
    for event in events.read() {
        let _: Option<()> = try {
            let entity = event.entity;
            let entity = visualiers.get(entity).ok()?.entity;
            let (mut swing_twist, parent, global_transform) = swing_twists.get_mut(entity).unwrap();
            let camera_transform = camera_global_transform.single().unwrap();
            let camera_translation = camera_transform.translation().to_anatomorph();
            let camera_rotation = camera_transform.rotation().to_anatomorph();
            let translation = global_transform.0.translation;
            let look_direction = Unit::new_normalize(translation - camera_translation);
            let camera_rotation =
                SO3::rotation_between_axis(&(camera_rotation * (-R3::z_axis())), &look_direction)?
                    * camera_rotation;
            let (camera2joint, joint_look_direction) = if let Some(parent) = parent {
                let world2joint = global_transforms
                    .get(parent.0)
                    .unwrap()
                    .0
                    .rotation
                    .inverse();

                (world2joint * camera_rotation, world2joint * look_direction)
            } else {
                (camera_rotation, look_direction)
            };
            let delta = event.delta.to_anatomorph();
            let delta_reject =
                camera2joint * delta.component_mul(&R2::new(1.0, -1.0)).push(0.0) / 256.0;
            let swing = swing_twist.swing;
            let project_coef = swing.dot(&joint_look_direction);
            let project = project_coef * joint_look_direction.into_inner();
            let reject = swing.into_inner() - project + delta_reject;
            let (reject, reject_norm_squared) = {
                const SAFE_NORM: R1 = 0.9;
                const SAFE_NORM_SQUARED: R1 = SAFE_NORM * SAFE_NORM;
                let norm_squared = reject.norm_squared();
                if norm_squared < SAFE_NORM_SQUARED {
                    (reject, norm_squared)
                } else {
                    (reject / norm_squared.sqrt() * SAFE_NORM, SAFE_NORM_SQUARED)
                }
            };
            let project = if project_coef > 0.0 { 1.0 } else { -1.0 }
                * (1.0 - reject_norm_squared).sqrt()
                * joint_look_direction.into_inner();
            swing_twist.swing = Unit::new_unchecked(project + reject);
        };
    }
}
