use anatomorph_math::{R3, SE3, SO3, bevy::ToBevy};
use bevy::{ecs::system::Query, prelude::*};

use crate::{Builtins, hierarchy};

pub mod joint;

#[derive(Debug, Default, Component)]
pub struct Transform(pub SE3);

#[derive(Debug, Default, Component)]
pub struct GlobalTransform(pub SE3);

#[derive(Debug, Clone, Component)]
#[require(Mesh3d,MeshMaterial3d<StandardMaterial>)]
pub struct Mesh {
    pub handle: Handle<bevy::prelude::Mesh>,
    pub translation: R3,
    pub scale: R3,
    pub rotation: SO3,
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            handle: Default::default(),
            translation: Default::default(),
            scale: R3::repeat(1.0),
            rotation: Default::default(),
        }
    }
}

#[derive(Debug,Default,Component)]
pub struct Controllable;

pub fn add<T: joint::Class>(
    commands: &mut Commands,
    joint_class: T,
    parent: Option<Entity>,
    mesh: Option<Mesh>,
    controllable:bool,
) -> Entity {
    let mut cmd = commands.spawn((
        Transform::default(),
        GlobalTransform::default(),
        hierarchy::Children::default(),
        joint_class,
    ));
    if let Some(parent) = parent {
        cmd.insert(hierarchy::Parent(parent));
    }
    if let Some(mesh) = mesh {
        cmd.insert(mesh);
    }
    if controllable{
        cmd.insert(Controllable);
    }
    cmd.id()
}

pub fn parent(commands: &mut Commands, child: Entity, parent: Entity) {
    commands.entity(child).insert(hierarchy::Parent(parent));
}

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (on_change_transforms, update_visual));
        joint::register::<joint::Free>(app);
        joint::register::<joint::SwingTwist>(app);
    }
}

fn on_change_transforms<'a>(
    roots: Query<Entity, (Without<hierarchy::Parent>, With<Transform>)>,
    transforms: Query<(&'a Transform, &'a hierarchy::Children)>,
    global_transforms: Query<&mut GlobalTransform>,
) {
    struct Context<'w, 's, 'a> {
        transforms: Query<'w, 's, (&'a Transform, &'a hierarchy::Children)>,
    }
    struct MutContext<'w, 's, 'a> {
        global_transforms: Query<'w, 's, &'a mut GlobalTransform>,
    }
    let ctx = Context { transforms };
    let mut mctx = MutContext { global_transforms };
    fn propagate(ctx: &Context, mctx: &mut MutContext, entity: Entity, parent_transform: SE3) {
        let (transform, children) = ctx.transforms.get(entity).unwrap();
        let transform = parent_transform * transform.0;
        mctx.global_transforms.get_mut(entity).unwrap().0 = transform;
        for child in children.0.iter().copied() {
            propagate(ctx, mctx, child, transform);
        }
    }

    for root in roots {
        propagate(&ctx, &mut mctx, root, default());
    }
}
fn update_visual(
    visual_bodies: Query<(
        &GlobalTransform,
        &Mesh,
        &mut bevy::prelude::Transform,
        &mut bevy::prelude::Mesh3d,
        &mut bevy::prelude::MeshMaterial3d<StandardMaterial>,
    )>,
    builtin: Res<Builtins>,
) {
    for (transform, mesh, mut bevy_transform, mut bevy_mesh, mut bevy_material) in visual_bodies {
        *bevy_transform = bevy::prelude::Transform {
            translation: (transform.0.translation + transform.0.rotation * mesh.translation.component_mul(&mesh.scale))
                .to_bevy(),
            rotation: (transform.0.rotation * mesh.rotation).to_bevy(),
            scale: mesh.scale.to_bevy(),
        };
        if bevy_mesh.0 != mesh.handle {
            bevy_mesh.0 = mesh.handle.clone();
        }
        if bevy_material.0 != builtin.default_material {
            bevy_material.0 = builtin.default_material.clone();
        }
    }
}
