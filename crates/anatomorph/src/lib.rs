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
use bevy::{camera::visibility::RenderLayers, prelude::*};
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

#[derive(Debug, Resource,Default)]
pub struct Builtins {
    pub default_material: Handle<StandardMaterial>,
    pub circle:Handle<Mesh>,
    pub rect:Handle<Mesh>,
    pub cube: Handle<Mesh>,
    pub sphere: Handle<Mesh>,
    pub yellow: Handle<ColorMaterial>,
    pub green: Handle<ColorMaterial>,
}

#[derive(Debug, Component)]
pub struct MainCamera;
#[derive(Debug, Component)]
pub struct UICamera;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut skeleton: ResMut<Skeleton>,
    mut builtins: ResMut<Builtins>,
) {
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Camera {
            ..Default::default()
        },
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
    commands.spawn((
        UICamera,
        Camera2d::default(),
        RenderLayers::layer(1),
        Camera {
            order: 1,
            ..Default::default()
        },
    ));
    let circle = meshes.add(Circle::new(0.5).mesh());
    let rect = meshes.add(Rectangle::new(1.0,1.0).mesh());
    let sphere = meshes.add(Circle::new(0.5).mesh());
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0).mesh());
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });
    let yellow = color_materials.add(ColorMaterial {
        color: Color::Srgba(Srgba { red: 1.0, green: 1.0, blue: 0.4, alpha: 0.2 }),
        ..default()
    });
    let green = color_materials.add(ColorMaterial {
        color: Color::Srgba(Srgba { red: 0.6, green: 1.0, blue: 0.6, alpha: 0.2 }),
        ..default()
    });
    builtins.default_material = material;
    builtins.circle = circle;
    builtins.rect = rect;
    builtins.sphere = sphere;
    builtins.cube = cube;
    builtins.yellow = yellow;
    builtins.green = green;
    *skeleton = Skeleton::new(SkeletonDescriptor {
        class: Box::new(skeleton::pole::Pole {
            length: 1.0,
            mesh: BodyMesh {
                handle: builtins.cube.clone(),
                translation: R3::new(0.0, 0.0, 0.5),
                scale: R3::new(0.2,0.2,1.0),
                ..Default::default()
            },
        }),
        children: vec![(
            skeleton::pole::END,
            SkeletonDescriptor {
                class: Box::new(skeleton::pole::Pole {
                    length: 2.0,
                    mesh: BodyMesh {
                        handle: builtins.cube.clone(),
                        translation: R3::new(0.0, 0.0, 0.5),
                        scale: R3::new(0.2,0.2,1.0),
                        ..Default::default()
                    },
                }),
                children: vec![(
                    skeleton::pole::END,
                    SkeletonDescriptor {
                        class: Box::new(skeleton::pole::Pole {
                            length: 3.0,
                            mesh: BodyMesh {
                                handle: builtins.cube.clone(),
                                translation: R3::new(0.0, 0.0, 0.5),
                                scale: R3::new(0.2,0.2,1.0),
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
        app.init_resource::<Builtins>();
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
