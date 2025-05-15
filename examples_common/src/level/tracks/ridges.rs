use crate::level::{
    common::{self, Param},
    utils::{BASE_Y, TextureAssets, TrackOffsets},
};
use bevy::prelude::*;
use std::collections::HashMap;

// --- Plugin Definition ---
pub struct RidgesTrackPlugin;

impl Plugin for RidgesTrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_ridges_track.after(super::super::load_assets_and_setup),
        );
    }
}

// --- Constants ---
const TRACK_NAME: &str = "Ridges";
const TRACK_Z: f32 = -60.0; // Place this track further back
const TEX_RIDGE: usize = 2 * 13 + 3; // Example texture
const RIDGE_PLANE_WIDTH: f32 = 3.0;
const RIDGE_THICKNESS: f32 = 0.2;

// --- Parameter Ranges ---
const PARAMS: &[(&str, Param)] = &[
    // Angle of each plane relative to horizontal (e.g., 30 means 30 degrees upwards)
    (
        "plane_angle_deg",
        Param::Float {
            start: 15.0,
            end: 80.0,
            step: 15.0,
        },
    ), // Angles: 20, 40
    // Length of the ridge along the track's Z axis
    (
        "length",
        Param::Float {
            start: 7.0,
            end: 7.0,
            step: 2.0,
        },
    ), // Lengths: 5.0, 7.0
];

// --- Setup System ---
fn setup_ridges_track(
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
            let plane_angle_deg = permutation["plane_angle_deg"] as f32;
            let length = permutation["length"] as f32;

            let name = format!("Ridge_a{:.0}_l{:.1}", plane_angle_deg, length);

            spawn_ridge_instance(
                cmds,
                mshs,
                mats,
                offsets,
                assets,
                &name,
                plane_angle_deg,
                length,
                TEX_RIDGE,
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

/// Spawns a single ridge instance (two angled planes meeting at a peak).
fn spawn_ridge_instance(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    track_offsets: &mut ResMut<TrackOffsets>,
    level_assets: &Res<TextureAssets>,
    name: &str,
    plane_angle_deg: f32, // Angle from horizontal (degrees)
    length: f32,
    texture_index: usize,
) {
    let angle_rad = plane_angle_deg.to_radians();
    // Calculate the total footprint width based on the projection of the planes
    let footprint_x = (RIDGE_PLANE_WIDTH * angle_rad.cos()) * 2.0;
    let section_center_x = track_offsets.get_and_advance(TRACK_NAME, footprint_x);

    if length <= 0.0 || plane_angle_deg <= 0.0 || plane_angle_deg >= 90.0 {
        warn!("Skipping ridge '{}': invalid dimensions/angle.", name);
        return;
    }

    // Size of each individual plane cuboid
    let plane_size = Vec3::new(RIDGE_PLANE_WIDTH, RIDGE_THICKNESS, length);

    // Calculate offsets and position for the planes
    // Horizontal offset from center to the center of each plane
    let plane_center_offset_x = (RIDGE_PLANE_WIDTH / 2.0) * angle_rad.cos();
    // Vertical offset from base to the center of each plane
    let plane_center_offset_y =
        (RIDGE_PLANE_WIDTH / 2.0) * angle_rad.sin() + (RIDGE_THICKNESS / 2.0) * angle_rad.cos();

    let plane_center_y = BASE_Y + plane_center_offset_y;

    // Spawn Left Plane
    let transform_left = Transform::from_xyz(
        section_center_x - plane_center_offset_x,
        plane_center_y,
        TRACK_Z,
    )
    .with_rotation(Quat::from_rotation_z(-angle_rad)); // Rotate upwards/outwards

    common::spawn_static_cuboid(
        commands,
        meshes,
        materials,
        level_assets,
        format!("{}_Left", name),
        plane_size,
        transform_left,
        texture_index,
    );

    // Spawn Right Plane
    let transform_right = Transform::from_xyz(
        section_center_x + plane_center_offset_x,
        plane_center_y,
        TRACK_Z,
    )
    .with_rotation(Quat::from_rotation_z(angle_rad)); // Rotate upwards/outwards

    common::spawn_static_cuboid(
        commands,
        meshes,
        materials,
        level_assets,
        format!("{}_Right", name),
        plane_size,
        transform_right,
        texture_index,
    );
}
