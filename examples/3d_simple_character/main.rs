mod plugin;

use avian3d::PhysicsPlugins;
use bevy::prelude::*;
use examples_common::{
    ExampleCommonPlugin,
    camera::{CameraTargetOf, MainCamera},
    input::default_input_contexts,
};
use plugin::Character;

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
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, set_main_camera_target)
        .run()
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Transform::from_xyz(0.0, 10.5, 0.0),
        default_input_contexts(),
        Character::default(),
        Mesh3d(meshes.add(Capsule3d::new(CHARACTER_RADIUS, CHARACTER_CAPSULE_LENGTH))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE.with_alpha(0.25),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        })),
    ));
}

fn set_main_camera_target(
    mut commands: Commands,
    main_camera: Single<Entity, Added<MainCamera>>,
    character: Single<Entity, With<Character>>,
) {
    let main_camera = main_camera.into_inner();
    let character = character.into_inner();
    commands
        .entity(character)
        .insert(CameraTargetOf(main_camera));
}
