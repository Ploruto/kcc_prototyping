use crate::level::{
    common::{self, Param},
    utils::{BASE_Y, TextureAssets, TrackOffsets},
};
use bevy::prelude::*;
use core::f32;
use std::collections::HashMap;

// --- Plugin Definition ---
pub struct UnevenPatchesTrackPlugin;

impl Plugin for UnevenPatchesTrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_uneven_patches_track.after(super::super::load_assets_and_setup),
        );
    }
}

// --- Constants ---
const TRACK_NAME: &str = "UnevenPatches";
const TRACK_Z: f32 = 40.0; // Place this track forward
const TEX_PATCH: usize = 0; // Example texture (ground?)
const PATCH_THICKNESS: f32 = 0.1;

// --- Parameter Ranges ---
// 2 * 2 * 2 = 8 instances
const PARAMS: &[(&str, Param)] = &[
    // Size of the square grid (e.g., 3 means 3x3 patches)
    (
        "grid_dim",
        Param::Int {
            start: 3,
            end: 3,
            step: 1,
        },
    ), // Grids: 3x3, 4x4
    // Size of each square patch
    (
        "patch_size",
        Param::Float {
            start: 2.0,
            end: 2.0,
            step: 0.5,
        },
    ), // Sizes: 1.5, 2.0
    // Max height difference (+/-) from BASE_Y for patches
    (
        "max_h_var",
        Param::Float {
            start: 0.05,
            end: 0.15,
            step: 0.1,
        },
    ), // Variations: 0.05, 0.15
];
// Spacing will be calculated based on patch_size (e.g., slight overlap)
const SPACING_FACTOR: f32 = 0.9; // Multiplier for patch_size to get spacing

// --- Setup System ---
fn setup_uneven_patches_track(
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
            let grid_dim = permutation["grid_dim"] as i32;
            let patch_size = permutation["patch_size"] as f32;
            let max_h_var = permutation["max_h_var"] as f32;

            let name = format!(
                "Patches_{}x{}_s{:.1}_h{:.2}",
                grid_dim, grid_dim, patch_size, max_h_var
            );
            let patch_spacing = patch_size * SPACING_FACTOR;

            spawn_patch_grid_instance(
                cmds,
                mshs,
                mats,
                offsets,
                assets,
                &name,
                grid_dim,
                patch_size,
                patch_spacing,
                max_h_var,
                TEX_PATCH,
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

/// Spawns a grid of uneven patches.
fn spawn_patch_grid_instance(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    track_offsets: &mut ResMut<TrackOffsets>,
    level_assets: &Res<TextureAssets>,
    name: &str,
    grid_dim: i32, // Grid is grid_dim x grid_dim
    patch_size: f32,
    patch_spacing: f32, // Spacing between patch centers
    max_h_var: f32,     // Max height variation (+/- from BASE_Y)
    texture_index: usize,
) {
    let grid_total_width = grid_dim as f32 * patch_spacing;
    // Footprint along X axis is the total width of the grid
    let section_center_x = track_offsets.get_and_advance(TRACK_NAME, grid_total_width);

    if grid_dim <= 0 || patch_size <= 0.0 || patch_spacing <= 0.0 {
        warn!("Skipping patch grid '{}': invalid dimensions.", name);
        return;
    }

    // Calculate starting position for the grid (bottom-left corner)
    let grid_start_x = section_center_x - grid_total_width / 2.0 + patch_spacing / 2.0;
    // Center grid Z around TRACK_Z
    let grid_start_z = TRACK_Z - grid_total_width / 2.0 + patch_spacing / 2.0;

    let patch_size_vec = Vec3::new(patch_size, PATCH_THICKNESS, patch_size);

    // Parent entity for the grid
    let parent_entity = commands
        .spawn((Transform::IDENTITY, Name::new(name.to_string())))
        .id();

    for i in 0..grid_dim {
        // X dimension index
        for j in 0..grid_dim {
            // Z dimension index
            let patch_center_x = grid_start_x + i as f32 * patch_spacing;
            let patch_center_z = grid_start_z + j as f32 * patch_spacing;

            // Deterministic height variation based on grid position
            let height_factor = ((i as f32 * 1.618 + j as f32 * f32::consts::E).sin() + 1.0) / 2.0; // Value between 0 and 1
            let height_offset = (height_factor * 2.0 - 1.0) * max_h_var; // Value between -max_h_var and +max_h_var
            let patch_y = BASE_Y + height_offset + PATCH_THICKNESS / 2.0;

            let patch_entity = common::spawn_static_cuboid(
                commands,
                meshes,
                materials,
                level_assets,
                format!("{}_patch_{}_{}", name, i, j),
                patch_size_vec,
                Transform::from_xyz(patch_center_x, patch_y, patch_center_z),
                texture_index,
            );
            commands.entity(parent_entity).add_child(patch_entity); // Optional parenting
        }
    }
}
