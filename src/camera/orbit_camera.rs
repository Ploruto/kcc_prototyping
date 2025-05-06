use super::{MainCamera, TargetOf};
use crate::input::{DefaultContext, Look, OrbitCameraContext, OrbitZoom};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use std::{f32::consts::PI, ops::Range};

pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_orbit_angles, orbit, prevent_blindness).chain(),
        )
        .add_systems(FixedPostUpdate, update_orbit_center)
        .add_observer(update_orbit_radius);
    }
}

#[derive(Component, InputContext)]
#[require(Camera3d, OrbitCenter, OrbitAngles)]
pub struct OrbitCamera {
    pub orbit_radius: f32,
    pub orbit_speed: f32,
    /// Must be in the range (-90°, 90°) to avoid glitchy quaternion math
    pub pitch_constraint: Range<f32>,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            orbit_radius: 5.0,
            orbit_speed: PI,
            pitch_constraint: -15f32.to_radians()..89.9f32.to_radians(),
        }
    }
}

/// By default the center of orbit will be equal to the position of the target.
#[derive(Default, Component)]
pub struct OrbitCenter(Vec3);

#[derive(Default, Component)]
pub struct OrbitAngles {
    pub pitch: f32,
    pub yaw: f32,
}

#[derive(Component)]
pub struct PreventBlindness {
    camera_collider: Collider,
}

impl Default for PreventBlindness {
    fn default() -> Self {
        Self {
            camera_collider: Collider::sphere(0.25),
        }
    }
}

fn update_orbit_center(
    targets: Query<(&Transform, &TargetOf)>,
    mut cameras: Query<&mut OrbitCenter>,
) {
    for (target_transform, target_of) in &targets {
        if let Ok(mut orbit_center) = cameras.get_mut(target_of.0) {
            orbit_center.0 = target_transform.translation;
        }
    }
}

fn update_orbit_radius(
    _trigger: Trigger<Fired<OrbitZoom>>,
    mut cameras: Query<(&mut OrbitCamera, &OrbitCenter, &OrbitAngles)>,
    actions: Single<&Actions<OrbitCameraContext>>,
) {
    let actions = actions.into_inner();
    for (mut camera, center, angles) in &mut cameras {
        let zoom_input = actions.action::<OrbitZoom>().value().as_axis2d();
        let zoom_delta = zoom_input.y * camera.orbit_radius * 0.1;
        camera.orbit_radius -= zoom_delta;
        camera.orbit_radius = camera.orbit_radius.clamp(0.1, 100.0);
    }
}

fn update_orbit_angles(
    mut cameras: Query<(&mut OrbitAngles, &OrbitCamera, &MainCamera)>,
    actions: Single<&Actions<DefaultContext>>,
    time: Res<Time>,
) {
    let actions = actions.into_inner();
    for (mut angles, camera, main_camera) in &mut cameras {
        let orbit_input = actions.action::<Look>().value().as_axis2d() * main_camera.sensitivity;
        let angle_deltas = orbit_input * camera.orbit_speed * time.delta_secs();
        angles.pitch += angle_deltas.y;
        angles.pitch =
            -(-angles.pitch).clamp(camera.pitch_constraint.start, camera.pitch_constraint.end);
        angles.yaw += angle_deltas.x;
    }
}

fn orbit(
    targets: Query<(&Transform, &TargetOf), Without<OrbitCamera>>,
    mut cameras: Query<(&mut Transform, &OrbitCenter, &OrbitAngles, &OrbitCamera)>,
) {
    for (target_transform, target_of) in &targets {
        if let Ok((mut camera_transform, center, angles, camera)) = cameras.get_mut(target_of.0) {
            let direction =
                Quat::from_euler(EulerRot::YXZ, angles.yaw, angles.pitch, 0.0) * Vec3::Z;
            let orbit_position = center.0 + direction * camera.orbit_radius;
            camera_transform.translation = orbit_position;
            camera_transform.look_at(target_transform.translation, Vec3::Y);
        }
    }
}

fn prevent_blindness(
    mut cameras: Query<(&mut Transform, &PreventBlindness), With<OrbitCamera>>,
    targets: Query<(Entity, &Transform, &TargetOf), Without<OrbitCamera>>,
    spatial_query: SpatialQuery,
) {
    for (target_entity, target_transform, target_of) in &targets {
        if let Ok((mut camera_transform, pb)) = cameras.get_mut(target_of.0) {
            let Ok((direction, distance)) =
                Dir3::new_and_length(camera_transform.translation - target_transform.translation)
            else {
                return;
            };

            if let Some(hit) = spatial_query.cast_shape(
                &pb.camera_collider,
                target_transform.translation,
                target_transform.rotation,
                direction,
                &ShapeCastConfig {
                    max_distance: distance,
                    ..Default::default()
                },
                &SpatialQueryFilter::from_excluded_entities([target_entity]),
            ) {
                let hit_position = target_transform.translation + direction * hit.distance;
                camera_transform.translation = hit_position;
            }
        }
    }
}
