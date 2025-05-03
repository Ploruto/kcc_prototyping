use std::f32::consts::PI;

use avian3d::prelude::{
    Collider, CollisionLayers, RigidBody, ShapeCastConfig, SpatialQuery, SpatialQueryFilter,
};
use bevy::{log::tracing::field::debug, prelude::*};
use bevy_enhanced_input::prelude::{ActionState, Actions};

use crate::{
    DefaultCamera, KCCMarker,
    input::{DefaultContext, Jump, Move},
};

pub struct KCCPlugin;

impl Plugin for KCCPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(KCCConfig {
            max_walkable_slope_angle: 45.0,
            max_step_height: 0.5,
            ground_check_distance: 1.5,
        });
        app.add_systems(
            FixedUpdate,
            (update_grounded_and_sliding_state, movement).chain(),
        );
    }
}

#[derive(Component)]
pub struct KinematicVelocity(pub Vec3);

#[derive(Component)]
struct Grounded;

#[derive(Component)]
struct Sliding;

#[derive(Resource)]
struct KCCConfig {
    pub max_walkable_slope_angle: f32, // in degrees
    pub max_step_height: f32,
    pub ground_check_distance: f32,
}

#[derive(Bundle)]
pub struct KCCBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub kcc_marker: KCCMarker,
    pub kinematic_velocity: KinematicVelocity,
}

impl Default for KCCBundle {
    fn default() -> Self {
        Self {
            collider: Collider::capsule(0.35, 1.0),
            rigid_body: RigidBody::Kinematic,
            kcc_marker: KCCMarker,
            kinematic_velocity: KinematicVelocity(Vec3::ZERO),
        }
    }
}

const EXAMPLE_MOVEMENT_SPEED: f32 = 8.0;

fn movement(
    mut q_kcc: Query<
        (
            Entity,
            &mut Transform,
            &mut KinematicVelocity,
            &Collider,
            &CollisionLayers,
        ),
        With<KCCMarker>,
    >,
    q_input: Single<&Actions<DefaultContext>>,
    q_camera: Query<&Transform, (With<DefaultCamera>, Without<KCCMarker>)>,
    time: Res<Time>,
    spatial_query: SpatialQuery,
) {
    // get camera rotation yaw
    let Some(camera_transform) = q_camera.single().ok() else {
        warn!("No camera found!");
        return;
    };

    if q_input.action::<Jump>().state() == ActionState::Fired {
        println!("Jump action fired!");
    }

    // Get the raw 2D input vector
    let input_vec = q_input.action::<Move>().value().as_axis2d();

    let Some((entity, mut kcc_transform, mut kinematic_vel, collider, layers)) =
        q_kcc.single_mut().ok()
    else {
        warn!("No KCC found!");
        return;
    };

    // Rotate the movement direction vector by the camera's yaw
    // movement_dir = Quat::from_rotation_y(camera_yaw) * movement_dir;
    let direction = kcc_transform
        .rotation
        .mul_vec3(Vec3::new(input_vec.x, 0.0, -input_vec.y))
        .normalize_or_zero();

    let mut artifical_velocity = direction * EXAMPLE_MOVEMENT_SPEED;

    let filter = SpatialQueryFilter::default()
        .with_excluded_entities(vec![entity])
        .with_mask(layers.filters);

    let rotation = kcc_transform.rotation;

    move_and_slide(
        MoveAndSlideConfig::default(),
        collider,
        time.delta_secs(),
        &entity,
        &mut kcc_transform.translation,
        &mut artifical_velocity,
        rotation,
        &spatial_query,
        &filter,
    );
}

////// EXAMPLE MOVEMENT /////////////
pub struct MoveAndSlideConfig {
    pub max_iterations: usize,
    pub epsilon: f32,
}

impl Default for MoveAndSlideConfig {
    fn default() -> Self {
        Self {
            max_iterations: 4,
            epsilon: 0.01,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn move_and_slide(
    config: MoveAndSlideConfig,
    collider: &Collider,
    delta_time: f32,
    entity: &Entity,
    translation: &mut Vec3,
    velocity: &mut Vec3,
    rotation: Quat,
    spatial_query: &SpatialQuery,
    filter: &SpatialQueryFilter,
) {
    let mut remaining_velocity = *velocity * delta_time;

    for _ in 0..config.max_iterations {
        if let Some(hit) = spatial_query.cast_shape(
            collider,
            *translation,
            rotation,
            Dir3::new(remaining_velocity.normalize_or_zero()).unwrap_or(Dir3::X),
            &ShapeCastConfig::from_max_distance(remaining_velocity.length()),
            filter,
        ) {
            // Calculate our safe distances to move
            let safe_distance = (hit.distance - config.epsilon).max(0.0);

            // How far is safe to translate by
            let safe_movement = remaining_velocity * safe_distance;

            // Move the transform to just before the point of collision
            *translation += safe_movement;

            // Update the velocity by how much we moved
            remaining_velocity -= safe_movement;

            // Project velocity onto the surface plane
            remaining_velocity = remaining_velocity.reject_from(hit.normal1);

            if remaining_velocity.dot(*velocity) < 0.0 {
                // Don't allow sliding back into the surface
                remaining_velocity = Vec3::ZERO;
                break;
            }
        } else {
            // No collision, move the full remaining distance
            *translation += remaining_velocity;
            break;
        }
    }

    // Update the velocity for the next frame
    *velocity = remaining_velocity;
}

fn update_grounded_and_sliding_state(
    mut q_kcc: Query<(Entity, &mut Transform, &mut KinematicVelocity, &Collider), With<KCCMarker>>,
    spatial_query: SpatialQuery,
    config: Res<KCCConfig>,
    mut commands: Commands,
) {
    let Some((entity, mut kcc_transform, mut kinematic_vel, collider)) = q_kcc.single_mut().ok()
    else {
        warn!("No KCC found!");
        return;
    };

    let filter = SpatialQueryFilter::default().with_excluded_entities(vec![entity]);

    let Some(ray) = spatial_query.cast_ray(
        kcc_transform.translation,
        Dir3::NEG_Y,
        config.ground_check_distance,
        false,
        &filter,
    ) else {
        // No ground detected, handle airborne state
        commands.entity(entity).remove::<Grounded>();
        commands.entity(entity).remove::<Sliding>();
        return;
    };

    // based on the angle of the normal, determine if we are grounded or sliding
    let angle = ray.normal.angle_between(Vec3::Y).to_degrees();
    let is_grounded = angle < config.max_walkable_slope_angle;

    let is_sliding = angle > config.max_walkable_slope_angle && angle < 90.0;

    if is_grounded {
        info!("Grounded! Angle: {}", angle.round());
        commands.entity(entity).insert(Grounded);
        commands.entity(entity).remove::<Sliding>();
    } else if is_sliding {
        info!("Sliding! Angle: {}", angle.round());
        commands.entity(entity).insert(Sliding);
        commands.entity(entity).remove::<Grounded>();
    } else {
        commands.entity(entity).remove::<Grounded>();
        commands.entity(entity).remove::<Sliding>();
    }
}
