pub mod camera;
pub mod input;
pub mod level;

use avian3d::prelude::*;
use bevy::{
    pbr::{Atmosphere, light_consts::lux},
    prelude::*,
    render::camera::Exposure,
};
use camera::{CameraPlugin, FollowOffset, MainCamera};
use input::InputPlugin;
use level::LevelGeneratorPlugin;

pub struct ExampleCommonPlugin;

impl Plugin for ExampleCommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InputPlugin,
            CameraPlugin,
            LevelGeneratorPlugin,
            PhysicsDebugPlugin::default(),
            PhysicsDiagnosticsPlugin,
            PhysicsDiagnosticsUiPlugin,
        ))
        .add_systems(Startup, setup);
    }
}

// Marker component used to freeze player movement when the main camera is in fly-mode.
// This shouldn't be strictly necessary if we figure out how to properly layer InputContexts.
#[derive(Component)]
pub struct Frozen;

fn setup(mut commands: Commands) {
    commands.spawn((
        MainCamera,
        FollowOffset::default(),
        // FollowOffset {
        //     absolute: Vec3::Y * EXAMPLE_CHARACTER_CAPSULE_LENGTH / 2.0,
        //     ..Default::default()
        // },
        Camera {
            hdr: true,
            ..Default::default()
        },
        Camera3d::default(),
        Atmosphere::EARTH,
        Exposure::SUNLIGHT,
        Projection::Perspective(PerspectiveProjection {
            fov: 90.0_f32.to_radians(),
            ..Default::default()
        }),
        AmbientLight {
            brightness: lux::AMBIENT_DAYLIGHT,
            ..Default::default()
        },
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // Sun
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: lux::RAW_SUNLIGHT,
            ..default()
        },
        Transform::from_xyz(0.0, 2.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
