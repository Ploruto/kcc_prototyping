use crate::input::{DefaultContext, Fly, FlyCameraContext, Move};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use super::Attachments;

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, fly_input);
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(FlySpeed)]
pub(crate) struct FlyingCamera;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub(crate) struct FlySpeed(pub f32);

impl Default for FlySpeed {
    fn default() -> Self {
        Self(10.0)
    }
}

fn fly_input(
    targets: Query<(
        &Actions<DefaultContext>,
        &Actions<FlyCameraContext>,
        &Attachments,
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
            let direction = transform.rotation * Vec3::new(move_input.x, fly_input, -move_input.y);
            transform.translation += direction * speed.0 * time.delta_secs();
        }
    }
}
