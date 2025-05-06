use std::{f32::consts::PI, time::Duration};

use avian3d::prelude::{
    Collider, CollisionLayers, RigidBody, Sensor, ShapeHitData, SpatialQuery, SpatialQueryFilter,
};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{ActionState, Actions};
use kcc_prototype::move_and_slide::{MoveAndSlideConfig, character_sweep, move_and_slide};

use crate::input::{DefaultContext, Jump, Move};

const EXAMPLE_MOVEMENT_SPEED: f32 = 8.0;
const EXAMPLE_FLOOR_ACCELERATION: f32 = 100.0;
const EXAMPLE_AIR_ACCELERATION: f32 = 40.0;
const EXAMPLE_FRICTION: f32 = 60.0;
const EXAMPLE_WALKABLE_ANGLE: f32 = PI / 4.0;
const EXAMPLE_JUMP_IMPULSE: f32 = 6.0;
const EXAMPLE_GRAVITY: f32 = 20.0; // realistic earth gravity tend to feel wrong for games
const EXAMPLE_CHARACTER_HEIGHT: f32 = 1.0;
const EXAMPLE_CHARACTER_RADIUS: f32 = 0.35;
const EXAMPLE_GROUND_MARGIN: f32 = 0.1;

pub struct KCCPlugin;

impl Plugin for KCCPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, movement);
    }
}

#[derive(Component)]
#[require(
    RigidBody = RigidBody::Kinematic,
    Collider = Capsule3d::new(EXAMPLE_CHARACTER_RADIUS, EXAMPLE_CHARACTER_HEIGHT),
)]
pub struct Character {
    velocity: Vec3,
    floor: Option<Dir3>,
    up: Dir3,
    jump_timer: Duration,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            floor: None,
            up: Dir3::Y,
            jump_timer: Duration::ZERO,
        }
    }
}

fn movement(
    mut q_kcc: Query<(
        Entity,
        &mut Transform,
        &mut Character,
        &Collider,
        &CollisionLayers,
    )>,
    q_input: Single<&Actions<DefaultContext>>,
    sensors: Query<Entity, With<Sensor>>,
    time: Res<Time>,
    spatial_query: SpatialQuery,
) {
    for (entity, mut transform, mut character, collider, layers) in &mut q_kcc {

        let mut jumped = false;
        let action_state = q_input.action::<Jump>().state();
        if action_state == ActionState::Fired || action_state == ActionState::Ongoing {
            if character.floor.is_some() {
                let impulse = character.up * EXAMPLE_JUMP_IMPULSE;
                character.velocity += impulse;
                character.floor = None;
                character.jump_timer = Duration::from_millis(20);
                jumped = true;
            }
        }

        // Get the raw 2D input vector
        let input_vec = q_input.action::<Move>().value().as_axis2d();

        // Rotate the movement direction vector by the camera's yaw
        let mut direction =
            (transform.rotation * Vec3::new(input_vec.x, 0.0, -input_vec.y)).normalize_or_zero();

        let max_acceleration = match character.floor {
            Some(floor_normal) => {
                apply_friction(&mut character.velocity, EXAMPLE_FRICTION, time.delta_secs());

                // Make sure velocity is never towards the floor since this makes the jump height inconsistent
                // let downward_vel = character.velocity.dot(*floor_normal).min(0.0);
                // character.velocity -= floor_normal * downward_vel;

                // Project input direction on the floor normal to allow walking down slopes
                // TODO: this is wrong, walking diagonally up/down slopes will be slightly off direction wise,
                // even more so for steep slopes.
                // direction = direction
                //     .reject_from_normalized(*floor_normal)
                //     .normalize_or_zero();

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
        accelerate(
            &mut character.velocity,
            direction,
            max_acceleration,
            EXAMPLE_MOVEMENT_SPEED,
            time.delta_secs(),
        );

        let rotation = transform.rotation;

        // Filter out the character entity as well as any entities not in the character's collision filter
        let mut filter = SpatialQueryFilter::default()
            .with_excluded_entities([entity])
            .with_mask(layers.filters);

        // Also filter out sensor entities
        filter.excluded_entities.extend(sensors);

        let config = MoveAndSlideConfig::default();

        let up = character.up;

        // Check if the floor is walkable
        let is_walkable = |hit: ShapeHitData| {
            let slope_angle = up.angle_between(hit.normal1);
            slope_angle < EXAMPLE_WALKABLE_ANGLE
        };

        let mut floor = None;
        move_and_slide(
            &spatial_query,
            collider,
            &mut transform.translation,
            &mut character.velocity,
            rotation,
            config,
            &filter,
            time.delta_secs(),
            |hit| {
                if is_walkable(hit.raw_hit)
                    // Must also be moving downwards (is this correct?)
                    && hit.out_velocity.y <= 0.0
                {
                    floor = Some(Dir3::new(hit.raw_hit.normal1).unwrap_or(Dir3::Y));
                }
            },
        );

        // Check for ground if we're not jumping
        // And only check ground if we're moving downwards
        if !jumped && character.velocity.y <= 0.0 {
            let ground_collider = Collider::cylinder(
                EXAMPLE_CHARACTER_RADIUS - 0.01, 
                EXAMPLE_CHARACTER_HEIGHT + EXAMPLE_CHARACTER_RADIUS * 2.0
            );
            if let Some((safe_movement_distance, hit)) = character_sweep(
                &ground_collider,
                config.epsilon,
                transform.translation - Vec3::new(0.0, EXAMPLE_GROUND_MARGIN, 0.0),
                -character.up,
                EXAMPLE_GROUND_MARGIN + config.epsilon,
                rotation,
                &spatial_query,
                &filter,
            ) {
                if is_walkable(hit) {
                    transform.translation.y -= safe_movement_distance;
                    //transform.translation.y = hit.point1.y + (EXAMPLE_CHARACTER_HEIGHT * 0.5 + EXAMPLE_CHARACTER_RADIUS * 0.5) + EXAMPLE_GROUND_MARGIN;
                    floor = Some(Dir3::new(hit.normal1).unwrap_or(Dir3::Y));
                }
            }
        }

        // Update jump timer
        if !character.jump_timer.is_zero() {
            character.jump_timer = character.jump_timer.saturating_sub(time.delta());
        }

        character.floor = floor;
    }
}

/// This is a simple example inspired by Quake, users are expected to bring their own logic for acceleration.
fn accelerate(
    velocity: &mut Vec3,
    direction: impl TryInto<Dir3>,
    max_acceleration: f32,
    target_speed: f32,
    delta: f32,
) {
    let Ok(direction) = direction.try_into() else {
        return;
    };

    // Current speed in the desired direction.
    let current_speed = velocity.dot(*direction);

    // No acceleration is needed if current speed exceeds target.
    if current_speed >= target_speed {
        return;
    }

    // Clamp to avoid acceleration past the target speed.
    let accel_speed = f32::min(target_speed - current_speed, max_acceleration * delta);

    *velocity += accel_speed * direction;
}

/// Constant acceleration in the opposite direction of velocity.
pub fn apply_friction(velocity: &mut Vec3, friction: f32, delta: f32) {
    let speed_sq = velocity.length_squared();

    if speed_sq < 1e-4 {
        return;
    }

    let factor = f32::exp(-friction / speed_sq.sqrt() * delta);

    *velocity *= factor;
}
