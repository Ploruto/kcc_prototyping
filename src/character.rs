use std::f32::consts::PI;

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::move_and_slide::*;

pub const EXAMPLE_CHARACTER_RADIUS: f32 = 0.35;
pub const EXAMPLE_CHARACTER_CAPSULE_LENGTH: f32 = 1.0;
pub const EXAMPLE_MOVEMENT_SPEED: f32 = 8.0;
pub const EXAMPLE_GROUND_ACCELERATION: f32 = 100.0;
pub const EXAMPLE_AIR_ACCELERATION: f32 = 40.0;
pub const EXAMPLE_FRICTION: f32 = 60.0;
pub const EXAMPLE_WALKABLE_ANGLE: f32 = PI / 4.0;
pub const EXAMPLE_JUMP_IMPULSE: f32 = 6.0;
pub const EXAMPLE_GRAVITY: f32 = 20.0; // realistic earth gravity tend to feel wrong for games
pub const EXAMPLE_STEP_HEIGHT: f32 = 0.25;
pub const EXAMPLE_GROUND_CHECK_DISTANCE: f32 = 0.1;

// @todo: probably want to improve the ergonomics of these
// functions by accepting a struct instead of a bunch of arguments,
// that way each can be commented and we can also provide sane defaults, and ordering doesn't matter.

/// Represents the ground a character is currently standing on.
#[derive(Reflect, Debug, PartialEq, Clone, Copy)]
pub struct Ground {
    pub entity: Entity,
    pub normal: Dir3,
}

impl Ground {
    /// Construct a new [`Ground`] if the `normal` is walkable with the given `walkable_angle` and `up` direction.
    pub fn new_if_walkable(
        entity: Entity,
        normal: impl TryInto<Dir3>,
        up: Dir3,
        walkable_angle: f32,
    ) -> Option<Self> {
        let normal = normal.try_into().ok()?;

        if !is_walkable(*normal, up, walkable_angle) {
            return None;
        }

        Some(Self { entity, normal })
    }

    /// Returns `true` if the [`Ground`] is walkable with the given `walkable_angle` and `up` direction.
    pub fn is_walkable(&self, up: Dir3, walkable_angle: f32) -> bool {
        is_walkable(*self.normal, up, walkable_angle)
    }
}

/// Checks if a surface is walkable based on its slope angle and the up direction.
pub fn is_walkable(normal: Vec3, up: Dir3, walkable_angle: f32) -> bool {
    let slope_angle = up.angle_between(normal);
    slope_angle < walkable_angle
}

/// Find and climb steps in the movement direction.
///
/// # Prerequisites
/// Before calling this function, it is recommended that:
/// - The character is grounded
/// - The character is hitting a wall
///
/// # How This Works
///
/// The step climbing process follows these steps:
///
/// 1. **Step Height Check**
///    - Perform a shape-cast from the character's position
///    - Cast is done above ground level by the step height
///    - Cast is in the direction of movement
///    - If nothing is hit, the wall is "step-uppable"
///
/// 2. **Height Discovery**
///    - If the above shape-cast hits nothing:
///      - Perform a downward shape-cast from the step up height
///        at the intended step up position based on the remaining motion.
///     - This determines the actual height of the step
///
/// 3. **Step Execution**
///    - Teleport up by the discovered height
///    - Move forward with remaining motion
pub fn try_climb_step(
    spatial_query: &SpatialQuery,
    collider: &Collider,
    translation: Vec3,
    motion: Vec3,
    rotation: Quat,
    up: Dir3,
    step_up_height: f32,
    epsilon: f32,
    filter: &SpatialQueryFilter,
) -> Option<(Vec3, ShapeHitData)> {
    let step_up_pos = translation + up * step_up_height;

    let horizontal_motion = motion.reject_from_normalized(Vec3::Y);

    // Only step up if horizontal motion is non zero
    if let Ok(direction) = Dir3::new(horizontal_motion) {
        // Step up and sweep forward
        let None = spatial_query.cast_shape(
            collider,
            step_up_pos,
            rotation,
            direction,
            &ShapeCastConfig {
                max_distance: horizontal_motion.length(),
                ..Default::default()
            },
            filter,
        ) else {
            // We hit something at the step height, so we can't climb it.
            return None;
        };
    }

    let step_down_pos = step_up_pos + horizontal_motion;

    // Step forward and sweep down
    let (safe_distance, step_down_hit) = sweep_check(
        collider,
        epsilon,
        step_down_pos,
        -up,
        step_up_height,
        rotation,
        spatial_query,
        filter,
    )?;

    let new_translation = step_down_pos - up * safe_distance;

    Some((new_translation, step_down_hit))
}

/// Sweep in the opposite direction of `up` and return the [`Ground`] if it's walkable.
pub fn ground_check(
    collider: &Collider,
    config: MoveAndSlideConfig,
    translation: Vec3,
    up: Dir3,
    rotation: Quat,
    spatial_query: &SpatialQuery,
    filter: &SpatialQueryFilter,
    floor_check_distance: f32,
    walkable_angle: f32,
) -> Option<(f32, Ground)> {
    let (safe_distance, hit) = sweep_check(
        collider,
        config.epsilon,
        translation,
        -up,
        floor_check_distance,
        rotation,
        spatial_query,
        filter,
    )?;

    let ground = Ground::new_if_walkable(hit.entity, hit.normal1, up, walkable_angle)?;

    Some((safe_distance, ground))
}

/// Projects a vector on a plane normal.
///
/// The returned vector has different properties depending on whether the plane is walkable or not:
/// - **walkable**: the vector will be aligned with the plane `normal` with the horizontal direction unchanged
/// - **non-walkable**: the vector will be aligned with both the plane `normal` and `up` direction
///
/// **Panics** if the `normal` is zero, infinite or `NaN`.
#[track_caller]
pub fn project_motion(
    motion: Vec3,
    normal: impl TryInto<Dir3>,
    up: Dir3,
    walkable_angle: f32,
) -> Vec3 {
    let normal = normal
        .try_into()
        .unwrap_or_else(|_| panic!("normal must not be zero, infinite or NaN"));

    match is_walkable(*normal, up, walkable_angle) {
        true => project_motion_on_ground(motion, normal, up),
        false => project_motion_on_wall(motion, normal, up),
    }
}

/// Projects a vector on a walkable plane.
///
/// The returned vector will be aligned with the plane `normal` with the horizontal direction unchanged.
///
/// **Panics** if the `normal` is zero, infinite or `NaN`.
#[track_caller]
pub fn project_motion_on_ground(motion: Vec3, normal: impl TryInto<Dir3>, up: Dir3) -> Vec3 {
    let normal = normal
        .try_into()
        .unwrap_or_else(|_| panic!("normal must not be zero, infinite or NaN"));

    // Split input vector into vertical and horizontal components
    let mut vertical = motion.project_onto(*up);
    let mut horizontal = motion - vertical;

    // Remove downward velocity
    if vertical.dot(*normal) < 0.0 {
        vertical = Vec3::ZERO;
    }

    let Ok(tangent) = Dir3::new(horizontal.cross(*up)) else {
        return vertical; // No horizontal velocity
    };

    // Calculate the horizontal direction along the slope
    // This gives us a vector that follows the slope but maintains original horizontal intent
    let Ok(horizontal_direction) = Dir3::new(tangent.cross(*normal)) else {
        // Horizontal direction is already perpendicular with the ground normal
        return vertical + horizontal;
    };

    // Project horizontal movement onto the calculated direction
    horizontal = horizontal.project_onto_normalized(*horizontal_direction);

    vertical + horizontal
}

/// Projects a vector on a non-walkable plane.
///
/// The returned vector will be aligned with both the `normal` and `up` direction.
///
/// This ensures the character is not able to slide up slopes that are not walkable.
///
/// **Panics** if the `normal` is zero, infinite or `NaN`.
#[track_caller]
pub fn project_motion_on_wall(motion: Vec3, normal: impl TryInto<Dir3>, up: Dir3) -> Vec3 {
    let normal = normal
        .try_into()
        .unwrap_or_else(|_| panic!("normal must not be zero, infinite or NaN"));

    // Split input vector into vertical and horizontal components
    let mut vertical = motion.project_onto(*up);
    let mut horizontal = motion - vertical;

    // Project the vertical part on the plane normal
    vertical = vertical.reject_from_normalized(*normal);

    let Ok(tangent) = Dir3::new(normal.cross(*up)) else {
        return vertical + horizontal; // This is not a wall
    };

    // Project horizontal movement along the wall tangent
    horizontal = horizontal.project_onto_normalized(*tangent);

    vertical + horizontal
}

/// Transform a point that's relative to a previous transform to a new transform's space.
///
/// Returns the new world-space position of the point.
pub fn transform_moving_point(
    point: Vec3,
    current_transform: &GlobalTransform,
    previous_transform: &GlobalTransform,
) -> Vec3 {
    // Convert world-space point to local space of the previous transform
    let prev_transform_inverse = previous_transform.compute_matrix().inverse();
    let point_in_local_space = prev_transform_inverse.transform_point3(point);

    // Convert local space point back to world space using the current transform
    current_transform
        .compute_matrix()
        .transform_point3(point_in_local_space)
}

/// Get the motion of a moving transform at the given `point`.
pub fn motion_on_point(
    point: Vec3,
    current_transform: &GlobalTransform,
    previous_transform: &GlobalTransform,
) -> Vec3 {
    transform_moving_point(point, current_transform, previous_transform) - point
}
