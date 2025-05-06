pub mod fly_camera;
pub mod fps_camera;
pub mod orbit_camera;

use crate::{
    input::{FlyCameraContext, OrbitCameraContext, ToggleCameraMode},
    movement::{Character, Frozen},
};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use fly_camera::{FlyCamera, FlyCameraPlugin};
use fps_camera::{FpsCamera, FpsCameraPlugin};
use orbit_camera::{OrbitCamera, OrbitCameraPlugin, PreventBlindness};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((FpsCameraPlugin, OrbitCameraPlugin, FlyCameraPlugin))
            .add_observer(cycle_camera_modes);
    }
}

// the CameraMode could potentially be out of sync with the type of the actual camera controller
// (maybe add component hook)
#[derive(Component)]
#[require(Camera3d, FpsCamera)]
pub struct MainCamera {
    pub mode: CameraMode,
    pub sensitivity: f32,
}

impl Default for MainCamera {
    fn default() -> Self {
        Self {
            mode: CameraMode::default(),
            sensitivity: 1.0,
        }
    }
}

#[derive(Default, Debug)]
pub enum CameraMode {
    #[default]
    Fps,
    Orbit,
    Fly,
}

#[derive(Component)]
#[relationship(relationship_target = Target)]
pub struct TargetOf(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = TargetOf)]
pub struct Target(Entity);

fn cycle_camera_modes(
    _: Trigger<Fired<ToggleCameraMode>>,
    main_camera: Single<(&mut MainCamera, Entity)>,
    player: Single<Entity, With<Character>>,
    mut commands: Commands,
) {
    let (mut main_cam, main_cam_entity) = main_camera.into_inner();
    let player_entity = player.into_inner();

    match main_cam.mode {
        CameraMode::Fps => {
            commands
                .entity(main_cam_entity)
                .remove::<FpsCamera>()
                .insert((OrbitCamera::default(), PreventBlindness::default()));
            main_cam.mode = CameraMode::Orbit;

            commands
                .entity(player_entity)
                .insert(Actions::<OrbitCameraContext>::default());
        }
        CameraMode::Orbit => {
            // the player shouldn't be controllable while in fly-cam-mode
            commands.entity(player_entity).insert(Frozen);
            commands
                .entity(main_cam_entity)
                .remove::<OrbitCamera>()
                .insert(FlyCamera::default());
            main_cam.mode = CameraMode::Fly;

            commands
                .entity(player_entity)
                .remove::<Actions<OrbitCameraContext>>()
                .insert(Actions::<FlyCameraContext>::default());
        }
        CameraMode::Fly => {
            // allow for player movement once in fps-mode
            commands.entity(player_entity).remove::<Frozen>();
            commands
                .entity(main_cam_entity)
                .remove::<FlyCamera>()
                .insert(FpsCamera);
            main_cam.mode = CameraMode::Fps;

            commands
                .entity(player_entity)
                .remove::<Actions<FlyCameraContext>>();
        }
    }
}
