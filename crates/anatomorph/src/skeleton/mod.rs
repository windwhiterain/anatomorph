use bevy::ecs::system::SystemChangeTick;
use bevy::ecs::{change_detection::Tick, system::SystemState};
use bevy::prelude::*;

use crate::multibody::{self, MultiBody, MultiBodyPlugin};

pub mod pole;

pub trait BoneClass: Sync + Send + std::fmt::Debug {
    fn bodies_len(&self) -> usize;
    fn update(&self, bone: &Bone, bodies: &mut [multibody::Body]);
}

#[derive(Debug)]
pub struct Bone {
    pub class: Box<dyn BoneClass>,
    pub body_offset: usize,
    pub parent: Option<BoneParent>,
}
#[derive(Debug)]
pub struct BoneParent {
    idx: usize,
    body_offset: usize,
}
#[derive(Debug, Resource, Default)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
    pub bodies_len: usize,
    pub ticks: Vec<Tick>,
}
impl Skeleton {
    pub fn new(descriptor: SkeletonDescriptor) -> Self {
        struct Context {
            skeleton: Skeleton,
        }
        impl Context {
            fn traverse(&mut self, descriptor: SkeletonDescriptor, parent: Option<BoneParent>) {
                let body_offset = self.skeleton.bodies_len;
                let idx: usize = self.skeleton.bones.len();
                self.skeleton.bodies_len += descriptor.class.bodies_len();
                self.skeleton.bones.push(Bone {
                    class: descriptor.class,
                    body_offset,
                    parent,
                });
                for (body_offset, child) in descriptor.children {
                    self.traverse(child, Some(BoneParent { idx, body_offset }));
                }
            }
        }
        let mut ctx = Context {
            skeleton: default(),
        };
        ctx.traverse(descriptor, None);
        ctx.skeleton
            .ticks
            .resize_with(ctx.skeleton.bones.len(), default);
        ctx.skeleton
    }
}
pub struct SkeletonDescriptor {
    pub class: Box<dyn BoneClass>,
    pub children: Vec<(usize, SkeletonDescriptor)>,
}
pub fn update_multibody(
    skeleton: Res<Skeleton>,
    mut multibody: ResMut<MultiBody>,
    system_change_tick: SystemChangeTick,
) {
    if skeleton.is_changed() {
        multibody.bodies.resize_with(skeleton.bodies_len, default);
        for idx in 0..skeleton.bones.len() {
            let tick = skeleton.ticks[idx];
            if tick.is_newer_than(system_change_tick.last_run(), system_change_tick.this_run()) {
                let bone = &skeleton.bones[idx];
                let bodies = &mut multibody.bodies
                    [bone.body_offset..bone.body_offset + bone.class.bodies_len()];
                bone.class.update(bone, bodies);
                bodies[0].parent = if let Some(parent) = &bone.parent {
                    Some(skeleton.bones[parent.idx].body_offset + parent.body_offset)
                } else {
                    None
                };
            }
        }
    }
}
pub struct SkeletonPlugin;
impl Plugin for SkeletonPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<MultiBodyPlugin>() {
            app.add_plugins(MultiBodyPlugin);
        }
        app.init_resource::<Skeleton>();
        app.add_systems(Update, update_multibody);
    }
}
