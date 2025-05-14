use crate::level::{
    common::{self, Param},
    utils::{BASE_Y, TextureAssets, TrackOffsets},
};
use bevy::prelude::*;
use std::collections::HashMap;

// --- Plugin Definition ---
pub struct HalfHeightObstaclesTrackPlugin;

impl Plugin for HalfHeightObstaclesTrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_half_height_obstacles_track.after(super::super::load_assets_and_setup),
        );
    }
}

// --- Constants ---
const TRACK_NAME: &str = "HalfHeightObstacles";
const TRACK_Z: f32 = 100.0; // Place this track far forward
const TEX_OBSTACLE: usize = 4 * 13; // Example texture (wall?)
const OBSTACLE_THICKNESS: f32 = 0.2; // Keep thickness constant

// --- Parameter Ranges ---
// 2 * 2 = 4 instances
const PARAMS: &[(&str, Param)] = &[
    // Height of the obstacle (should require crouching for typical characters)
    (
        "height",
        Param::Float {
            start: 0.8,
            end: 1.2,
            step: 0.4,
        },
    ), // Heights: 0.8, 1.2
    // Width of the obstacle wall
    (
        "width",
        Param::Float {
            start: 3.0,
            end: 5.0,
            step: 2.0,
        },
    ), // Widths: 3.0, 5.0
];

// --- Setup System ---
fn setup_half_height_obstacles_track(
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
            let height = permutation["height"] as f32;
            let width = permutation["width"] as f32;

            let name = format!("HalfObstacle_h{:.1}_w{:.1}", height, width);

            spawn_half_obstacle_instance(
                cmds,
                mshs,
                mats,
                offsets,
                assets,
                &name,
                height,
                width,
                TEX_OBSTACLE,
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

/// Spawns a single half-height obstacle instance.
fn spawn_half_obstacle_instance(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    track_offsets: &mut ResMut<TrackOffsets>,
    level_assets: &Res<TextureAssets>,
    name: &str,
    height: f32,
    width: f32,
    texture_index: usize,
) {
    // Footprint is the width of the obstacle
    let section_center_x = track_offsets.get_and_advance(TRACK_NAME, width);

    if height <= 0.0 || width <= 0.0 {
        warn!("Skipping half-obstacle '{}': invalid dimensions.", name);
        return;
    }

    let obstacle_size = Vec3::new(width, height, OBSTACLE_THICKNESS);
    let obstacle_pos = Vec3::new(
        section_center_x,
        BASE_Y + height / 2.0, // Position based on its height from the ground
        TRACK_Z,
    );

    common::spawn_static_cuboid(
        commands,
        meshes,
        materials,
        level_assets,
        name.to_string(),
        obstacle_size,
        Transform::from_translation(obstacle_pos),
        texture_index,
    );
}
