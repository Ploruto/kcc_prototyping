pub mod common;
pub mod tracks;
pub mod utils;

use bevy::{asset::LoadState, prelude::*};

use tracks::*;
use utils::{DEFAULT_TRACK_SPACING, TextureAssets, TrackOffsets}; // Import resources and constants // Import all track plugins

// --- Plugin Definition ---
pub struct LevelGeneratorPlugin;

impl Plugin for LevelGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app
            // --- Resources ---
            .init_resource::<TrackOffsets>()
            .init_resource::<TextureAssets>()
            // Set default spacing after initialization
            .add_systems(Startup, initialize_track_offsets)
            // --- Asset Loading ---
            // Run asset loading first
            .add_systems(Startup, load_assets_and_setup.pipe(check_asset_loading))
            // --- Add Track Plugins ---
            // These plugins add their own Startup systems that will run *after*
            // load_assets_and_setup and initialize_track_offsets due to ordering
            .add_plugins((
                GroundPlugin,
                StairsTrackPlugin,
                RampsTrackPlugin,
                MovingPlatformsTrackPlugin,
                CrevicesTrackPlugin,
                RidgesTrackPlugin,
                UnevenPatchesTrackPlugin,
                DebrisFieldTrackPlugin,
                NarrowBeamsTrackPlugin,
                HalfHeightObstaclesTrackPlugin,
                AngledWallsTrackPlugin,
                ShapeObstaclesTrackPlugin,
                CapsuleForestTrackPlugin,
                CylinderBridgeTrackPlugin,
                // Add other track plugins here:
                // WallsTrackPlugin,
                // CeilingsTrackPlugin,
            ))
            // --- General Setup ---
            .insert_resource(AmbientLight {
                brightness: 700.0, // Adjust brightness as needed
                ..default()
            });
    }
}

// --- Initialization System ---
fn initialize_track_offsets(mut track_offsets: ResMut<TrackOffsets>) {
    // Explicitly set default spacing if not done via Default impl
    if track_offsets.default_spacing == 0.0 {
        // Check if it's the default f32 value
        track_offsets.default_spacing = DEFAULT_TRACK_SPACING;
        info!(
            "Initialized TrackOffsets default spacing to: {}",
            DEFAULT_TRACK_SPACING
        );
    }
}

// --- Asset Loading System ---
fn load_assets_and_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Starting asset loading...");
    let mut prototype_textures = vec![];
    let colors = ["Dark", "Light", "Green", "Red", "Orange", "Purple"];
    let mut handles_to_check = Vec::new();

    for color in colors {
        for i in 1..=13 {
            let texture_path = format!("textures/{}/texture_{:02}.png", color, i);
            // Basic load, settings applied via Image settings file or default
            let handle: Handle<Image> = asset_server.load(&texture_path);
            prototype_textures.push(handle.clone());
            handles_to_check.push(handle.untyped()); // Track handles for checking load state
        }
    }

    let fallback_material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(150, 140, 130),
        perceptual_roughness: 0.8,
        metallic: 0.1,
        ..default()
    });

    commands.insert_resource(TextureAssets {
        prototype_textures,
        fallback_material: fallback_material_handle,
    });

    // Store handles to check in a temporary resource or pass them differently
    // For simplicity, we'll check in a subsequent system using AssetServer directly
    info!(
        "Asset loading initiated for {} textures.",
        handles_to_check.len()
    );
    // commands.insert_resource(LoadingTextureHandles(handles_to_check));
}

// --- Asset Load Checking System ---
fn check_asset_loading(
    asset_server: Res<AssetServer>,
    texture_assets: Res<TextureAssets>, // Access the handles stored earlier
) {
    info!("Checking asset loading status...");
    let mut all_loaded = true;
    let mut failed_count = 0;

    for handle in &texture_assets.prototype_textures {
        match asset_server.load_state(handle) {
            LoadState::Loaded => { /* Optional: log success */ }
            LoadState::Failed(_) => {
                warn!("Failed to load texture asset: {:?}", handle);
                all_loaded = false;
                failed_count += 1;
            }
            _ => {
                // NotLoaded or Loading
                all_loaded = false;
                // Optional: Log assets still loading
            }
        }
    }

    if all_loaded {
        info!("All prototype textures loaded successfully.");
    } else if failed_count > 0 {
        error!("{} prototype textures failed to load.", failed_count);
        // Potentially panic or handle this error state appropriately
    } else {
        warn!("Some prototype textures are still loading. Level generation might use fallbacks.");
        // You might want a state machine to wait until loading is complete
    }
}
