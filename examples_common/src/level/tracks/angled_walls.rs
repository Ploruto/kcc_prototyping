use crate::level::{
    common::{self, Param},
    utils::{BASE_Y, TextureAssets, TrackOffsets},
};
use bevy::prelude::*;
use std::collections::HashMap;

// --- Plugin Definition ---
pub struct AngledWallsTrackPlugin;

impl Plugin for AngledWallsTrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_angled_walls_track.after(super::super::load_assets_and_setup),
        );
    }
}

// --- Constants ---
const TRACK_NAME: &str = "AngledWalls";
const TRACK_Z: f32 = -80.0;
const TEX_WALL: usize = 3 * 13 + 1; // Example texture
const WALL_HEIGHT: f32 = 4.0;
const WALL_THICKNESS: f32 = 0.2;

// --- Parameter Ranges ---
const PARAMS: &[(&str, Param)] = &[
    // Angle (degrees) each wall deviates from parallel to the Z axis.
    // Positive = converging inwards along +Z, Negative = diverging outwards.
    (
        "wall_angle_dev_deg",
        Param::Float {
            start: -30.0,
            end: 30.0,
            step: 15.0,
        },
    ), // Angles: -3, +3
    // Initial width of the corridor at the start (along X)
    (
        "corridor_width",
        Param::Float {
            start: 1.0,
            end: 2.,
            step: 1.0,
        },
    ), // Widths: 1.0, 2.0
];
// Length of the corridor walls along Z
const CORRIDOR_LENGTH: f32 = 7.5;

// --- Setup System ---
fn setup_angled_walls_track(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut track_offsets: ResMut<TrackOffsets>,
    level_assets: Res<TextureAssets>,
    mut animation_clips: ResMut<Assets<AnimationClip>>, // Needed for signature
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    info!("Generating track: {}", TRACK_NAME);

    let generator_closure =
        |permutation: &HashMap<String, f64>,
         cmds: &mut Commands,
         mshs: &mut ResMut<Assets<Mesh>>,
         mats: &mut ResMut<Assets<StandardMaterial>>,
         offsets: &mut ResMut<TrackOffsets>,
         assets: &Res<TextureAssets>,
         _clips: &mut ResMut<Assets<AnimationClip>>,
         _graphs: &mut ResMut<Assets<AnimationGraph>>| {
            let wall_angle_dev_deg = permutation["wall_angle_dev_deg"] as f32;
            let corridor_width = permutation["corridor_width"] as f32;

            let name = format!(
                "AngledCorridor_w{:.1}_dev{:.0}",
                corridor_width, wall_angle_dev_deg
            );

            spawn_angled_corridor_instance(
                cmds,
                mshs,
                mats,
                offsets,
                assets,
                &name,
                corridor_width,
                wall_angle_dev_deg,
                TEX_WALL,
            );
        };

    common::generate_permutations(
        PARAMS,
        generator_closure,
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut track_offsets,
        &level_assets,
        &mut animation_clips,
        &mut animation_graphs,
    );
}

/// Spawns a single corridor with slightly angled walls.
fn spawn_angled_corridor_instance(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    track_offsets: &mut ResMut<TrackOffsets>,
    level_assets: &Res<TextureAssets>,
    name: &str,
    corridor_width: f32,
    wall_angle_dev_deg: f32, // Deviation angle from Z axis
    texture_index: usize,
) {
    // Footprint is roughly the corridor width
    let section_center_x =
        track_offsets.get_and_advance(TRACK_NAME, corridor_width + WALL_THICKNESS * 2.0);

    if corridor_width <= 0.0 || CORRIDOR_LENGTH <= 0.0 {
        warn!("Skipping angled corridor '{}': invalid dimensions.", name);
        return;
    }

    let wall_size = Vec3::new(WALL_THICKNESS, WALL_HEIGHT, CORRIDOR_LENGTH);
    let wall_angle_rad = wall_angle_dev_deg.to_radians();
    let half_width = corridor_width / 2.0;

    // Calculate position for the center of each wall segment
    // Walls are centered vertically at BASE_Y + height/2
    // Walls are centered along Z at TRACK_Z
    // Walls are offset horizontally by half_width + thickness/2 (approx)
    let wall_center_y = BASE_Y + WALL_HEIGHT / 2.0;
    let wall_center_z = TRACK_Z; // Center the length of the wall on the track Z

    // Spawn Left Wall
    let left_wall_x = section_center_x - half_width - (WALL_THICKNESS / 2.0);
    let transform_left = Transform::from_xyz(left_wall_x, wall_center_y, wall_center_z)
        .with_rotation(Quat::from_rotation_y(wall_angle_rad)); // Rotate around Y

    common::spawn_static_cuboid(
        commands,
        meshes,
        materials,
        level_assets,
        format!("{}_Left", name),
        wall_size,
        transform_left,
        texture_index,
    );

    // Spawn Right Wall
    let right_wall_x = section_center_x + half_width + (WALL_THICKNESS / 2.0);
    let transform_right = Transform::from_xyz(right_wall_x, wall_center_y, wall_center_z)
        .with_rotation(Quat::from_rotation_y(-wall_angle_rad)); // Rotate opposite direction

    common::spawn_static_cuboid(
        commands,
        meshes,
        materials,
        level_assets,
        format!("{}_Right", name),
        wall_size,
        transform_right,
        texture_index,
    );
}
