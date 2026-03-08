use bevy::prelude::*;
use nalgebra::Unit;

use crate::{
    BodyVisualizer, IntoAnatomorph, bevy_utils::World2Pixel, multibody::SO3, tool::ToolPlugin,
};

pub struct ControlPlugin;

fn on_drag(
    mut events: MessageReader<Pointer<Drag>>,
    mut scene: ResMut<crate::Scene>,
    body_visualizer: Query<&BodyVisualizer>,
    global_transform: Query<&GlobalTransform>,
    world_to_pixel: World2Pixel,
    camera_global_transform: Query<&GlobalTransform, With<Camera>>,
) {
    for event in events.read() {
        let _: Option<()> = try {
            let entity = event.entity;
            let idx = body_visualizer.get(entity).ok()?.idx;
            let parent = scene.multibody.parent_of(idx);
            let camera_translation_world = camera_global_transform
                .single()
                .unwrap()
                .translation()
                .into_anatomorph();
            let center_world = global_transform
                .get(entity)
                .unwrap()
                .translation()
                .into_anatomorph();
            let center_pixel = world_to_pixel.world2pixel(center_world)?;
            let arc = event.delta.into_anatomorph();
            let radius = event.pointer_location.position.into_anatomorph() - center_pixel;
            let angle = radius.perp(&arc) / radius.norm_squared();
            let axis = Unit::new_normalize(center_world - camera_translation_world);
            let global_rotation = SO3::from_axis_angle(&axis, angle);
            let rotation = if let Some(parent) = parent {
                let parent_global_rotation = scene.multibody.global_transform_of(parent).rotation;
                parent_global_rotation.conjugate() * global_rotation * parent_global_rotation
            } else {
                global_rotation
            };
            let mut transform = scene.multibody.transform_of(idx);
            transform.rotation = rotation * transform.rotation;
            scene.multibody.set_transform_of(idx, transform);
        };
    }
}

impl ToolPlugin for ControlPlugin {}

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        Self::register(app);
        app.add_systems(Update, on_drag.run_if(Self::enbale_condition));
    }
}
