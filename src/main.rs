use avian3d::{
    PhysicsPlugins,
    prelude::{Collider, PhysicsDebugPlugin, RigidBody},
};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::{ActionState, Actions};
use input::{DefaultContext, InputPlugin, Jump};
use level::LevelPlugin;
use movement::{KCCBundle, KCCPlugin};
mod input;
mod level;
mod movement;

#[derive(Component)]
struct KCCMarker;

#[derive(Component)]
struct DefaultCamera;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        InputPlugin,
        PhysicsPlugins::default(),
        PhysicsDebugPlugin::default(),
        LevelPlugin,
        KCCPlugin,
    ));
    app.add_systems(Startup, setup);

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Transform::from_xyz(0.0, 3.5, 0.0),
        Actions::<DefaultContext>::default(),
        KCCBundle::default(),
        children![(
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                fov: 75.0_f32.to_radians(),
                ..Default::default()
            }),
            DefaultCamera,
            Transform::from_xyz(0.0, 0.75, 0.0)
        )],
    ));
}
