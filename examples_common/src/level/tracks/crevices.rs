use crate::level::{
    common::{self, Param},
    utils::{BASE_Y, TextureAssets, TrackOffsets},
};
use bevy::prelude::*;
use std::collections::HashMap;

// --- Plugin Definition ---
pub struct CrevicesTrackPlugin;

impl Plugin for CrevicesTrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_crevices_track.after(super::super::load_assets_and_setup),
        );
    }
}

// --- Constants ---
const TRACK_NAME: &str = "Crevices";
const TRACK_Z: f32 = -40.0; // Place this track further back
const TEX_WALL: usize = 3 * 13; // Example texture
const WALL_HEIGHT: f32 = 5.0; // Keep height constant for simplicity
const WALL_THICKNESS: f32 = 0.2;

// --- Parameter Ranges ---
// Keep permutations low: 2 * 2 * 2 = 8 instances
const PARAMS: &[(&str, Param)] = &[
    // Width at the top opening of the crevice
    (
        "top_width",
        Param::Float {
            start: 0.5,
            end: 1.5,
            step: 1.0,
        },
    ), // Widths: 0.5, 1.5
    // Angle of each wall relative to vertical (e.g., 30 means 30 degrees inwards from vertical)
    (
        "wall_angle_deg",
        Param::Float {
            start: 20.0,
            end: 40.0,
            step: 20.0,
        },
    ), // Angles: 20, 40
    // Length of the crevice along the track's Z axis
    (
        "length",
        Param::Float {
            start: 4.0,
            end: 6.0,
            step: 2.0,
        },
    ), // Lengths: 4.0, 6.0
];

// --- Setup System ---
fn setup_crevices_track(
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
            let top_width = permutation["top_width"] as f32;
            let wall_angle_deg = permutation["wall_angle_deg"] as f32;
            let length = permutation["length"] as f32;

            let name = format!(
                "Crevice_w{:.1}_a{:.0}_l{:.1}",
                top_width, wall_angle_deg, length
            );

            spawn_crevice_instance(
                cmds,
                mshs,
                mats,
                offsets,
                assets,
                &name,
                top_width,
                wall_angle_deg,
                length,
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

/// Spawns a single crevice instance (two angled walls).
fn spawn_crevice_instance(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    track_offsets: &mut ResMut<TrackOffsets>,
    level_assets: &Res<TextureAssets>,
    name: &str,
    top_width: f32,
    wall_angle_deg: f32, // Angle from vertical (degrees)
    length: f32,
    texture_index: usize,
) {
    // Calculate the footprint based on the top width and wall thickness projection
    let angle_rad = wall_angle_deg.to_radians();
    let footprint_x = top_width + WALL_THICKNESS * angle_rad.sin() * 2.0; // Approximate width
    let section_center_x = track_offsets.get_and_advance(TRACK_NAME, footprint_x);

    if top_width <= 0.0 || length <= 0.0 || wall_angle_deg <= 0.0 || wall_angle_deg >= 90.0 {
        warn!("Skipping crevice '{}': invalid dimensions/angle.", name);
        return;
    }

    let wall_size = Vec3::new(WALL_THICKNESS, WALL_HEIGHT, length);

    // Calculate offsets for the two walls based on the angle and top width
    // Horizontal offset from center to the *inner* edge of the wall base
    let inner_edge_offset_x = top_width / 2.0;
    // Horizontal position of the wall's center
    let wall_center_offset_x = inner_edge_offset_x + (WALL_THICKNESS / 2.0) * angle_rad.cos();
    // Vertical offset of the wall's center due to rotation
    let wall_center_offset_y = -(WALL_THICKNESS / 2.0) * angle_rad.sin();

    let wall_base_y = BASE_Y + WALL_HEIGHT / 2.0 + wall_center_offset_y;

    // Spawn Left Wall
    let transform_left = Transform::from_xyz(
        section_center_x - wall_center_offset_x,
        wall_base_y,
        TRACK_Z,
    )
    .with_rotation(Quat::from_rotation_z(angle_rad)); // Rotate inwards

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
    let transform_right = Transform::from_xyz(
        section_center_x + wall_center_offset_x,
        wall_base_y,
        TRACK_Z,
    )
    .with_rotation(Quat::from_rotation_z(-angle_rad)); // Rotate inwards

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
