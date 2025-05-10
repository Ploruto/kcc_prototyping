use std::f32::consts::PI;

use avian3d::{
    prelude::{
        Collider, CollisionLayers, PhysicsSet, RigidBody, Sensor, SpatialQuery, SpatialQueryFilter,
    },
    sync::PreviousGlobalTransform,
};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{ActionState, Actions};

use crate::{
    camera::MainCamera,
    character::*,
    input::{self, DefaultContext, Jump},
    move_and_slide::*,
};

// @todo: we should probably move all of this into an example file, then make the project a lib instead of a bin.

pub struct KCCPlugin;

impl Plugin for KCCPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPreUpdate, update_character_filter);
        app.add_systems(
            FixedUpdate,
            (movement, platform_movement.after(PhysicsSet::Sync)),
        );
        app.add_systems(Update, jump_input);
    }
}

/// Cache the [`SpatialQueryFilter`] of the character to avoid re-allocating the excluded entities map every time it's used.
///
/// This has to be a seperate component because otherwise the `character` cannot be mutated during a `move_and_slide` loop.
#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
struct CharacterFilter(SpatialQueryFilter);

fn update_character_filter(
    mut query: Query<(Entity, &mut CharacterFilter, &CollisionLayers)>,
    sensors: Query<Entity, With<Sensor>>,
) {
    for (entity, mut filter, collidion_layers) in &mut query {
        // Filter out any entities that's not in the character's collision filter
        filter.0.mask = collidion_layers.filters.into();

        // Filter out all sensor entities along with the character entity
        filter.0.excluded_entities.clear();
        filter
            .0
            .excluded_entities
            .extend(sensors.iter().chain([entity]));
    }
}

#[derive(Component)]
#[require(
    RigidBody = RigidBody::Kinematic,
    Collider = Capsule3d::new(EXAMPLE_CHARACTER_RADIUS, EXAMPLE_CHARACTER_CAPSULE_LENGTH),
    CharacterFilter,
)]
pub struct Character {
    velocity: Vec3,
    ground: Option<Ground>,
    previous_ground: Option<Ground>,
    up: Dir3,
    config: MoveAndSlideConfig,
}

impl Character {
    /// Launch the character, clearing the grounded state if launched away from the `ground` normal.
    pub fn launch(&mut self, impulse: Vec3) {
        if let Some(ground) = self.ground {
            // Clear grounded if launched away from the ground
            if ground.normal.dot(impulse) > 0.0 {
                self.ground = None;
            }
        }

        self.velocity += impulse
    }

    /// Launch the character on the `up` axis, overriding the downward velocity.
    pub fn jump(&mut self, impulse: f32) {
        // Override downward velocity
        let down = self.velocity.dot(*self.up).min(0.0);
        self.launch(self.up * impulse + self.up * -down);
    }

    /// Returns `true` if the character is standing on the ground.
    pub fn grounded(&self) -> bool {
        self.ground.is_some()
    }
}

impl Default for Character {
    fn default() -> Self {
        Self {
            velocity: Vec3::ZERO,
            ground: None,
            previous_ground: None,
            up: Dir3::Y,
            config: MoveAndSlideConfig::default(),
        }
    }
}

// Marker component used to freeze player movement when the main camera is in fly-mode.
// This shouldn't be strictly necessary if we figure out how to properly layer InputContexts.
#[derive(Component)]
pub struct Frozen;

fn jump_input(mut query: Query<(&mut Character, &Actions<DefaultContext>)>) {
    for (mut character, actions) in &mut query {
        if character.grounded() && actions.action::<Jump>().state() == ActionState::Fired {
            character.jump(EXAMPLE_JUMP_IMPULSE);
        }
    }
}

fn platform_movement(
    spatial_query: SpatialQuery,
    mut query: Query<(&mut Transform, &mut Character, &Collider, &CharacterFilter)>,
    platforms: Query<(&GlobalTransform, &PreviousGlobalTransform)>,
    time: Res<Time>,
) {
    for (mut transform, mut character, collider, filter) in &mut query {
        let platform_motion = |entity| {
            platforms.get(entity).map_or(
                Vec3::ZERO,
                |(platform_transform, prev_platform_transform)| {
                    motion_on_point(
                        transform.translation,
                        platform_transform,
                        prev_platform_transform,
                    )
                },
            )
        };

        match (character.ground, character.previous_ground) {
            // Currently on the platform, follow it's movement
            (Some(ground), ..) => {
                let platform_motion = platform_motion(ground.entity);

                // Sweep in the platform movement direction to avoid passing through walls
                if let Ok((direction, max_distance)) = Dir3::new_and_length(platform_motion) {
                    let safe_distance = sweep_check(
                        collider,
                        character.config.epsilon,
                        transform.translation,
                        direction,
                        max_distance,
                        transform.rotation,
                        &spatial_query,
                        &filter.0,
                    )
                    .map(|(d, _)| d)
                    .unwrap_or(max_distance);

                    transform.translation += direction * safe_distance;
                };
            }
            // Left the platform, inherit the platform velocity
            (None, Some(previous_ground)) => {
                let platform_velocity = platform_motion(previous_ground.entity) / time.delta_secs();
                character.velocity += platform_velocity;
            }
            _ => {}
        }

        character.previous_ground = character.ground;
    }
}

fn movement(
    mut q_kcc: Query<
        (
            &Actions<DefaultContext>,
            &mut Transform,
            &mut Character,
            &Collider,
            &CharacterFilter,
            Has<Sensor>,
        ),
        Without<Frozen>,
    >,
    main_camera: Single<&Transform, (With<MainCamera>, Without<Character>)>,
    time: Res<Time>,
    spatial_query: SpatialQuery,
) {
    let main_camera_transform = main_camera.into_inner();
    for (actions, mut transform, mut character, collider, filter, has_sensor) in &mut q_kcc {
        // Get the raw 2D input vector
        let input_vec = actions.action::<input::Move>().value().as_axis2d();

        // Extract just the yaw from the camera rotation
        let camera_yaw = main_camera_transform.rotation.to_euler(EulerRot::YXZ).0;
        let yaw_rotation = Quat::from_rotation_y(camera_yaw);

        // Rotate the movement direction vector by only the camera's yaw
        let direction = yaw_rotation * Vec3::new(input_vec.x, 0.0, -input_vec.y);

        let max_acceleration = match character.ground {
            Some(_) => {
                let friction = friction(character.velocity, EXAMPLE_FRICTION, time.delta_secs());
                character.velocity += friction;

                EXAMPLE_GROUND_ACCELERATION
            }
            None => {
                // Apply gravity when not grounded
                let gravity = character.up * -EXAMPLE_GRAVITY * time.delta_secs();
                character.velocity += gravity;

                EXAMPLE_AIR_ACCELERATION
            }
        };

        // accelerate in the movement direction
        let mut move_accel = acceleration(
            character.velocity,
            direction,
            max_acceleration,
            EXAMPLE_MOVEMENT_SPEED,
            time.delta_secs(),
        );

        // We can skip everything if the character has a sensor component
        if has_sensor {
            character.velocity += move_accel;
            transform.translation += character.velocity * time.delta_secs();

            continue;
        }

        // We need to store the new ground for the ground check to work properly
        let mut new_ground = None;

        if let Some(ground) = character.ground {
            // Project acceleration on the ground plane
            move_accel = project_motion_on_ground(move_accel, *ground.normal, character.up);
        }

        // Sweep in the movement direction to find a plane to project acceleration on
        // This is a seperate step because trying to do this in the `move_and_slide` callback
        // results in "sticking" to the wall rather than sliding down at the expected rate
        if let Ok((direction, max_distance)) = Dir3::new_and_length(move_accel * time.delta_secs())
        {
            if let Some((safe_distance, hit)) = sweep_check(
                collider,
                character.config.epsilon,
                transform.translation,
                direction,
                max_distance,
                transform.rotation,
                &spatial_query,
                &filter.0,
            ) {
                // Move to the hit point
                transform.translation += direction * safe_distance;

                if let Some(ground) = Ground::new_if_walkable(
                    hit.entity,
                    hit.normal1,
                    character.up,
                    EXAMPLE_WALKABLE_ANGLE,
                ) {
                    new_ground = Some(ground);

                    // If the ground is walkable, project motion on ground plane
                    move_accel = project_motion_on_ground(move_accel, hit.normal1, character.up);
                } else if let Some(step_result) = try_step_up_on_hit(
                    collider,
                    transform.translation,
                    transform.rotation,
                    character.up,
                    hit.normal1,
                    direction,
                    max_distance - safe_distance,
                    character.config.epsilon,
                    &spatial_query,
                    &filter.0,
                    time.delta_secs(),
                ) {
                    new_ground = Some(step_result.ground);

                    // Step up
                    transform.translation = step_result.translation;
                } else {
                    // If the ground is not walkable, project motion on wall plane
                    move_accel = project_motion_on_wall(move_accel, hit.normal1, character.up);
                }
            }
        }

        character.velocity += move_accel;

        let move_result = move_and_slide(
            &spatial_query,
            &collider,
            transform.translation,
            character.velocity,
            transform.rotation,
            character.config,
            &filter.0,
            time.delta_secs(),
            |hit| {
                if let Some(ground) = Ground::new_if_walkable(
                    hit.hit_data.entity,
                    hit.hit_data.normal1,
                    character.up,
                    EXAMPLE_WALKABLE_ANGLE,
                ) {
                    new_ground = Some(ground);

                    // Avoid sliding down slopes when just landing
                    if !character.grounded() {
                        *hit.velocity = project_motion_on_ground(
                            *hit.velocity,
                            hit.hit_data.normal1,
                            character.up,
                        );

                        character.velocity = project_motion_on_ground(
                            character.velocity,
                            hit.hit_data.normal1,
                            character.up,
                        );
                    }

                    return true;
                }

                let grounded = character.grounded() || new_ground.is_some();

                // In order to try step up we need to be grounded and hitting a "wall".
                if grounded {
                    if let Some(step_result) = try_step_up_on_hit(
                        collider,
                        *hit.translation,
                        transform.rotation,
                        character.up,
                        hit.hit_data.normal1,
                        hit.direction,
                        hit.remaining_motion,
                        character.config.epsilon,
                        &spatial_query,
                        &filter.0,
                        time.delta_secs(),
                    ) {
                        new_ground = Some(step_result.ground);

                        // Subtract the stepped distance from remaining time to avoid moving further
                        *hit.remaining_time =
                            (*hit.remaining_time - step_result.move_time).max(0.0);

                        // We need to override the translation here because the we stepped up
                        *hit.translation = step_result.translation;

                        // Successfully stepped, don't slide this iteration
                        return false;
                    }
                }

                // Slide vleocity along walls
                match grounded {
                    // Avoid sliding up walls when grounded
                    true => {
                        character.velocity = project_motion_on_wall(
                            character.velocity,
                            hit.hit_data.normal1,
                            character.up,
                        );

                        *hit.velocity = project_motion_on_wall(
                            *hit.velocity,
                            hit.hit_data.normal1,
                            character.up,
                        )
                    }
                    false => {
                        character.velocity = character.velocity.reject_from(hit.hit_data.normal1)
                    }
                };

                true
            },
        );

        transform.translation = move_result.new_translation;

        // Check if the previous ground is still there and snap to it
        if character.grounded() {
            if let Some((safe_distance, ground)) = ground_check(
                &collider,
                character.config,
                transform.translation,
                character.up,
                transform.rotation,
                &spatial_query,
                &filter.0,
                EXAMPLE_GROUND_CHECK_DISTANCE,
                EXAMPLE_WALKABLE_ANGLE,
            ) {
                transform.translation -= character.up * safe_distance;
                new_ground = Some(ground);
            }
        }

        // Update the ground
        character.ground = new_ground;
    }
}

struct StepUpResult {
    translation: Vec3,
    move_time: f32,
    ground: Ground,
}

fn try_step_up_on_hit(
    collider: &Collider,
    translation: Vec3,
    rotation: Quat,
    up: Dir3,
    hit_normal: Vec3,
    direction: Dir3,
    mut step_forward: f32,
    epsilon: f32,
    spatial_query: &SpatialQuery,
    filter: &SpatialQueryFilter,
    delta_time: f32,
) -> Option<StepUpResult> {
    let horizontal_normal = hit_normal.reject_from_normalized(*up).normalize_or_zero();

    // This is necessary for capsule colliders since the normal angle changes depending on
    // how far out on a ledge the character is standing
    let a = 1.0 - EXAMPLE_WALKABLE_ANGLE.cos();
    let min_inward_distance = EXAMPLE_CHARACTER_RADIUS * a;

    // Step into the hit normal alil bit, this helps with the capsule collider.
    // Cylinders don't need this since they have a flat bottom.
    let inward = min_inward_distance + epsilon * PI;

    // Step a lil bit less forward to account for stepping into the hit normal
    step_forward = (step_forward - inward).max(0.0);

    let step_motion = direction * step_forward - horizontal_normal * inward;

    let Some((step_translation, hit)) = try_climb_step(
        spatial_query,
        &collider,
        translation,
        step_motion,
        rotation,
        up,
        EXAMPLE_STEP_HEIGHT + EXAMPLE_GROUND_CHECK_DISTANCE,
        epsilon,
        &filter,
    ) else {
        // Can't stand here, slide instead
        return None;
    };

    let ground = Ground::new_if_walkable(
        hit.entity,
        hit.normal1,
        up,
        // Subtract a small amount from walkable angle to make sure we can't step
        // on surfaces that are nearly excactly the walkable angle of the character
        EXAMPLE_WALKABLE_ANGLE - 1e-4,
    )?;

    if !is_walkable(hit.normal1, up, EXAMPLE_WALKABLE_ANGLE - 1e-4) {
        return None;
    }

    // Subtract the stepped distance from remaining time to avoid moving further
    let move_time = (step_forward + inward) * delta_time;

    Some(StepUpResult {
        translation: step_translation,
        move_time,
        ground,
    })
}

/// This is a simple example inspired by Quake, users are expected to bring their own logic for acceleration.
#[must_use]
fn acceleration(
    velocity: Vec3,
    direction: impl TryInto<Dir3>,
    max_acceleration: f32,
    target_speed: f32,
    delta: f32,
) -> Vec3 {
    let Ok(direction) = direction.try_into() else {
        return Vec3::ZERO;
    };

    // Current speed in the desired direction.
    let current_speed = velocity.dot(*direction);

    // No acceleration is needed if current speed exceeds target.
    if current_speed >= target_speed {
        return Vec3::ZERO;
    }

    // Clamp to avoid acceleration past the target speed.
    let accel_speed = f32::min(target_speed - current_speed, max_acceleration * delta);

    direction * accel_speed
}

/// Constant acceleration in the opposite direction of velocity.
#[must_use]
pub fn friction(velocity: Vec3, friction: f32, delta: f32) -> Vec3 {
    let speed_sq = velocity.length_squared();

    if speed_sq < 1e-4 {
        return Vec3::ZERO;
    }

    let factor = f32::exp(-friction / speed_sq.sqrt() * delta);

    -velocity * (1.0 - factor)
}
