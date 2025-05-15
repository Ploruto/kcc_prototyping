use super::{CameraTarget, CameraTargetOf, FollowOrigin};
use crate::input::{OrbitCameraContext, OrbitZoom};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        RunFixedMainLoop,
        zoom_input.in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop),
    );
    app.add_systems(Update, update_spring_arm.after(super::update_origin));
}

#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
pub(super) struct SpringArm {
    pub distance: f32,
    pub target_distance: f32,
    pub recover_speed: f32,
    pub collision_radius: f32,
    pub filters: LayerMask,
}

impl Default for SpringArm {
    fn default() -> Self {
        Self {
            distance: 4.0,
            target_distance: 4.0,
            recover_speed: 6.0,
            collision_radius: 0.1,
            filters: LayerMask::ALL,
        }
    }
}

#[derive(Component, Reflect, Default, Debug, Clone, Copy)]
#[reflect(Component)]
pub(super) struct FirstPersonCamera; // Used for toggling the spring arm distance without removing it

pub(super) fn zoom_input(
    targets: Query<(&Actions<OrbitCameraContext>, &CameraTarget)>,
    mut cameras: Query<&mut SpringArm>,
) -> Result {
    for (actions, owned_cameras) in &targets {
        let mut iter = cameras.iter_many_mut(owned_cameras.iter());
        while let Some(mut arm) = iter.fetch_next() {
            let zoom_input = actions.action::<OrbitZoom>().value().as_axis2d();
            let zoom_delta = zoom_input.y * arm.distance * 0.1; // TODO: configurable speed

            arm.target_distance -= zoom_delta;
            arm.target_distance = arm.target_distance.clamp(0.1, 100.0); // TODO: configurable range
        }
    }

    Ok(())
}

pub(super) fn update_spring_arm(
    spatial_query: SpatialQuery,
    mut cameras: Query<(
        &mut SpringArm,
        &mut Transform,
        &FollowOrigin,
        &CameraTargetOf,
        Has<FirstPersonCamera>,
    )>,
    time: Res<Time>,
) {
    for (mut arm, mut camera_transform, origin, attached_to, first_person) in &mut cameras {
        let direction = camera_transform.rotation * Dir3::Z;

        let filter =
            SpatialQueryFilter::from_mask(arm.filters).with_excluded_entities([attached_to.0]);

        // Smoothly interpolate to an arm distance of 0.0 when in first person mode
        if first_person {
            arm.distance = arm
                .distance
                .lerp(0.0, arm.recover_speed * time.delta_secs());
        } else if let Some(hit) = spatial_query.cast_shape(
            &Collider::sphere(arm.collision_radius),
            origin.0,
            Quat::IDENTITY,
            direction,
            &ShapeCastConfig {
                max_distance: arm.target_distance,
                ..Default::default()
            },
            &filter,
        ) {
            // If there's a collision, quickly snap to the hit distance to avoid clipping with the world
            arm.distance = hit.distance;
        } else {
            // Otherwise, interpolate to the target distance
            let distance = arm
                .distance
                .lerp(arm.target_distance, arm.recover_speed * time.delta_secs());
            arm.distance = distance;
        }

        camera_transform.translation = origin.0 + direction * arm.distance;
    }
}
