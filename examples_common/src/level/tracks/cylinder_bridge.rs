use crate::level::{
    common::{self, Param},
    utils::{BASE_Y, TextureAssets, TrackOffsets},
};
use avian3d::prelude::Collider;
use bevy::prelude::*;
use std::collections::HashMap;

// --- Plugin Definition ---
pub struct CylinderBridgeTrackPlugin;

impl Plugin for CylinderBridgeTrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_cylinder_bridge_track.after(super::super::load_assets_and_setup),
        );
    }
}

// --- Constants ---
const TRACK_NAME: &str = "CylinderBridge";
const TRACK_Z: f32 = -140.0; // Place this track
const TEX_CYLINDER: usize = 5 * 13; // Example texture

// --- Parameter Ranges ---
// 2 * 2 = 4 instances
const PARAMS: &[(&str, Param)] = &[
    // Radius of the cylinder
    (
        "radius",
        Param::Float {
            start: 1.2,
            end: 1.2,
            step: 0.4,
        },
    ), // Radii: 0.8, 1.2
    // Angle (degrees) the cylinder is sloped along its length (Z axis)
    (
        "slope_deg",
        Param::Float {
            start: 0.0,
            end: 60.0,
            step: 15.0,
        },
    ), // Slopes: -5, +5
];
// Length of the cylinder bridge
const LENGTH: f32 = 10.0;

// --- Setup System ---
fn setup_cylinder_bridge_track(
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
            let radius = permutation["radius"] as f32;
            let slope_deg = permutation["slope_deg"] as f32;

            let name = format!("CylBridge_r{:.1}_s{:.0}", radius, slope_deg);

            spawn_cylinder_bridge_instance(
                cmds,
                mshs,
                mats,
                offsets,
                assets,
                &name,
                radius,
                slope_deg,
                TEX_CYLINDER,
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

/// Spawns a single cylinder bridge instance, potentially sloped.
fn spawn_cylinder_bridge_instance(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    track_offsets: &mut ResMut<TrackOffsets>,
    level_assets: &Res<TextureAssets>,
    name: &str,
    radius: f32,
    slope_deg: f32, // Slope along Z axis
    texture_index: usize,
) {
    // Footprint along X is diameter
    let footprint_x = radius * 2.0;
    let section_center_x = track_offsets.get_and_advance(TRACK_NAME, footprint_x);

    if radius <= 0.0 || LENGTH <= 0.0 {
        warn!("Skipping cylinder bridge '{}': invalid dimensions.", name);
        return;
    }

    let slope_rad = slope_deg.to_radians();

    // Position the center of the cylinder
    // Base Y position should be BASE_Y (bottom touches ground)
    let center_y = BASE_Y + radius;
    let center_z = TRACK_Z; // Center length on track Z

    // Create mesh and collider
    // Bevy Cylinder length is height, Avian Cylinder length is height.
    // For a bridge lying down, the Bevy mesh needs rotation, Avian collider needs rotation.
    let mesh_handle = meshes.add(Cylinder::new(radius, LENGTH));
    let collider = Collider::cylinder(radius, LENGTH); // Avian collider matches Bevy mesh orientation initially

    // Create transform: position, rotate 90 deg around X to lay it flat, then apply slope around X
    let transform = Transform::from_xyz(section_center_x, center_y, center_z)
        // First, lay it flat along Z axis
        .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
        // Then, apply slope rotation around X axis (relative to its now horizontal orientation)
        // This might need adjustment depending on desired slope axis relative to world/local.
        // Let's assume slope along world Z, meaning rotation around world X *after* laying flat.
        // Alternatively, rotate around its *local* Y axis before laying flat?
        // Let's try rotating around world X after laying flat.
        .with_rotation(Quat::from_rotation_x(slope_rad)); // This slopes ends up/down along Y

    // --- Alternative Rotation: Slope along Z ---
    // If we want the cylinder axis to slope upwards/downwards along the Z direction:
    // 1. Rotate around X to lay flat.
    // 2. Rotate around Y to align length with Z. (Already done by default mesh orientation + step 1?)
    // 3. Rotate around X *again* to introduce slope along Z. (Seems redundant)
    // Let's try rotating around Y *before* laying flat to achieve slope along Z.
    // let initial_rotation = Quat::from_rotation_y(slope_rad);
    // let final_rotation = initial_rotation * Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    // let transform = Transform::from_xyz(section_center_x, center_y, center_z)
    //     .with_rotation(final_rotation);
    // --- Sticking with simpler X rotation for now, needs testing ---

    // Bounding box approximation
    let bbox = Vec3::new(radius * 2.0, radius * 2.0, LENGTH); // Before slope rotation

    common::spawn_static_shape(
        commands,
        materials,
        level_assets,
        name.to_string(),
        mesh_handle,
        collider, // Collider orientation matches mesh
        transform,
        texture_index,
        bbox,
    );
}
