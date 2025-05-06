use avian3d::{PhysicsPlugins, prelude::PhysicsDebugPlugin};
use bevy::{
    pbr::{Atmosphere, light_consts::lux},
    prelude::*,
    render::camera::Exposure,
};
use bevy_enhanced_input::prelude::Actions;
use kcc_prototype::{
    camera::{CameraPlugin, MainCamera, TargetOf},
    input::{DefaultContext, InputPlugin},
    level::LevelGeneratorPlugin,
    movement::{Character, KCCPlugin},
};

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            InputPlugin,
            CameraPlugin,
            PhysicsPlugins::default(),
            //PhysicsDebugPlugin::default(),
            LevelGeneratorPlugin,
            KCCPlugin,
        ))
        .add_systems(Startup, setup)
        .run()
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let main_cam = commands
        .spawn((
            MainCamera::default(),
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
            AmbientLight {
                brightness: lux::AMBIENT_DAYLIGHT,
                ..Default::default()
            },
            Transform::from_xyz(0.0, 0.5, 0.0),
        ))
        .id();

    commands.spawn((
        Transform::from_xyz(0.0, 10.5, 0.0),
        Actions::<DefaultContext>::default(),
        Character::default(),
        TargetOf(main_cam),
        Mesh3d(meshes.add(Capsule3d::new(0.35, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE.with_alpha(0.25),
            alpha_mode: AlphaMode::Blend,
            ..Default::default()
        })),
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
