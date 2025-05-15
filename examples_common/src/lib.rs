pub mod camera;
pub mod input;
pub mod level;
pub mod movement;

use avian3d::prelude::*;
use bevy::{
    pbr::{Atmosphere, light_consts::lux},
    prelude::*,
    render::camera::Exposure,
};
use bevy_enhanced_input::prelude::Actions;
use camera::{CameraPlugin, FollowOffset, MainCamera};
use input::{DefaultContext, FlyCameraContext, InputPlugin, OrbitCameraContext};
use level::LevelGeneratorPlugin;
use movement::{Character, KCCPlugin};

pub const EXAMPLE_CHARACTER_RADIUS: f32 = 0.35;
pub const EXAMPLE_CHARACTER_CAPSULE_LENGTH: f32 = 1.0;
pub const EXAMPLE_MOVEMENT_SPEED: f32 = 8.0;
pub const EXAMPLE_GROUND_ACCELERATION: f32 = 100.0;
pub const EXAMPLE_AIR_ACCELERATION: f32 = 40.0;
pub const EXAMPLE_FRICTION: f32 = 60.0;
pub const EXAMPLE_WALKABLE_ANGLE: f32 = std::f32::consts::PI / 4.0;
pub const EXAMPLE_JUMP_IMPULSE: f32 = 6.0;
pub const EXAMPLE_GRAVITY: f32 = 20.0; // realistic earth gravity tend to feel wrong for games
pub const EXAMPLE_STEP_HEIGHT: f32 = 0.25;
pub const EXAMPLE_GROUND_CHECK_DISTANCE: f32 = 0.1;

#[derive(Component)]
#[relationship(relationship_target = Attachments)]
pub struct AttachedTo(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = AttachedTo)]
pub struct Attachments(Vec<Entity>); // not sure about generaling this 

pub struct ExampleCommonPlugin;

impl Plugin for ExampleCommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InputPlugin,
            CameraPlugin,
            LevelGeneratorPlugin,
            KCCPlugin,
            PhysicsDiagnosticsPlugin,
            PhysicsDiagnosticsUiPlugin,
        ))
        .add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Transform::from_xyz(0.0, 10.5, 0.0),
        Actions::<DefaultContext>::default(),
        Actions::<FlyCameraContext>::default(),
        Actions::<OrbitCameraContext>::default(),
        Character::default(),
        Mesh3d(meshes.add(Capsule3d::new(
            EXAMPLE_CHARACTER_RADIUS,
            EXAMPLE_CHARACTER_CAPSULE_LENGTH,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE.with_alpha(0.25),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        })),
        Attachments::spawn_one((
            MainCamera,
            FollowOffset {
                absolute: Vec3::Y * EXAMPLE_CHARACTER_CAPSULE_LENGTH / 2.0,
                ..Default::default()
            },
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
        )),
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
