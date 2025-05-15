use crate::level::{
    common::{self, Param},
    utils::{BASE_Y, TextureAssets, TrackOffsets},
};
use avian3d::prelude::Collider;
use bevy::prelude::*;
use core::f32;
use std::collections::HashMap;

// --- Plugin Definition ---
pub struct CapsuleForestTrackPlugin;

impl Plugin for CapsuleForestTrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_capsule_forest_track.after(super::super::load_assets_and_setup),
        );
    }
}

// --- Constants ---
const TRACK_NAME: &str = "CapsuleForest";
const TRACK_Z: f32 = -120.0; // Place this track
const TEX_CAPSULE: usize = 4 * 13; // Example texture

// --- Parameter Ranges ---
// 2 * 2 = 4 instances
const PARAMS: &[(&str, Param)] = &[
    // Area dimensions (square area)
    (
        "area_dim",
        Param::Float {
            start: 8.0,
            end: 8.0,
            step: 2.0,
        },
    ), // Dims: 6x6, 8x8
    // Number of capsules
    (
        "capsule_count",
        Param::Int {
            start: 4,
            end: 4,
            step: 4,
        },
    ), // Counts: 8, 12
];
// Fixed ranges for capsule geometry for simplicity
const MIN_RADIUS: f32 = 0.3;
const MAX_RADIUS: f32 = 0.6;
const MIN_HALF_HEIGHT: f32 = 0.5; // Half length of cylindrical part
const MAX_HALF_HEIGHT: f32 = 1.5;

// --- Setup System ---
fn setup_capsule_forest_track(
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
            let capsule_count = permutation["capsule_count"] as i32;

            let name = format!(
                "CapsuleForest_{:.1}x{:.1}_n{}",
                area_dim, area_dim, capsule_count
            );

            spawn_capsule_forest_instance(
                cmds,
                mshs,
                mats,
                offsets,
                assets,
                &name,
                area_dim,
                capsule_count,
                TEX_CAPSULE,
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

/// Spawns a field of vertical capsules.
fn spawn_capsule_forest_instance(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    track_offsets: &mut ResMut<TrackOffsets>,
    level_assets: &Res<TextureAssets>,
    name: &str,
    area_dim: f32,
    capsule_count: i32,
    texture_index: usize,
) {
    let section_center_x = track_offsets.get_and_advance(TRACK_NAME, area_dim);

    if area_dim <= 0.0 || capsule_count <= 0 {
        warn!("Skipping capsule forest '{}': invalid dimensions.", name);
        return;
    }

    let area_start_x = section_center_x - area_dim / 2.0;
    let area_start_z = TRACK_Z - area_dim / 2.0;

    let parent_entity = commands
        .spawn((Transform::default(), Name::new(name.to_string())))
        .id();

    for i in 0..capsule_count {
        // Deterministic pseudo-random placement and size
        let factor_i = i as f32 / capsule_count as f32;
        let radius = MIN_RADIUS + (factor_i * 1.618).fract() * (MAX_RADIUS - MIN_RADIUS);
        let half_height = MIN_HALF_HEIGHT
            + (factor_i * f32::consts::E).fract() * (MAX_HALF_HEIGHT - MIN_HALF_HEIGHT);

        let pos_x = area_start_x + (factor_i * f32::consts::PI).fract() * area_dim;
        let pos_z = area_start_z + (factor_i * 5.12345).fract() * area_dim;

        // Calculate Y pos to place the bottom hemisphere cap near BASE_Y
        let pos_y = BASE_Y + radius + half_height; // Center of the capsule

        let transform = Transform::from_xyz(pos_x, pos_y, pos_z);
        let mesh_handle = meshes.add(Capsule3d::new(radius, half_height * 2.0)); // Bevy Capsule uses full height
        let collider = Collider::capsule(radius, half_height * 2.0); // Avian Capsule uses full height
        let bbox = Vec3::new(radius * 2.0, half_height * 2.0 + radius * 2.0, radius * 2.0);

        let capsule_entity = common::spawn_static_shape(
            commands,
            materials,
            level_assets,
            format!("{}_capsule_{}", name, i),
            mesh_handle,
            collider,
            transform,
            texture_index,
            bbox,
        );
        commands.entity(parent_entity).add_child(capsule_entity);
    }
}
