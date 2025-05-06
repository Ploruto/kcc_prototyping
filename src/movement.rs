use std::{f32::consts::PI, time::Duration};

use avian3d::prelude::{
    Collider, CollisionLayers, RigidBody, Sensor, ShapeHitData, SpatialQuery, SpatialQueryFilter,
};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{ActionState, Actions};

use crate::{
    camera::MainCamera,
    input::{self, DefaultContext, Jump},
    move_and_slide::{MoveAndSlideConfig, character_sweep, move_and_slide},
};

const EXAMPLE_MOVEMENT_SPEED: f32 = 8.0;
const EXAMPLE_FLOOR_ACCELERATION: f32 = 10.0;
const EXAMPLE_AIR_ACCELERATION: f32 = 3.0;
const EXAMPLE_FRICTION: f32 = 12.0;
const EXAMPLE_WALKABLE_ANGLE: f32 = PI / 4.0;
const EXAMPLE_JUMP_IMPULSE: f32 = 6.0;
const EXAMPLE_GRAVITY: f32 = 16.0; // realistic earth gravity tend to feel wrong for games
const EXAMPLE_CHARACTER_HEIGHT: f32 = 1.7;
const EXAMPLE_CHARACTER_RADIUS: f32 = 0.35;
const EXAMPLE_GROUND_CHECK_DISTANCE: f32 = 0.2;

pub struct KCCPlugin;

impl Plugin for KCCPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, movement);
    }
}

// Marker component used to freeze player movement when the main camera is in fly-mode.
// This shouldn't be strictly necessary if we figure out how to properly layer InputContexts.
#[derive(Component)]
pub struct Frozen;


#[derive(Component)]
#[require(
    RigidBody = RigidBody::Kinematic,
    Collider = Cylinder::new(EXAMPLE_CHARACTER_RADIUS, EXAMPLE_CHARACTER_HEIGHT),
)]

pub struct Character {
    velocity: Vec3,
    floor: Option<Dir3>,
    up: Dir3,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            floor: None,
            up: Dir3::Y,
        }
    }
}

fn movement(
    mut q_kcc: Query<
        (
            Entity,
            &Actions<DefaultContext>,
            &mut Transform,
            &mut Character,
            &Collider,
            &CollisionLayers,
        ),
        Without<Frozen>,
    >,
    main_camera: Single<&Transform, (With<MainCamera>, Without<Character>)>,
    sensors: Query<Entity, With<Sensor>>,
    time: Res<Time>,
    spatial_query: SpatialQuery,
) {
    let main_camera_transform = main_camera.into_inner();

    for (entity, actions, mut transform, mut character, collider, layers) in &mut q_kcc {
        let config = MoveAndSlideConfig::default();

        let mut jumped = false;
        let action_state = actions.action::<Jump>().state();
        if action_state == ActionState::Fired || action_state == ActionState::Ongoing {
            if character.floor.is_some() {
                character.velocity.y = EXAMPLE_JUMP_IMPULSE;
                character.floor = None;
                jumped = true;
            }
        }

        let input_vec = actions.action::<input::Move>().value().as_axis2d();
        
        // Get camera's forward and right vectors (ignoring Y component)
        let forward = main_camera_transform.forward();
        let right = main_camera_transform.right();
        let forward = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        let right = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();
        
        // Combine input with camera vectors
        let direction = (forward * input_vec.y + right * input_vec.x).normalize_or_zero();
        
        let max_acceleration = match character.floor {
            Some(_floor_normal) => {
                character.velocity = apply_friction(
                    character.velocity,  
                    character.velocity.length(), 
                    EXAMPLE_FRICTION, 
                    time.delta_secs()
                );
                EXAMPLE_FLOOR_ACCELERATION
            }
            None => {
                // Apply gravity when not grounded
                let gravity = character.up * -EXAMPLE_GRAVITY * time.delta_secs();
                character.velocity += gravity;

                EXAMPLE_AIR_ACCELERATION
            }
        };

        // accelerate in the movement direction
        let current_speed = character.velocity.dot(direction);
        character.velocity += accelerate(
            direction,
            EXAMPLE_MOVEMENT_SPEED,
            current_speed,
            max_acceleration,
            time.delta_secs(),
        );

        let rotation = transform.rotation;

        // Filter out the character entity as well as any entities not in the character's collision filter
        let mut filter = SpatialQueryFilter::default()
            .with_excluded_entities([entity])
            .with_mask(layers.filters);

        // Also filter out sensor entities
        filter.excluded_entities.extend(sensors);

        let up = character.up;

        // Check if the floor is walkable
        let is_walkable = |hit: ShapeHitData| {
            let slope_angle = up.angle_between(hit.normal1);
            slope_angle < EXAMPLE_WALKABLE_ANGLE
        };

        let mut floor: Option<Dir3> = None;

        if let Some(move_and_slide_result) = move_and_slide(
            &spatial_query,
            collider,
            transform.translation,
            character.velocity,
            rotation,
            config,
            &filter,
            time.delta_secs(),
            |hit| {
                if is_walkable(hit.raw_hit) {
                    floor = Some(Dir3::new(hit.raw_hit.normal1).unwrap_or(Dir3::Y));
                }
            },
        ) {
            transform.translation = move_and_slide_result.new_translation;
            character.velocity = move_and_slide_result.new_velocity;
        }

        if !jumped {
            let ground_collider = Collider::cylinder(
                EXAMPLE_CHARACTER_RADIUS,
                EXAMPLE_CHARACTER_HEIGHT
            );
            if let Some((_safe_movement_distance, hit)) = character_sweep(
                &ground_collider,
                config.epsilon,
                transform.translation,
                -character.up,
                EXAMPLE_GROUND_CHECK_DISTANCE,
                rotation,
                &spatial_query,
                &filter,
            ) {
                if is_walkable(hit) {
                    floor = Some(Dir3::new(hit.normal1).unwrap_or(Dir3::Y));
                }
            }
        }

        character.floor = floor;
    }
}

/// This is a simple example inspired by Quake 3, users are expected to bring their own logic for acceleration.
fn accelerate(
    wish_direction: Vec3,
    wish_speed: f32,
    current_speed: f32,
    accel: f32,
    delta_seconds: f32,
) -> Vec3 {
    let add_speed = wish_speed - current_speed;

    if add_speed <= 0.0 {
        return Vec3::ZERO;
    }

    let mut accel_speed = accel * delta_seconds * wish_speed;
    if accel_speed > add_speed {
        accel_speed = add_speed;
    }

    wish_direction * accel_speed
}

// Also from Quake 3
fn apply_friction(velocity: Vec3, current_speed: f32, drag: f32, delta_seconds: f32) -> Vec3 {
    let mut new_speed;
    let mut drop = 0.0;

    drop += current_speed * drag * delta_seconds;

    new_speed = current_speed - drop;
    if new_speed < 0.0 {
        new_speed = 0.0;
    }

    if new_speed != 0.0 {
        new_speed /= current_speed;
    }

    velocity * new_speed
}