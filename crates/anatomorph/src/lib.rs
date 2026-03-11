#![feature(try_blocks)]
use crate::{
    multibody::{Body, BodyMesh, MultiBody, MultiBodyPlugin},
    skeleton::{Skeleton, SkeletonDescriptor, SkeletonPlugin},
    tool::{
        control::ControlPlugin,
        select::{SelectPlugin, Selectable},
    },
};
use anatomorph_math::{Aff3, R3, SE3};
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use crate::tool::Tools;

pub mod bevy_utils;
pub mod multibody;
pub mod skeleton;
pub mod tool;

#[derive(Debug)]
pub struct Dependant<T> {
    pub value: Option<T>,
    pub dirty: bool,
}

impl<T: Default> Default for Dependant<T> {
    fn default() -> Self {
        Self {
            value: Default::default(),
            dirty: true,
        }
    }
}
pub struct AnatomorphPlugin;

#[derive(Debug, Resource)]
pub struct Builtin {
    pub default_material: Handle<StandardMaterial>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut skeleton: ResMut<Skeleton>,
    mut control_settings: ResMut<tool::control::swing_twist::Settings>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(8.0, 8.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 1000.,
            shadows_enabled: false,
            ..default()
        },
        PanOrbitCamera {
            button_orbit: MouseButton::Right,
            button_pan: MouseButton::Middle,
            ..default()
        },
        MeshPickingCamera,
    ));
    let sphere = meshes.add(Sphere::new(0.5).mesh().uv(32, 18));
    let cube = meshes.add(Cuboid::new(0.2, 0.2, 1.0).mesh());
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });
    commands.insert_resource(Builtin {
        default_material: material.clone(),
    });
    control_settings.mesh = sphere;
    control_settings.material = material;
    *skeleton = Skeleton::new(SkeletonDescriptor {
        class: Box::new(skeleton::pole::Pole {
            length: 1.0,
            mesh: BodyMesh {
                handle: cube.clone(),
                translation: R3::new(0.0, 0.0, 0.5),
                ..Default::default()
            },
        }),
        children: vec![(
            skeleton::pole::END,
            SkeletonDescriptor {
                class: Box::new(skeleton::pole::Pole {
                    length: 2.0,
                    mesh: BodyMesh {
                        handle: cube.clone(),
                        translation: R3::new(0.0, 0.0, 0.5),
                        ..Default::default()
                    },
                }),
                children: vec![(
                    skeleton::pole::END,
                    SkeletonDescriptor {
                        class: Box::new(skeleton::pole::Pole {
                            length: 3.0,
                            mesh: BodyMesh {
                                handle: cube.clone(),
                                translation: R3::new(0.0, 0.0, 0.5),
                                ..Default::default()
                            },
                        }),
                        children: vec![],
                    },
                )],
            },
        )],
    })
}

impl Plugin for AnatomorphPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Tools>();
        app.add_plugins((
            DefaultPlugins,
            PanOrbitCameraPlugin,
            MultiBodyPlugin,
            SelectPlugin,
            SkeletonPlugin,
            ControlPlugin,
        ));
        app.add_systems(Startup, setup);
    }
}
