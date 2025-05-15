pub mod camera;
pub mod input;
pub mod level;

use avian3d::prelude::*;
use bevy::{pbr::light_consts::lux, prelude::*};
use camera::CameraPlugin;
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
