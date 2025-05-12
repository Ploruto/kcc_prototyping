use crate::level::{
    common::{self, Param},
    utils::{BASE_Y, TextureAssets, TrackOffsets},
};
use bevy::prelude::*;
use core::f32;
use std::collections::HashMap;

// --- Plugin Definition ---
pub struct DebrisFieldTrackPlugin;

impl Plugin for DebrisFieldTrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_debris_field_track.after(super::super::load_assets_and_setup),
        );
    }
}

// --- Constants ---
const TRACK_NAME: &str = "DebrisField";
const TRACK_Z: f32 = 60.0; // Place this track forward
const TEX_DEBRIS: usize = 6; // Example texture

// --- Parameter Ranges ---
// 2 * 2 * 2 = 8 instances
const PARAMS: &[(&str, Param)] = &[
    // Area dimensions (square area for simplicity)
    (
        "area_dim",
        Param::Float {
            start: 4.0,
            end: 4.0,
            step: 2.0,
        },
    ), // Dims: 5x5, 7x7
    // Number of debris items in the area
    (
        "debris_count",
        Param::Int {
            start: 6,
            end: 6,
            step: 10,
        },
    ), // Counts: 10, 20
    // Max size of a debris item (min size will be derived)
    (
        "max_size",
        Param::Float {
            start: 0.2,
            end: 0.4,
            step: 0.2,
        },
    ), // Sizes: 0.2, 0.4
];
const MIN_SIZE_FACTOR: f32 = 0.3; // Min size = max_size * MIN_SIZE_FACTOR

// --- Setup System ---
fn setup_debris_field_track(
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
            let area_dim = permutation["area_dim"] as f32;
            let debris_count = permutation["debris_count"] as i32;
            let max_size = permutation["max_size"] as f32;
            let min_size = max_size * MIN_SIZE_FACTOR;

            let name = format!(
                "Debris_{:.1}x{:.1}_n{}_s{:.2}",
                area_dim, area_dim, debris_count, max_size
            );

            spawn_debris_field_instance(
                cmds,
                mshs,
                mats,
                offsets,
                assets,
                &name,
                area_dim,
                debris_count,
                min_size,
                max_size,
                TEX_DEBRIS,
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

/// Spawns a field of small debris objects.
fn spawn_debris_field_instance(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    track_offsets: &mut ResMut<TrackOffsets>,
    level_assets: &Res<TextureAssets>,
    name: &str,
    area_dim: f32, // Square area dimension
    debris_count: i32,
    min_size: f32,
    max_size: f32,
    texture_index: usize,
) {
    // Footprint is the width of the area
    let section_center_x = track_offsets.get_and_advance(TRACK_NAME, area_dim);

    if area_dim <= 0.0 || debris_count <= 0 || min_size <= 0.0 || max_size <= min_size {
        warn!("Skipping debris field '{}': invalid dimensions.", name);
        return;
    }

    let area_start_x = section_center_x - area_dim / 2.0;
    let area_start_z = TRACK_Z - area_dim / 2.0; // Center area around TRACK_Z

    // Parent entity for the debris
    let parent_entity = commands
        .spawn((
            Transform::IDENTITY, // Debris positioned globally
            Name::new(name.to_string()),
        ))
        .id();

    for i in 0..debris_count {
        // Deterministic pseudo-random placement and size
        let factor_i = i as f32 / debris_count as f32; // 0..1
        let pseudo_random_size = min_size + (factor_i * 1.618).fract() * (max_size - min_size);
        let debris_size = Vec3::splat(pseudo_random_size);

        let pseudo_random_x = area_start_x + (factor_i * f32::consts::PI).fract() * area_dim;
        let pseudo_random_z = area_start_z + (factor_i * f32::consts::E).fract() * area_dim;

        let debris_pos = Vec3::new(
            pseudo_random_x,
            BASE_Y + debris_size.y / 2.0,
            pseudo_random_z,
        );

        // Optional: Add deterministic rotation
        let rot_y = (factor_i * 5.123).fract() * std::f32::consts::TAU; // TAU = 2*PI
        let transform =
            Transform::from_translation(debris_pos).with_rotation(Quat::from_rotation_y(rot_y));

        let debris_entity = common::spawn_static_cuboid(
            commands,
            meshes,
            materials,
            level_assets,
            format!("{}_debris_{}", name, i),
            debris_size,
            transform,
            texture_index,
        );
        commands.entity(parent_entity).add_child(debris_entity);
    }
}
