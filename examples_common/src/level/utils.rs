use bevy::prelude::*;
use std::collections::HashMap;

// --- Core Configuration ---
pub const BASE_Y: f32 = 0.0;
pub const UV_TILE_FACTOR: f32 = 5.0;
pub const DEFAULT_TRACK_SPACING: f32 = 5.0; // Spacing between elements *within* a track
pub const DEFAULT_TRACK_START_X: f32 = -90.0; // Common starting X for all tracks

/// Resource to track the next available X-coordinate *per track*.
#[derive(Resource, Debug)]
pub struct TrackOffsets {
    /// Key: Track Name (String), Value: Next available X coordinate for that track
    pub offsets: HashMap<String, f32>,
    /// Default spacing added after each element within a track
    pub default_spacing: f32,
}

// Implement Default manually to set default_spacing
impl Default for TrackOffsets {
    fn default() -> Self {
        Self {
            offsets: HashMap::new(),
            default_spacing: DEFAULT_TRACK_SPACING,
        }
    }
}

impl TrackOffsets {
    /// Gets the current placement X for a track and advances the offset
    /// for the next element on that *same* track.
    /// Initializes the track if it doesn't exist.
    pub fn get_and_advance(&mut self, track_name: &str, element_footprint_x: f32) -> f32 {
        let current_x_ref = self
            .offsets
            .entry(track_name.to_string())
            .or_insert(DEFAULT_TRACK_START_X);

        let placement_x = *current_x_ref;

        *current_x_ref += element_footprint_x.max(1.0) + self.default_spacing;

        placement_x
    }
}

/// Resource holding handles to loaded prototype textures and fallback material.
#[derive(Resource, Default)]
pub struct TextureAssets {
    pub prototype_textures: Vec<Handle<Image>>,
    pub fallback_material: Handle<StandardMaterial>,
}

// --- Components ---

/// Marker component for level geometry entities.
#[derive(Component)]
pub struct Geometry;
