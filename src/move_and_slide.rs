use avian3d::prelude::*;
use bevy::prelude::*;
const SIMILARITY_THRESHOLD: f32 = 0.999;

#[must_use]
pub fn character_sweep(
    collider: &Collider,
    epsilon: f32,
    origin: Vec3,
    direction: Dir3,
    distance: f32,
    rotation: Quat,
    spatial_query: &SpatialQuery,
    filter: &SpatialQueryFilter,
) -> Option<(f32, ShapeHitData)> {
    let hit = spatial_query.cast_shape(
        collider,
        origin,
        rotation,
        direction,
        &ShapeCastConfig {
            max_distance: distance,
            target_distance: 0.0,
            ignore_origin_penetration: true,
            compute_contact_on_penetration: true,
            ..Default::default()
        },
        filter,
    )?;

    // How far is safe to translate by
    let safe_distance = (hit.distance - epsilon).max(0.0);
    Some((safe_distance, hit))
}

////// EXAMPLE MOVEMENT /////////////
#[derive(Clone, Copy)]
pub struct MoveAndSlideConfig {
    pub max_iterations: usize,
    pub skin_width: f32,
    pub epsilon: f32,
}

impl Default for MoveAndSlideConfig {
    fn default() -> Self {
        Self {
            max_iterations: 4,
            skin_width: 0.01,
            epsilon: 0.0001,
        }
    }
}

pub struct MoveAndSlideHit<'a> {
    pub raw_hit: ShapeHitData,
    pub remaining_time: f32,
    pub safe_movement_distance: f32,
    pub out_velocity: &'a mut Vec3,
    pub out_translation: &'a mut Vec3,
}

#[allow(clippy::too_many_arguments)]
pub fn move_and_slide(
    spatial_query: &SpatialQuery,
    collider: &Collider,
    translation: &mut Vec3,
    velocity: &mut Vec3,
    rotation: Quat,
    config: MoveAndSlideConfig,
    filter: &SpatialQueryFilter,
    delta_time: f32,
    mut on_hit: impl FnMut(&mut MoveAndSlideHit),
) {
    let Ok(original_direction) = Dir3::new(*velocity) else {
        return;
    };

    let mut remaining_time = delta_time;
    let mut hits = Vec::with_capacity(config.max_iterations);
    
    for _ in 0..config.max_iterations {
        let max_distance = velocity.length() * remaining_time;
        let direction = velocity.normalize_or_zero();

        let Some((safe_movement, hit)) = character_sweep(
            collider,
            config.epsilon,
            *translation,
            Dir3::new(direction).unwrap_or(Dir3::Y),
            max_distance + config.skin_width,
            rotation,
            spatial_query,
            filter,
        ) else {
            // No collision, move the full remaining distance
            *translation += *velocity * remaining_time;
            break;
        };

        on_hit(&mut MoveAndSlideHit {
            raw_hit: hit,
            remaining_time,
            safe_movement_distance: safe_movement,
            out_velocity: velocity,
            out_translation: translation,
        });

        hits.push(hit.normal1);

        // Project velocity and remaining motion onto the surface plane
        *velocity = solve_collision_planes(*velocity, &hits, *original_direction);

        // Quake2: "If velocity is against original velocity, stop early to avoid tiny oscilations in sloping corners."
        if velocity.dot(*original_direction) <= 0.0 {
            break;
        }

        // Calculate movement and remaining time
        let movement = (safe_movement - config.skin_width).max(0.0);
        let movement_ratio = if max_distance > 0.0 {
            (movement / max_distance).clamp(0.0, 1.0)
        } else {
            0.0
        };
        remaining_time *= 1.0 - movement_ratio;
        *translation += direction * movement;
    }
}

fn similar_plane(normal1: Vec3, normal2: Vec3) -> bool {
    normal1.dot(normal2) > SIMILARITY_THRESHOLD
}

fn solve_collision_planes(
    velocity: Vec3,
    hits: &[Vec3],
    original_velocity_direction: Vec3,
) -> Vec3 {
    // Early out if we have no velocity or no hits
    if velocity.length_squared() <= 0.0 || original_velocity_direction.length_squared() <= 0.0 {
        return Vec3::ZERO;
    }

    if hits.is_empty() {
        return velocity;
    }

    // Do our initial rejection to calculate the sliding velocity.
    let first_hit_normal = hits[hits.len() - 1];
    if velocity.dot(first_hit_normal) >= 0.0 {
        return velocity;
    }

    let initial_velocity = velocity.reject_from_normalized(first_hit_normal);

    // Join the original velocity direction as an additional constraining plane
    let original_velocity_normal = original_velocity_direction.normalize_or_zero();
    let all_hits: Vec<Vec3> = std::iter::once(original_velocity_normal)
        .chain(hits.iter().cloned())
        .collect();

    // We should filter out any normals that are similar to the existing constraints
    let mut filtered_hits = all_hits.iter().filter(|&n| {
        !similar_plane(first_hit_normal, *n) && !similar_plane(original_velocity_normal, *n)
    });

    filtered_hits.try_fold(initial_velocity, |vel, second_hit_normal| {
        let vel = vel.reject_from_normalized(*second_hit_normal);
        let vel_dir = vel.normalize_or_zero();

        // If the velocity is already parallel to the first hit normal, we can return it directly
        if similar_plane(vel_dir, first_hit_normal) {
            // If the velocity is small enough we can just assume we have no reason to move
            if vel.length_squared() <= f32::EPSILON {
                Err(vel)
            } else {
                // Otherwise we need to keep working.
                Ok(vel)
            }
        } else {
            // If we have a valid second hit normal, we can calculate the crease direction
            let crease_dir = first_hit_normal.cross(*second_hit_normal).normalize_or_zero();
            let vel_proj = vel.project_onto(crease_dir);
            let vel_proj_dir = vel_proj.normalize_or_zero();

            // Check if the velocity projection is a corner case
            // A corner case is when the velocity projection is not similar to either of the hit normals
            // but is similar to the crease direction formed by the two hit normals.
            let is_corner = all_hits.iter().any(|third_hit_normal| {
                !similar_plane(first_hit_normal, *third_hit_normal) &&
                !similar_plane(*second_hit_normal, *third_hit_normal) &&
                similar_plane(vel_proj_dir, *third_hit_normal)
            });

            // If we are in a corner case, add a small nudge away from both surfaces
            if is_corner {
                let nudge = (first_hit_normal + *second_hit_normal).normalize_or_zero() * 0.01;
                Err(vel_proj + nudge)
            } else if vel_proj.length_squared() <= f32::EPSILON {
                // Otherwise we can return the velocity if we have a small enough projection
                Err(vel_proj)
            } else {
                // Otherwise lets keep working with the projection
                Ok(vel_proj)
            }
        }
    }).unwrap_or_else(|vel| vel)
}