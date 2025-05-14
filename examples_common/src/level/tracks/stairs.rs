// src/level/plugins/stairs.rs

use crate::level::{
    common::{self, Param}, // Use common helpers and Param enum
    utils::{BASE_Y, TextureAssets, TrackOffsets}, // Use resources/constants
};
use bevy::prelude::*;
use std::collections::HashMap;

// --- Plugin Definition ---
pub struct StairsTrackPlugin;

impl Plugin for StairsTrackPlugin {
    fn build(&self, app: &mut App) {
        // Ensure assets are loaded and TrackOffsets is initialized before running
        app.add_systems(
            Startup,
            setup_stairs_track.after(super::super::load_assets_and_setup), // Depends on TextureAssets resource
                                                                           // .after(TrackOffsets::initialize) // If you add an explicit init system
        );
    }
}

// --- Constants for the Stairs Track ---
const TRACK_NAME: &str = "Stairs";
const TRACK_Z: f32 = -20.0;
const TEX_STAIR: usize = 9;

// --- Parameter Ranges ---
const PARAMS: &[(&str, Param)] = &[
    (
        "width",
        Param::Float {
            start: 4.0,
            end: 4.0,
            step: 1.0,
        },
    ),
    (
        "step_height",
        Param::Float {
            start: 0.1,
            end: 0.3,
            step: 0.1,
        },
    ),
    (
        "step_depth",
        Param::Float {
            start: 0.2,
            end: 0.6,
            step: 0.2,
        },
    ),
    (
        "num_steps",
        Param::Int {
            start: 4,
            end: 4,
            step: 4,
        },
    ),
];

// --- Setup System ---
fn setup_stairs_track(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut track_offsets: ResMut<TrackOffsets>,
    level_assets: Res<TextureAssets>,
    // Animation resources needed by generate_permutations signature, even if not used here
    mut animation_clips: ResMut<Assets<AnimationClip>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    info!("Generating track: {}", TRACK_NAME);

    // Define the closure that spawns one instance based on a permutation
    let generator_closure =
        |permutation: &HashMap<String, f64>,
         cmds: &mut Commands,
         mshs: &mut ResMut<Assets<Mesh>>,
         mats: &mut ResMut<Assets<StandardMaterial>>,
         offsets: &mut ResMut<TrackOffsets>,
         assets: &Res<TextureAssets>,
         _clips: &mut ResMut<Assets<AnimationClip>>, // Mark unused if not needed
         _graphs: &mut ResMut<Assets<AnimationGraph>>| {
            // Extract parameters, converting f64 back to expected types (f32, i32)
            let width = permutation["width"] as f32;
            let step_height = permutation["step_height"] as f32;
            let step_depth = permutation["step_depth"] as f32;
            let num_steps = permutation["num_steps"] as i32; // Be careful with float->int conversion if step isn't integer

            // Generate a unique name
            let name = format!(
                "Stairs_w{:.1}_h{:.1}_d{:.1}_n{}",
                width, step_height, step_depth, num_steps
            );

            // Spawn the instance using the extracted parameters
            spawn_steps_instance(
                cmds,
                mshs,
                mats,
                offsets,
                assets,
                &name,
                num_steps,
                width,
                step_height,
                step_depth,
                TEX_STAIR,
            );
        };

    // Call the permutation generator
    common::generate_permutations(
        PARAMS,
        generator_closure,
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut track_offsets,
        &level_assets,
        &mut animation_clips, // Pass animation resources
        &mut animation_graphs,
    );
}

/// Spawns a single instance (set) of steps on the stairs track.
/// (Function remains largely the same as before)
fn spawn_steps_instance(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    track_offsets: &mut ResMut<TrackOffsets>,
    level_assets: &Res<TextureAssets>,
    base_name: &str,
    num_steps: i32,
    width: f32,
    step_height: f32,
    step_depth: f32,
    texture_index: usize,
) {
    let section_center_x = track_offsets.get_and_advance(TRACK_NAME, width);
    let step_start_y = BASE_Y;

    // Parent entity to group the steps
    let parent_entity = commands
        .spawn((
            Transform::from_xyz(section_center_x, 0.0, TRACK_Z),
            Name::new(base_name.to_string()),
        ))
        .id();

    for i in 0..num_steps {
        if width <= 0.0 || step_height <= 0.0 || step_depth <= 0.0 {
            warn!(
                "Skipping step {} for '{}': non-positive dims.",
                i + 1,
                base_name
            );
            continue;
        }
        let step_size = Vec3::new(width, step_height, step_depth);
        let relative_y = step_start_y + (i as f32 + 0.5) * step_size.y;
        let relative_z = (i as f32 + 0.5) * step_size.z - (num_steps as f32 * step_size.z / 2.0);

        let step_entity = common::spawn_static_cuboid(
            commands,
            meshes,
            materials,
            level_assets,
            format!("{}_step{}", base_name, i + 1),
            step_size,
            Transform::from_xyz(0.0, relative_y, relative_z), // Relative to parent
            texture_index,
        );
        commands.entity(parent_entity).add_child(step_entity);
    }
}
