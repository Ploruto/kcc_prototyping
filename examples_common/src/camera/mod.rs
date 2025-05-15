pub mod fly_camera;
pub mod orbit_camera;

use crate::{
    Frozen,
    input::{DefaultContext, Look, ToggleFlyCam, ToggleViewPerspective},
};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use fly_camera::{FlySpeed, FlyingCamera};
use orbit_camera::{FirstPersonCamera, SpringArm};
use std::f32::consts::PI;

pub(crate) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((fly_camera::plugin, orbit_camera::plugin))
            .add_systems(
                RunFixedMainLoop,
                view_input.in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop),
            )
            .add_systems(Update, update_origin)
            .add_observer(toggle_cam_perspective)
            .add_observer(toggle_fly_cam);
    }
}

#[derive(Component)]
#[require(Camera3d, Sensitivity, ViewAngles, FollowOrigin, SpringArm, FlySpeed)]
pub struct MainCamera;

#[derive(Component)]
#[relationship(relationship_target = TargetedBy)]
pub struct Targeting(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = Targeting)]
pub struct TargetedBy(Entity);

/// The look sensitivity of a camera
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub(crate) struct Sensitivity(pub f32);

impl Default for Sensitivity {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Component, Reflect, Default, Debug, Clone, Copy)]
#[reflect(Component)]
struct ViewAngles {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

impl ViewAngles {
    pub fn to_quat(&self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, self.roll)
    }
}

/// The origin of an attached camera, corresponds to the translation of the [`AttachedTo`] entity + [`FollowOffset`]
#[derive(Component, Reflect, Default, Debug, PartialEq, Clone, Copy)]
#[reflect(Component)]
#[require(FollowOffset)]
pub(crate) struct FollowOrigin(pub Vec3);

/// The offset of an attached camera
#[derive(Component, Reflect, Default, Debug, Clone, Copy)]
#[reflect(Component)]
pub struct FollowOffset {
    pub absolute: Vec3,
    pub relative: Vec3,
}

fn toggle_cam_perspective(
    trigger: Trigger<Fired<ToggleViewPerspective>>,
    mut commands: Commands,
    targets: Query<&TargetedBy>,
    cameras: Query<(Entity, Has<FirstPersonCamera>), With<Camera>>,
) {
    if let Ok(target) = targets.get(trigger.target()) {
        if let Ok((camera, is_first_person)) = cameras.get(target.0) {
            match is_first_person {
                true => commands.entity(camera).remove::<FirstPersonCamera>(),
                false => commands.entity(camera).insert(FirstPersonCamera),
            };
        }
    }
}

fn toggle_fly_cam(
    trigger: Trigger<Fired<ToggleFlyCam>>,
    mut commands: Commands,
    targets: Query<&TargetedBy>,
    cameras: Query<(Entity, Has<FlyingCamera>), With<Camera>>,
) {
    if let Ok(target) = targets.get(trigger.target()) {
        if let Ok((camera, is_fly_camera)) = cameras.get(target.0) {
            match is_fly_camera {
                true => {
                    commands.entity(trigger.target()).remove::<Frozen>();
                    commands
                        .entity(camera)
                        .remove::<FlyingCamera>()
                        .insert(FollowOrigin::default());
                }
                false => {
                    commands.entity(trigger.target()).insert(Frozen);
                    commands
                        .entity(camera)
                        .remove::<FollowOrigin>()
                        .insert(FlyingCamera);
                }
            };
        }
    }
}

fn view_input(
    mut cameras: Query<(&mut ViewAngles, &mut Transform, &Sensitivity)>,
    actions: Single<&Actions<DefaultContext>>,
    time: Res<Time>,
) {
    let actions = actions.into_inner();

    for (mut angles, mut transform, sensitivity) in &mut cameras {
        let orbit_input = actions.action::<Look>().value().as_axis2d() * sensitivity.0;
        let angle_deltas = orbit_input * PI * time.delta_secs();

        angles.pitch += angle_deltas.y;
        angles.pitch = angles.pitch.clamp(-PI / 2.0 + 0.01, PI / 2.0 - 0.01);
        angles.yaw += angle_deltas.x;

        transform.rotation = angles.to_quat();
    }
}

fn update_origin(
    targets: Query<&GlobalTransform>,
    mut cameras: Query<(
        &mut FollowOrigin,
        &mut Transform,
        &ViewAngles,
        &FollowOffset,
        &Targeting,
    )>,
) {
    for (mut origin, mut transform, angles, offset, targeting) in &mut cameras {
        if let Ok(orbit_transform) = targets.get(targeting.0) {
            let mut point = orbit_transform.translation();
            point += offset.absolute;
            point += angles.to_quat() * offset.relative;
            origin.0 = point;
            transform.translation = point;
        }
    }
}
