use crate::level::{
    common::{self, Param},
    utils::{BASE_Y, TextureAssets, TrackOffsets},
};
use bevy::prelude::*;
use std::collections::HashMap;

// --- Plugin Definition ---
pub struct NarrowBeamsTrackPlugin;

impl Plugin for NarrowBeamsTrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_narrow_beams_track.after(super::super::load_assets_and_setup),
        );
    }
}

// --- Constants ---
const TRACK_NAME: &str = "NarrowBeams";
const TRACK_Z: f32 = 80.0; // Place this track forward
const TEX_BEAM: usize = 1 * 13; // Example texture
const BEAM_HEIGHT: f32 = 0.2; // Keep height constant

// --- Parameter Ranges ---
// 2 * 2 = 4 instances
const PARAMS: &[(&str, Param)] = &[
    // Width of the beam
    (
        "width",
        Param::Float {
            start: 0.3,
            end: 0.5,
            step: 0.2,
        },
    ), // Widths: 0.3, 0.5
    // Length of the beam
    (
        "length",
        Param::Float {
            start: 8.0,
            end: 12.0,
            step: 4.0,
        },
    ), // Lengths: 8.0, 12.0
];
// Beam starts slightly above ground
const BEAM_START_HEIGHT_OFFSET: f32 = 1.0;

// --- Setup System ---
fn setup_narrow_beams_track(
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
            let width = permutation["width"] as f32;
            let length = permutation["length"] as f32;

            let name = format!("Beam_w{:.1}_l{:.1}", width, length);

            spawn_beam_instance(
                cmds, mshs, mats, offsets, assets, &name, width, length, TEX_BEAM,
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

/// Spawns a single narrow beam instance.
fn spawn_beam_instance(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    track_offsets: &mut ResMut<TrackOffsets>,
    level_assets: &Res<TextureAssets>,
    name: &str,
    width: f32,
    length: f32,
    texture_index: usize,
) {
    // Footprint is the width of the beam itself
    let section_center_x = track_offsets.get_and_advance(TRACK_NAME, width);

    if width <= 0.0 || length <= 0.0 {
        warn!("Skipping beam '{}': invalid dimensions.", name);
        return;
    }

    let beam_size = Vec3::new(width, BEAM_HEIGHT, length);
    let beam_pos = Vec3::new(
        section_center_x,
        BASE_Y + BEAM_START_HEIGHT_OFFSET + BEAM_HEIGHT / 2.0,
        TRACK_Z, // Center beam on track Z
    );

    common::spawn_static_cuboid(
        commands,
        meshes,
        materials,
        level_assets,
        name.to_string(),
        beam_size,
        Transform::from_translation(beam_pos),
        texture_index,
    );
}
