use super::{MainCamera, TargetOf};
use crate::input::Look;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use std::f32::consts::FRAC_PI_2;

pub struct FpsCameraPlugin;

impl Plugin for FpsCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(look)
            .add_systems(FixedPostUpdate, follow_target);
    }
}

#[derive(Default, Component)]
pub struct FpsCamera;

fn follow_target(
    targets: Query<(&TargetOf, &Transform), Without<FpsCamera>>,
    mut cameras: Query<&mut Transform, With<FpsCamera>>,
) {
    for (target_of, target_transform) in &targets {
        if let Ok(mut cam_transform) = cameras.get_mut(target_of.0) {
            cam_transform.translation = target_transform.translation;
        }
    }
}

fn look(
    trigger: Trigger<Fired<Look>>,
    camera: Single<(&mut Transform, &MainCamera), With<FpsCamera>>,
) {
    let (mut transform, main_camera) = camera.into_inner();
    let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
    yaw += trigger.value.x.to_radians() * main_camera.sensitivity;
    pitch += trigger.value.y.to_radians() * main_camera.sensitivity;
    pitch = pitch.clamp(-FRAC_PI_2, FRAC_PI_2);
    transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
}
