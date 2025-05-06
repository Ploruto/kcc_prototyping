use avian3d::{PhysicsPlugins, prelude::PhysicsDebugPlugin};
use bevy::{
    pbr::{Atmosphere, light_consts::lux},
    prelude::*,
    render::camera::Exposure,
};
use bevy_enhanced_input::prelude::Actions;
use input::{DefaultContext, InputPlugin};
use level::LevelGeneratorPlugin;
use movement::{Character, KCCPlugin};

mod input;
mod level;

mod movement;

#[derive(Component)]
struct DefaultCamera;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        InputPlugin,
        PhysicsPlugins::default(),
        //PhysicsDebugPlugin::default(),
        LevelGeneratorPlugin,
        KCCPlugin,
    ));
    app.add_systems(Startup, setup);

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Transform::from_xyz(0.0, 20.0, 0.0),
        Actions::<DefaultContext>::default(),
        Character::default(),
        children![(
            // Camera with Earth's atmosphere
            Camera3d::default(),
            Camera {
                hdr: true,
                ..Default::default()
            },
            Atmosphere::EARTH,
            Exposure::SUNLIGHT,
            Projection::Perspective(PerspectiveProjection {
                fov: 90.0_f32.to_radians(),
                ..Default::default()
            }),
            DefaultCamera,
            Transform::from_xyz(0.0, 0.5, 0.0)
        )],
    ));

    // Sun
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: lux::RAW_SUNLIGHT,
            ..default()
        },
        Transform::from_xyz(0.0, 2.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
