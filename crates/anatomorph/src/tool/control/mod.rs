use crate::{
    UIGizmo,
    bevy_utils::World2Screen,
    hierarchy,
    multibody::{self},
};
use anatomorph_math::{PI, R1, R2, R3, SO3, bevy::ToBevy as _};
use arrayvec::ArrayVec;
use bevy::prelude::*;
use nalgebra::{Rotation2, Unit};

pub mod swing_twist;

#[derive(Debug, Component)]
pub struct Controlled {
    pub visualize_group: Entity,
}

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(swing_twist::Plugin);
        app.add_systems(Update, (on_add_controllables, update_visual));
    }
}

fn on_add_controllables(
    controllables: Query<Entity, Added<multibody::Controllable>>,
    mut commands: Commands,
) {
    for entity in controllables {
        let visualize_group = commands
            .spawn((Transform::default(), Visibility::Hidden))
            .id();
        commands
            .entity(entity)
            .insert(Controlled { visualize_group });
    }
}

fn update_visual<'a>(
    controlleds: Query<(Entity, &Controlled)>,
    parent_bodies: Query<&'a hierarchy::Parent>,
    children_bodies: Query<&'a hierarchy::Children>,
    screen_positions: Query<&'a multibody::ScreenPositon>,
    mut visualizer_groups: Query<(&mut Transform, &mut Visibility)>,
    mut gizmos: Gizmos<UIGizmo>,
    time: Res<Time>,
) {
    struct Context<'w, 's, 'a> {
        origin: R2,
        max_radius: R1,
        min_radius: R1,
        parent_bodies: &'w Query<'w, 's, &'a hierarchy::Parent>,
        children_bodies: &'w Query<'w, 's, &'a hierarchy::Children>,
        screen_positions: &'w Query<'w, 's, &'a multibody::ScreenPositon>,
    }
    #[derive(Default)]
    struct MutContext {
        parent_offsets: ArrayVec<R2, 2>,
        child_offsets: ArrayVec<R2, 2>,
    }
    fn traverse_parents(ctx: &Context, mctx: &mut MutContext, entity: Entity) {
        let _: Option<()> = try {
            let parent = ctx.parent_bodies.get(entity).ok()?;
            let screen_position = ctx.screen_positions.get(parent.0).unwrap().0?;
            let offset = screen_position - ctx.origin;
            let offset_norm_squared = offset.norm_squared();
            if offset_norm_squared > ctx.min_radius * ctx.min_radius {
                mctx.parent_offsets.push(offset/offset_norm_squared.sqrt());
                if mctx.parent_offsets.is_full() {
                    None?
                }
            }
            if offset_norm_squared < ctx.max_radius * ctx.max_radius {
                traverse_parents(ctx, mctx, parent.0);
            }
        };
    }
    fn traverse_children(ctx: &Context, mctx: &mut MutContext, entity: Entity) {
        let children = ctx.children_bodies.get(entity).unwrap();
        let recursive = children.0.len() == 1;
        for child in children.0.iter().copied() {
            let Some(screen_position) = ctx.screen_positions.get(child).unwrap().0 else {
                continue;
            };
            let offset = screen_position - ctx.origin;
            let offset_norm_squared = offset.norm_squared();
            if offset_norm_squared > ctx.min_radius * ctx.min_radius {
                mctx.parent_offsets.push(offset/offset_norm_squared.sqrt());
            }
            if recursive && offset_norm_squared < ctx.max_radius * ctx.max_radius && !mctx.parent_offsets.is_full() {
                traverse_children(ctx, mctx, child);
            }
        }
    }
    let mut candidate_offsets:[R2;8]= default();
    for i in 0..8{
        candidate_offsets[i] = Rotation2::new(i as R1*PI/4.0)*R2::x();
    }
    for (entity, controlled) in controlleds {
        let (mut transform, mut visibility) = visualizer_groups
            .get_mut(controlled.visualize_group)
            .unwrap();
        let Some(screen_position) = screen_positions.get(entity).unwrap().0 else{
            *visibility = Visibility::Hidden;
            continue;
        };
        let ctx = Context{
            origin: screen_position,
            max_radius: 32.0,
            min_radius: 2.0,
            parent_bodies:&parent_bodies,
            children_bodies:&children_bodies,
            screen_positions: &screen_positions,
        };
        let mut mctx = MutContext::default();
        traverse_parents(&ctx, &mut mctx, entity);
        traverse_children(&ctx, &mut mctx, entity);
        let mut i = 0usize;
        let mut loss = R1::MAX;
        for (this_i,candidate_offset) in candidate_offsets.iter().enumerate(){
            let Some(mut this_loss) = mctx.parent_offsets.iter().chain(mctx.child_offsets.iter()).map(|x|x.dot(&candidate_offset)).max_by(|a,b|a.partial_cmp(b).unwrap())else{break};
            this_loss+= (time.elapsed_secs_wrapped()/16.0+this_i as R1).sin()/64.0;
            if this_loss<loss{
                loss = this_loss;
                i = this_i;   
            }
        }
        let offset = candidate_offsets[i];
        let axis = if offset.x.abs() > offset.y.abs() {
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
        let primary_offset_position = screen_position+offset*32.0;
        let final_offset_position = primary_offset_position + axis.into_inner()*8.0;

        gizmos.circle_2d(
            Isometry2d {
                translation: screen_position.to_bevy(),
                ..Default::default()
            },
            2.0,
            Color::srgb(1.0, 0.7, 0.3),
        );
        gizmos.line_2d(
            screen_position.to_bevy(),
            primary_offset_position.to_bevy(),
            Color::srgb(0.3, 1.0, 0.7),
        );
        gizmos.line_2d(
            primary_offset_position.to_bevy(),
            final_offset_position.to_bevy(),
            Color::srgb(0.3, 1.0, 0.7),
        );
        
        transform.translation = (final_offset_position.push(1.0)).to_bevy();
        transform.rotation = SO3::rotation_between_axis(
            &R3::x_axis(),
            &Unit::new_unchecked(axis.into_inner().push(0.0)),
        )
        .unwrap_or_else(|| SO3::from_axis_angle(&R3::z_axis(), PI))
        .to_bevy();
        *visibility = Visibility::Inherited;
    }
}