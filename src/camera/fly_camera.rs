use crate::input::{self, FlyVerticalMoveDown, FlyVerticalMoveUp, Look};
use bevy::{input::gamepad::GamepadInput, prelude::*};
use bevy_enhanced_input::prelude::*;
use std::f32::consts::FRAC_PI_2;

use super::MainCamera;

pub struct FlyCameraPlugin;

impl Plugin for FlyCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(look)
            .add_observer(fly)
            .add_observer(vertical_fly_up)
            .add_observer(vertical_fly_down);
    }
}

#[derive(Component)]
pub struct FlyCamera {
    speed: f32,
}

impl Default for FlyCamera {
    fn default() -> Self {
        Self { speed: 10.0 }
    }
}

fn fly(
    trigger: Trigger<Fired<input::Move>>,
    camera: Single<(&mut Transform, &FlyCamera)>,
    time: Res<Time>,
) {
    let (mut transform, cam) = camera.into_inner();
    let direction = transform.rotation * Vec3::new(trigger.value.x, 0.0, -trigger.value.y);
    transform.translation += direction * cam.speed * time.delta_secs();
}

fn vertical_fly_up(
    _trigger: Trigger<Fired<FlyVerticalMoveUp>>,
    camera: Single<(&mut Transform, &FlyCamera)>,
    time: Res<Time>,
) {
    let (mut transform, cam) = camera.into_inner();
    transform.translation.y += cam.speed * time.delta_secs();
}

fn vertical_fly_down(
    _trigger: Trigger<Fired<FlyVerticalMoveDown>>,
    camera: Single<(&mut Transform, &FlyCamera)>,
    time: Res<Time>,
) {
    let (mut transform, cam) = camera.into_inner();
    transform.translation.y -= cam.speed * time.delta_secs();
}

fn look(
    trigger: Trigger<Fired<Look>>,
    camera: Single<(&mut Transform, &MainCamera), With<FlyCamera>>,
) {
    let (mut transform, main_camera) = camera.into_inner();
    let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
    yaw += trigger.value.x.to_radians() * main_camera.sensitivity;
    pitch += trigger.value.y.to_radians() * main_camera.sensitivity;
    pitch = pitch.clamp(-FRAC_PI_2, FRAC_PI_2);
    transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
}
