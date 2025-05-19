use crate::level::{
    common,
    utils::{BASE_Y, TextureAssets, TrackOffsets},
};
use avian3d::prelude::Collider; // Use Collider enum directly
use bevy::prelude::*;

// --- Plugin Definition ---
pub struct ShapeObstaclesTrackPlugin;

impl Plugin for ShapeObstaclesTrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_shape_obstacles_track.after(super::super::load_assets_and_setup),
        );
    }
}

// --- Constants ---
const TRACK_NAME: &str = "ShapeObstacles";
const TRACK_Z: f32 = -100.0; // Place this track
const TEX_OBSTACLE: usize = 2 * 13; // Example texture

// Define the sequence of shapes and their parameters
// Format: (Name, Mesh, Collider, BoundingBox Size, Y Offset from Base)
// Y Offset helps place the *base* of the shape near BASE_Y
type ShapeDefinition = (
    &'static str,                          // Base name part
    fn(&mut Assets<Mesh>) -> Handle<Mesh>, // Function to create mesh
    Collider,                              // Collider definition
    Vec3,                                  // Approx bounding box for UVs
    f32,                                   // Y offset for base placement
);

/// Function to generate the sequence of shape definitions.
/// This avoids issues with static/const initialization of non-const functions like Collider::sphere.
fn get_shape_sequence() -> Vec<ShapeDefinition> {
    vec![
        // Sphere
        (
            "Sphere",
            |meshes: &mut Assets<Mesh>| meshes.add(Sphere::new(0.8).mesh().uv(32, 18)), // Mesh creation fn
            Collider::sphere(0.8), // Create collider instance here
            Vec3::splat(1.6),      // Bbox approx diameter
            0.8,                   // Offset by radius
        ),
        // Capsule (Vertical)
        (
            "CapsuleV",
            |meshes: &mut Assets<Mesh>| meshes.add(Capsule3d::new(0.6, 1.5 * 2.0)), // Bevy Capsule takes radius, full height
            Collider::capsule(0.6, 1.5 * 2.0), // Avian Capsule takes radius, full height
            Vec3::new(1.2, 1.5 * 2.0 + 1.2, 1.2), // Bbox approx (diameter, total height, diameter)
            0.6 + 1.5,                         // Offset by radius + half_height (center of capsule)
        ),
        // Cylinder (Vertical)
        (
            "CylinderV",
            |meshes: &mut Assets<Mesh>| meshes.add(Cylinder::new(0.7, 2.0)), // Radius, height
            Collider::cylinder(0.7, 2.0),                                    // Radius, height
            Vec3::new(1.4, 2.0, 1.4),                                        // Bbox approx
            1.0, // Offset by half_height
        ),
        // Cone (Vertical)
        (
            "ConeV",
            |meshes: &mut Assets<Mesh>| meshes.add(Cone::new(0.9, 2.2)), // Radius, height
            Collider::cone(0.9, 2.2),                                    // Radius, height
            Vec3::new(1.8, 2.2, 1.8),                                    // Bbox approx
            1.1,                                                         // Offset by half_height
        ),
    ]
}

// --- Setup System ---
fn setup_shape_obstacles_track(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut track_offsets: ResMut<TrackOffsets>,
    level_assets: Res<TextureAssets>,
    // These are unused but required by generate_permutations signature
    mut _animation_clips: ResMut<Assets<AnimationClip>>,
    mut _animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    info!("Generating track: {}", TRACK_NAME);

    // No permutations needed, just iterate through the fixed sequence
    for (i, (base_name, mesh_fn, collider, bbox, y_offset)) in
        get_shape_sequence().iter().enumerate()
    {
        let name = format!("{}_{}", base_name, i);
        // Footprint is based on the bounding box X dimension
        let footprint_x = bbox.x;
        let section_center_x = track_offsets.get_and_advance(TRACK_NAME, footprint_x);

        // Create the specific mesh instance for this shape
        let mesh_handle = mesh_fn(&mut meshes);

        // Calculate position
        let transform = Transform::from_xyz(
            section_center_x,
            BASE_Y + y_offset, // Place base near ground
            TRACK_Z,
        );

        // Spawn using the generic shape spawner
        common::spawn_static_shape(
            &mut commands,
            &mut materials,
            &level_assets,
            name,
            mesh_handle,
            collider.clone(), // Clone collider definition
            transform,
            TEX_OBSTACLE,
            *bbox, // Pass bounding box
        );
    }
}
