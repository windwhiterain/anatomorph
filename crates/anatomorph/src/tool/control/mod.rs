use crate::{Builtins, bevy_utils::World2Screen, multibody::{self, MultiBody}, tool::ToolPlugin};
use anatomorph_math::R2;
use bevy::prelude::*;
use nalgebra::Unit;

pub mod swing_twist;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(swing_twist::Plugin);
    }
}

// #[derive(Debug, Component)]
// pub struct Visualizer;

// pub struct Controller{
//     pub axis:Unit<R2>,
//     pub offset:R2,
// }
// pub struct Controllers{
//     pub controllers: Vec<Controller>,
// }


// fn visualize(
//     mut commands: Commands,
//     multibody: Res<MultiBody>,
//     global_transforms: Res<multibody::GlobalTransforms>,
//     mut controller: Query<(Entity, &mut Transform),With<Visualizer>>,
//     settings: Res<Builtins>,
//     world2screen: World2Screen,
// ) {
//     let mut joints = multibody.swing_twist_controllers.iter().filter_map(|idx| {
//         let body_idx = multibody.swing_twist_joints[idx].body;
//         if let Some(target_transform) = global_transforms.global_transforms.get(body_idx) {
//             if let Some(screen_position) = world2screen.world2screen(target_transform.translation) {
//                 Some((
//                     idx,
//                     Transform {
//                         translation: screen_position.push(1.0).to_bevy(),
//                         scale: R3::repeat(16.0).to_bevy(),
//                         ..Default::default()
//                     },
//                 ))
//             } else {
//                 None
//             }
//         } else {
//             None
//         }
//     });
//     for (entity, mut transform, mut visualizer) in controller.iter_mut() {
//         if let Some((idx, target_transform)) = joints.next() {
//             *transform = target_transform;
//             visualizer.idx = idx;
//         } else {
//             commands.entity(entity).despawn();
//         }
//     }
//     for (idx, target_transform) in joints {
//         commands.spawn((
//             target_transform,
//             Visualizer { idx },
//             Mesh2d(settings.rect.clone()),
//             MeshMaterial2d(settings.yellow.clone()),
//             RenderLayers::layer(1),
//         ));
//     }
// }
