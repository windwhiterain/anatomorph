use crate::{
    UIGizmo, bevy_utils::World2Screen, multibody::{self}
};
use anatomorph_math::{R1, R2, bevy::ToBevy as _};
use bevy::prelude::*;
use nalgebra::Unit;

pub mod swing_twist;

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(swing_twist::Plugin);
        app.add_systems(Update, (attach_controller,update));
    }
}

#[derive(Debug, Component)]
pub struct Controller {
    pub screen_position: Option<R2>,
    pub axis: Unit<R2>,
    pub offset: R2,
}

fn attach_controller(
    controllables: Query<Entity, Added<multibody::Controllable>>,
    mut commands: Commands,
) {
    for entity in controllables {
        commands.entity(entity).insert(Controller {
            screen_position: default(),
            axis: -R2::x_axis(),
            offset: default(),
        });
    }
}

fn update(
    mut controllers: Query<(&mut Controller, &multibody::GlobalTransform)>,
    mut gizmos: Gizmos<UIGizmo>,
    world2screen: World2Screen,
) {
    let mut n = 0usize;
    let mut average: R2 = default();
    for (mut controller, global_transform) in &mut controllers {
        let screen_position = world2screen.world2screen(global_transform.0.translation);
        controller.screen_position = screen_position;
        if let Some(screen_position) = screen_position {
            if n == 0 {
                average = screen_position;
            } else {
                average = average + (screen_position - average) / (n + 1) as R1;
            }
            n+=1;
        }
    }
    for (mut controller, _) in &mut controllers {
        if let Some(screen_position) = controller.screen_position {
            let offset = screen_position - average;
            controller.axis = if offset.x.abs() > offset.y.abs() {
                if offset.x > 0.0 {
                    R2::x_axis()
                } else {
                    -R2::x_axis()
                }
            } else {
                if offset.y > 0.0 {
                    R2::y_axis()
                } else {
                    -R2::y_axis()
                }
            };
            let mut offset = offset/8.0;
            let offset_norm_squared = offset.norm_squared();
            if offset_norm_squared >(32*32) as R1{
                offset*= ((32*32) as R1/offset_norm_squared).sqrt()
            }
            controller.offset = offset + controller.axis.into_inner() * 16.0;

            gizmos.circle_2d(Isometry2d{translation:screen_position.to_bevy(),..Default::default()}, 2.0, Color::srgb(1.0, 0.7, 0.3));
            gizmos.line_2d(
                screen_position.to_bevy(),
                (screen_position + offset).to_bevy(),
                Color::srgb(0.3, 1.0, 0.7),
            );
            gizmos.line_2d(
                (screen_position + offset).to_bevy(),
                (screen_position + offset + controller.axis.into_inner() * 16.0).to_bevy(),
                Color::srgb(0.3, 1.0, 0.7),
            );
        }
    }
}
