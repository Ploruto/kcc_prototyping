use avian3d::prelude::{Collider, RigidBody};
use bevy::{math::Affine2, prelude::*};
use std::collections::HashMap;

// Import necessary items from utils.rs (adjust path if needed)
use super::utils::{Geometry, TextureAssets, TrackOffsets, UV_TILE_FACTOR};

// --- Spawning Helpers ---

/// Calculates UV scaling based on object size to maintain texture density.
pub fn calculate_uv_scale(object_size: Vec3, tile_factor: f32) -> Affine2 {
    let mut dims = [
        object_size.x.abs(),
        object_size.y.abs(),
        object_size.z.abs(),
    ];
    dims.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    Affine2::from_scale(Vec2::new(dims[0], dims[1]) / tile_factor)
}

/// Creates a StandardMaterial with specific texture and UV transform.
pub fn create_material_with_uv(
    texture_index: usize,
    object_size: Vec3,
    level_assets: &Res<TextureAssets>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Handle<StandardMaterial> {
    match level_assets.prototype_textures.get(texture_index) {
        Some(texture_handle) => {
            let uv_transform = calculate_uv_scale(object_size, UV_TILE_FACTOR);
            materials.add(StandardMaterial {
                base_color_texture: Some(texture_handle.clone()),
                uv_transform,
                perceptual_roughness: 0.7,
                metallic: 0.1,
                ..default()
            })
        }
        None => {
            if !level_assets.prototype_textures.is_empty() {
                warn!(
                    "Texture index {} out of bounds (max {}). Using fallback.",
                    texture_index,
                    level_assets.prototype_textures.len().saturating_sub(1)
                );
            } else {
                warn!("TextureAssets empty. Using fallback.");
            }
            level_assets.fallback_material.clone()
        }
    }
}

/// Calculates UV scaling based on object size approximation (bounding box).
/// Note: This might not be perfect for complex shapes, but provides a starting point.
pub fn calculate_uv_scale_approx(bounding_box_size: Vec3, tile_factor: f32) -> Affine2 {
    let mut dims = [
        bounding_box_size.x.abs(),
        bounding_box_size.y.abs(),
        bounding_box_size.z.abs(),
    ];
    dims.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    Affine2::from_scale(Vec2::new(dims[0], dims[1]) / tile_factor)
}

/// Creates a StandardMaterial with specific texture and UV transform (using approx scale).
pub fn create_material_with_uv_approx(
    texture_index: usize,
    bounding_box_size: Vec3, // Use bounding box for UV scale approximation
    level_assets: &Res<TextureAssets>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Handle<StandardMaterial> {
    match level_assets.prototype_textures.get(texture_index) {
        Some(texture_handle) => {
            // Use bounding box for approximate UV scaling
            let uv_transform = calculate_uv_scale_approx(bounding_box_size, UV_TILE_FACTOR);
            materials.add(StandardMaterial {
                base_color_texture: Some(texture_handle.clone()),
                uv_transform,
                perceptual_roughness: 0.7,
                metallic: 0.1,
                ..default()
            })
        }
        None => {
            if !level_assets.prototype_textures.is_empty() {
                warn!(
                    "Texture index {} out of bounds (max {}). Using fallback.",
                    texture_index,
                    level_assets.prototype_textures.len().saturating_sub(1)
                );
            } else {
                warn!("TextureAssets empty. Using fallback.");
            }
            level_assets.fallback_material.clone()
        }
    }
}

/// Spawns a basic static cuboid entity.
pub fn spawn_static_cuboid(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    level_assets: &Res<TextureAssets>,
    name: String,
    size: Vec3,
    transform: Transform,
    texture_index: usize,
) -> Entity {
    if size.x <= 0.0 || size.y <= 0.0 || size.z <= 0.0 {
        error!(
            "Spawn static cuboid '{}': non-positive dims {:?}",
            name, size
        );
        return commands.spawn_empty().id();
    }
    let material = create_material_with_uv(texture_index, size, level_assets, materials);
    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::from_size(size))),
            MeshMaterial3d(material.clone()),
            transform,
            RigidBody::Static,
            Collider::cuboid(size.x, size.y, size.z),
            Geometry,
            Name::new(name),
        ))
        .id()
}

/// Spawns a kinematic cuboid entity (useful base for moving platforms).
pub fn spawn_kinematic_cuboid(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    level_assets: &Res<TextureAssets>,
    name: String,
    size: Vec3,
    transform: Transform,
    texture_index: usize,
) -> Entity {
    if size.x <= 0.0 || size.y <= 0.0 || size.z <= 0.0 {
        error!(
            "Spawn kinematic cuboid '{}': non-positive dims {:?}",
            name, size
        );
        return commands.spawn_empty().id();
    }
    let material = create_material_with_uv(texture_index, size, level_assets, materials);
    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::from_size(size))),
            MeshMaterial3d(material.clone()),
            transform,
            RigidBody::Kinematic, // Key difference
            Collider::cuboid(size.x, size.y, size.z),
            Geometry,
            Name::new(name.clone()), // Clone name for Name component
        ))
        .id()
}

/// Spawns a static entity with a specified mesh and collider.
pub fn spawn_static_shape(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    level_assets: &Res<TextureAssets>,
    name: String,
    mesh_handle: Handle<Mesh>, // Use pre-added mesh handle
    collider: Collider,        // Use specified collider
    transform: Transform,
    texture_index: usize,
    bounding_box_size: Vec3, // Provide approx bounding box for UV scaling
) -> Entity {
    let material =
        create_material_with_uv_approx(texture_index, bounding_box_size, level_assets, materials);

    commands
        .spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material),
            transform,
            RigidBody::Static,
            collider, // Use the provided collider
            Geometry,
            Name::new(name),
        ))
        .id()
}

// --- Permutation Generation ---

/// Represents a parameter range for permutation generation.
#[derive(Clone, Debug)]
pub enum Param {
    Float { start: f32, end: f32, step: f32 },
    Int { start: i32, end: i32, step: i32 },
    // Vec3, Bool, etc. could be added if needed
}

/// Generates permutations for a set of parameters and calls a function for each combination.
/// This version passes animation resources optionally via the closure's captured environment
/// or by making the generator function generic over a tuple of resources if needed.
/// For simplicity here, we assume the generator function knows if it needs animation resources.
pub fn generate_permutations<F>(
    params: &[(&str, Param)],
    mut generator_fn: F,
    // Pass necessary Bevy resources that the generator_fn will need
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    track_offsets: &mut ResMut<TrackOffsets>,
    level_assets: &Res<TextureAssets>,
    // Animation resources are passed but only used if the generator needs them
    animation_clips: &mut ResMut<Assets<AnimationClip>>,
    animation_graphs: &mut ResMut<Assets<AnimationGraph>>,
) where
    F: FnMut(
        // Closure takes the current permutation and all necessary resources
        &HashMap<String, f64>, // Current permutation values (f64 for flexibility)
        &mut Commands,
        &mut ResMut<Assets<Mesh>>,
        &mut ResMut<Assets<StandardMaterial>>,
        &mut ResMut<TrackOffsets>,
        &Res<TextureAssets>,
        &mut ResMut<Assets<AnimationClip>>, // Pass animation resources
        &mut ResMut<Assets<AnimationGraph>>,
    ),
{
    if params.is_empty() {
        return;
    }

    // Generate the value lists for each parameter range
    let param_definitions: Vec<(&str, Vec<f64>)> = params
        .iter()
        .map(|(name, p)| {
            let values = match p {
                Param::Float { start, end, step } => {
                    generate_range_f64(*start as f64, *end as f64, *step as f64)
                }
                Param::Int { start, end, step } => {
                    generate_range_i64(*start as i64, *end as i64, *step as i64)
                        .into_iter()
                        .map(|i| i as f64) // Convert integers to f64 for the map
                        .collect()
                }
            };
            (*name, values)
        })
        .filter(|(_, values)| !values.is_empty()) // Filter out ranges that yield no values
        .collect();

    if param_definitions.len() != params.len() {
        warn!("Some parameter ranges generated no values. Permutations might be incomplete.");
        if param_definitions.is_empty() {
            return;
        } // No permutations possible
    }

    let num_params = param_definitions.len();
    let mut indices = vec![0; num_params];

    // Loop through all permutations
    loop {
        // Build the current permutation map (String -> f64)
        let current_permutation: HashMap<String, f64> = param_definitions
            .iter()
            .enumerate()
            .map(|(i, (name, values))| (name.to_string(), values[indices[i]]))
            .collect();

        // Call the provided generator function with the current permutation map and resources
        generator_fn(
            &current_permutation,
            commands,
            meshes,
            materials,
            track_offsets,
            level_assets,
            animation_clips, // Pass animation resources through
            animation_graphs,
        );

        // Increment indices to get the next permutation
        let mut current_param_index = num_params - 1;
        loop {
            indices[current_param_index] += 1;
            // Check if the current parameter's index is within bounds
            if indices[current_param_index] < param_definitions[current_param_index].1.len() {
                break; // Index incremented successfully
            }

            // Reset current index and move to the previous parameter (carry over)
            indices[current_param_index] = 0;
            if current_param_index == 0 {
                return; // All permutations generated
            }
            current_param_index -= 1;
        }
    }
}

// --- Range Generation Helpers (Internal) ---

fn generate_range_f64(start: f64, end: f64, increment: f64) -> Vec<f64> {
    let mut values = Vec::new();
    if increment.abs() < f64::EPSILON {
        if (start - end).abs() < f64::EPSILON {
            values.push(start);
        } else {
            warn!(
                "generate_range_f64 zero increment: start={}, end={}",
                start, end
            );
            values.push(start);
        }
        return values;
    }
    let mut current = start;
    let tolerance = increment.abs() * 0.5;
    if increment > 0.0 {
        while current <= end + tolerance {
            values.push(current);
            current += increment;
        }
    } else {
        while current >= end - tolerance {
            values.push(current);
            current += increment;
        }
    }
    // Ensure start is included if range direction matches increment sign
    if values.is_empty() && (start <= end && increment > 0.0 || start >= end && increment < 0.0) {
        values.push(start);
    }
    values
}

fn generate_range_i64(start: i64, end: i64, increment: i64) -> Vec<i64> {
    if increment == 0 {
        if start == end {
            return vec![start];
        } else {
            warn!(
                "generate_range_i64 zero increment: start={}, end={}",
                start, end
            );
            return vec![start];
        }
    }
    let mut values = Vec::new();
    let mut current = start;
    if increment > 0 {
        while current <= end {
            values.push(current);
            // Check for potential overflow before adding increment
            if let Some(next_val) = current.checked_add(increment) {
                current = next_val;
            } else {
                warn!("Integer overflow detected during range generation.");
                break; // Stop to prevent wrapping around
            }
        }
    } else {
        // increment < 0
        while current >= end {
            values.push(current);
            // Check for potential underflow before adding (subtracting) increment
            if let Some(next_val) = current.checked_add(increment) {
                current = next_val;
            } else {
                warn!("Integer underflow detected during range generation.");
                break; // Stop to prevent wrapping around
            }
        }
    }
    // Ensure start is included if range direction matches increment sign
    if values.is_empty() && (start <= end && increment > 0 || start >= end && increment < 0) {
        values.push(start);
    }
    values
}
