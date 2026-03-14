use anatomorph::{AnatomorphPlugin, Builtins, multibody, skeleton};
use anatomorph_math::R3;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(AnatomorphPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, builtins: Res<Builtins>) {
    {
        let chain0 = skeleton::add(
            &mut commands,
            skeleton::pole::Pole::new(
                1.0,
                multibody::Mesh {
                    handle: builtins.cube.clone(),
                    translation: R3::new(0.0, 0.0, 0.5),
                    scale: R3::new(0.2, 0.2, 1.0),
                    ..Default::default()
                },
            ),
            default(),
            None,
        );
        let chain1 = skeleton::add(
            &mut commands,
            skeleton::pole::Pole::new(
                2.0,
                multibody::Mesh {
                    handle: builtins.cube.clone(),
                    translation: R3::new(0.0, 0.0, 0.5),
                    scale: R3::new(0.2, 0.2, 1.0),
                    ..Default::default()
                },
            ),
            default(),
            Some(skeleton::Parent::new(chain0, skeleton::pole::CHILD_END)),
        );
        let chain2 = skeleton::add(
            &mut commands,
            skeleton::pole::Pole::new(
                3.0,
                multibody::Mesh {
                    handle: builtins.cube.clone(),
                    translation: R3::new(0.0, 0.0, 0.5),
                    scale: R3::new(0.2, 0.2, 1.0),
                    ..Default::default()
                },
            ),
            default(),
            Some(skeleton::Parent::new(chain1, skeleton::pole::CHILD_END)),
        );
    }
}
