use anatomorph_math::{
    R2, R3,
    bevy::{ToAnatomorph as _, ToBevy as _},
};
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(SystemParam)]
pub struct World2Pixel<'w, 's> {
    camera: Query<'w, 's, (&'static Camera, &'static GlobalTransform)>,
}

impl<'w, 's> World2Pixel<'w, 's> {
    pub fn world2pixel(&self, position_world: R3) -> Option<R2> {
        let (camera, camera_global_transform) = self.camera.single().unwrap();

        // 1. World → NDC (Normalized Device Coordinates)
        let ndc = camera
            .world_to_ndc(camera_global_transform, position_world.to_bevy())?
            .to_anatomorph();

        // If behind camera, ndc.z < 0
        if ndc.z < 0.0 {
            return None;
        }

        // 2. NDC → Pixel coordinates
        let window_size = camera.logical_target_size().unwrap().to_anatomorph();
        let position_pixel = ((ndc.xy().component_mul(&R2::new(1.0, -1.0)) + R2::new(1.0, 1.0))
            / 2.0)
            .component_mul(&window_size);

        Some(position_pixel)
    }
}
