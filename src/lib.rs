use crate::{
    multibody::{Body, MultiBody, NONE_PARENT, R3, SE3, SO3},
    tool::select::{SelectCamera, SelectPlugin, Selectable},
};
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use crate::tool::Tools;

pub mod multibody;
pub mod tool;

pub trait IntoBevy {
    type Bevy;
    fn into_bevy(self) -> Self::Bevy;
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
                        orientation: SO3::default(),
                    },
                    parent: NONE_PARENT,
                },
                Body {
                    transform: SE3 {
                        translation: R3::new(0.0, 1.0, 0.0),
                        orientation: SO3::default(),
                    },
                    parent: NONE_PARENT,
                },
                Body {
                    transform: SE3 {
                        translation: R3::new(0.0, 1.0, 0.0),
                        orientation: SO3::default(),
                    },
                    parent: NONE_PARENT,
                },
            ]),
        }
    }
}

#[derive(Component)]
pub struct TransformVisualizer;

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
        PanOrbitCamera { ..default() },
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
fn visualize_scene(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &TransformVisualizer)>,
    scene: Res<Scene>,
    builtin: Res<Builtin>,
) {
    let target_length = scene.multibody.bodies.len();
    let mut idx = 0usize;
    for (entity, mut transform, _) in query.iter_mut() {
        if idx < target_length {
            let target_transform = scene.multibody.bodies[idx].transform;
            *transform = target_transform.into_bevy();
        } else {
            commands.entity(entity).despawn();
        }
        idx += 1;
    }
    for idx in idx..target_length {
        let target_transform = scene.multibody.bodies[idx].transform;
        commands.spawn((
            target_transform.into_bevy(),
            TransformVisualizer,
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
        app.add_plugins((DefaultPlugins, PanOrbitCameraPlugin, SelectPlugin));
        app.add_systems(Startup, setup);
        app.add_systems(Update, visualize_scene);
    }
}
