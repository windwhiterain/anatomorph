use bevy::{
    DefaultPlugins,
    app::{App, Startup, Update},
    asset::Assets,
    ecs::system::{Commands, ResMut},
    math::primitives::Sphere,
    mesh::{Mesh, Mesh3d, Meshable},
    prelude::*,
    transform::components::Transform,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
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
    ));
    let sphere = meshes.add(Sphere::new(0.5).mesh());
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });
    commands.spawn((
        Mesh3d(sphere),
        Transform::default(),
        MeshMaterial3d(material),
    ));
}
