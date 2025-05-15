use crate::level::{
    common::{self, Param},
    utils::{BASE_Y, TextureAssets, TrackOffsets},
};
use bevy::prelude::*;
use std::collections::HashMap;

// --- Plugin Definition ---
pub struct RampsTrackPlugin;

impl Plugin for RampsTrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_ramps_track.after(super::super::load_assets_and_setup),
        );
    }
}

// --- Constants ---
const TRACK_NAME: &str = "Ramps";
const TRACK_Z: f32 = 0.0;
const TEX_RAMP: usize = 4 * 13 + 7;
const WIDTH: f32 = 4.0; // Keep width constant for ramps in this example
const THICKNESS: f32 = 0.2;

// --- Parameter Ranges ---
const PARAMS: &[(&str, Param)] = &[
    (
        "length",
        Param::Float {
            start: 4.0,
            end: 8.0,
            step: 4.0,
        },
    ),
    (
        "angle",
        Param::Float {
            start: 5.0,
            end: 80.0,
            step: 15.0,
        },
    ), // Angle in degrees
];

// --- Setup System ---
fn setup_ramps_track(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut track_offsets: ResMut<TrackOffsets>,
    level_assets: Res<TextureAssets>,
    mut animation_clips: ResMut<Assets<AnimationClip>>, // Still needed for signature
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
            let length = permutation["length"] as f32;
            let angle_degrees = permutation["angle"] as f32;

            let name = format!("Ramp_l{:.1}_a{:.0}", length, angle_degrees);

            spawn_ramp_instance(
                cmds,
                mshs,
                mats,
                offsets,
                assets,
                &name,
                WIDTH,
                length,
                THICKNESS,
                angle_degrees,
                TEX_RAMP,
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

/// Spawns a single ramp instance on the ramps track.
/// (Function remains largely the same as before)
fn spawn_ramp_instance(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    track_offsets: &mut ResMut<TrackOffsets>,
    level_assets: &Res<TextureAssets>,
    name: &str,
    width: f32,
    length: f32,
    thickness: f32,
    angle_degrees: f32,
    texture_index: usize,
) {
    let section_center_x = track_offsets.get_and_advance(TRACK_NAME, width);

    if width <= 0.0 || length <= 0.0 || thickness <= 0.0 {
        warn!("Skipping ramp '{}': non-positive dims.", name);
        return;
    }

    let ramp_size = Vec3::new(width, thickness, length);
    let angle_rad = angle_degrees.to_radians();

    let length_y_proj = (ramp_size.z / 2.0) * angle_rad.sin();
    let thickness_y_proj = (ramp_size.y / 2.0) * angle_rad.cos();
    let ramp_center_y = BASE_Y + length_y_proj + thickness_y_proj;

    // Ramps typically extend along Z, adjust position relative to track Z
    // If angle > 0, ramp goes up towards +Y and extends towards -Z relative to its center
    let length_z_proj = (ramp_size.z / 2.0) * angle_rad.cos();
    let _ramp_center_z = TRACK_Z - length_z_proj * angle_rad.signum(); // Adjust based on angle sign? Or assume positive angle means upward slope towards +Z? Let's assume upward slope along +Z axis relative to local X. Needs testing.
    // Let's stick to the original calculation for now, assuming rotation places it correctly.
    let ramp_center_z = TRACK_Z; // Place center at track Z, rotation handles orientation.

    let transform = Transform::from_xyz(section_center_x, ramp_center_y, ramp_center_z)
        .with_rotation(Quat::from_rotation_x(-angle_rad));

    common::spawn_static_cuboid(
        commands,
        meshes,
        materials,
        level_assets,
        name.to_string(),
        ramp_size,
        transform,
        texture_index,
    );
}
