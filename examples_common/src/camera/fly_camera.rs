use crate::input::{DefaultContext, Fly, FlyCameraContext, Move};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use super::TargetedBy;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        RunFixedMainLoop,
        fly_input.in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop),
    );
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(FlySpeed)]
pub(super) struct FlyingCamera;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub(super) struct FlySpeed(pub f32);

impl Default for FlySpeed {
    fn default() -> Self {
        Self(10.0)
    }
}

fn fly_input(
    targets: Query<(
        &Actions<DefaultContext>,
        &Actions<FlyCameraContext>,
        &TargetedBy,
    )>,
    mut cameras: Query<(&mut Transform, &FlySpeed), With<FlyingCamera>>,
    time: Res<Time>,
) {
    for (default_actions, fly_actions, attachments) in &targets {
        let move_input = default_actions.action::<Move>().value().as_axis2d();
        let fly_input = fly_actions.action::<Fly>().value().as_axis1d();

        if move_input == Vec2::ZERO && fly_input == 0.0 {
            continue;
        }

        let mut iter = cameras.iter_many_mut(attachments.iter());
        while let Some((mut transform, speed)) = iter.fetch_next() {
            let mut direction = transform.rotation * Vec3::new(move_input.x, 0.0, -move_input.y);
            direction.y += fly_input;
            transform.translation += direction * speed.0 * time.delta_secs();
        }
    }
}
