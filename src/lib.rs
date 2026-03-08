#![feature(try_blocks)]
use crate::{
    multibody::{Body, MultiBody, NONE_PARENT, R3, SE3, SO3},
    tool::{control::ControlPlugin, select::{SelectCamera, SelectPlugin, Selectable}},
};
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use crate::tool::Tools;

pub mod multibody;
pub mod tool;
pub mod bevy_utils;

pub trait IntoBevy {
    type Bevy;
    fn into_bevy(self) -> Self::Bevy;
}
pub trait IntoAnatomorph {
    type Anatomorph;
    fn into_anatomorph(self) -> Self::Anatomorph;
}
pub struct AnatomorphPlugin;
#[derive(Debug, Resource)]
pub struct Scene {
    pub multibody: MultiBody,
}

#[derive(Debug, Resource)]
pub struct Builtin {
    pub transform_visualize_mesh: Handle<Mesh>,
    pub transform_visualize_material: Handle<StandardMaterial>,
}

impl Scene {
    pub fn test() -> Self {
        Self {
            multibody: MultiBody::new(vec![
                Body {
                    transform: SE3 {
                        translation: R3::new(0.0, 0.0, 0.0),
                        rotation: SO3::default(),
                    },
                    parent: NONE_PARENT,
                },
                Body {
                    transform: SE3 {
                        translation: R3::new(0.0, 1.0, 0.0),
                        rotation: SO3::default(),
                    },
                    parent: 0,
                },
                Body {
                    transform: SE3 {
                        translation: R3::new(1.0, 0.0, 0.0),
                        rotation: SO3::default(),
                    },
                    parent: 1,
                },
            ]),
        }
    }
}

#[derive(Component)]
pub struct BodyVisualizer
{
    idx:usize
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
        PanOrbitCamera {button_orbit:MouseButton::Right,button_pan:MouseButton::Middle, ..default() },
        SelectCamera,
    ));
    let sphere = meshes.add(Sphere::new(0.5).mesh().uv(32, 18));
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });
    commands.insert_resource(Builtin {
        transform_visualize_mesh: sphere,
        transform_visualize_material: material,
    });
}
fn update_scene(mut scene: ResMut<Scene>) {
    scene.multibody.update_globbal_transforms();
}
fn visualize_scene(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut BodyVisualizer)>,
    scene: Res<Scene>,
    builtin: Res<Builtin>,
) {
    let target_length = scene.multibody.bodies.len();
    let mut idx = 0usize;
    for (entity, mut transform, mut visualizer) in query.iter_mut() {
        if idx < target_length {
            let target_transform = scene.multibody.global_transform_of(idx);
            *transform = target_transform.into_bevy();
            visualizer.idx = idx;
        } else {
            commands.entity(entity).despawn();
        }
        idx += 1;
    }
    for idx in idx..target_length {
        let target_transform = scene.multibody.global_transform_of(idx);
        commands.spawn((
            target_transform.into_bevy(),
            BodyVisualizer{idx},
            Mesh3d(builtin.transform_visualize_mesh.clone()),
            MeshMaterial3d(builtin.transform_visualize_material.clone()),
            Selectable,
        ));
    }
}

impl Plugin for AnatomorphPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Tools>();
        app.insert_resource(Scene::test());
        app.add_plugins((DefaultPlugins, PanOrbitCameraPlugin, SelectPlugin,ControlPlugin));
        app.add_systems(Startup, setup);
        app.add_systems(Update, (update_scene, visualize_scene));
    }
}
