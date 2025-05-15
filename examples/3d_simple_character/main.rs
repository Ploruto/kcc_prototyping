mod plugin;

use avian3d::PhysicsPlugins;
use bevy::{
    pbr::{Atmosphere, light_consts::lux},
    prelude::*,
    render::camera::Exposure,
};
use examples_common::{
    ExampleCommonPlugin,
    camera::{FollowOffset, MainCamera, Targeting},
    input::default_input_contexts,
};
use plugin::{Character, KCCPlugin};

const CHARACTER_RADIUS: f32 = 0.35;
const CHARACTER_CAPSULE_LENGTH: f32 = 1.0;
const MOVEMENT_SPEED: f32 = 8.0;
const GROUND_ACCELERATION: f32 = 100.0;
const AIR_ACCELERATION: f32 = 40.0;
const FRICTION: f32 = 60.0;
const WALKABLE_ANGLE: f32 = std::f32::consts::PI / 4.0;
const JUMP_IMPULSE: f32 = 6.0;
const GRAVITY: f32 = 20.0; // realistic earth gravity tends to feel wrong for games
const STEP_HEIGHT: f32 = 0.25;
const GROUND_CHECK_DISTANCE: f32 = 0.1;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ExampleCommonPlugin,
            PhysicsPlugins::default(),
            KCCPlugin,
        ))
        .add_systems(Startup, setup)
        .run()
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let character = commands
        .spawn((
            Transform::from_xyz(0.0, 10.5, 0.0),
            default_input_contexts(),
            Character::default(),
            Mesh3d(meshes.add(Capsule3d::new(CHARACTER_RADIUS, CHARACTER_CAPSULE_LENGTH))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE.with_alpha(0.25),
                alpha_mode: AlphaMode::Blend,
                ..Default::default()
            })),
        ))
        .id();

    commands.spawn((
        MainCamera,
        Targeting(character),
        FollowOffset {
            absolute: Vec3::Y * CHARACTER_CAPSULE_LENGTH / 2.0,
            ..Default::default()
        },
        Camera {
            hdr: true,
            ..Default::default()
        },
        Msaa::default(),
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
}
