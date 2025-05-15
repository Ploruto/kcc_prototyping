// src/level/plugins/ground.rs

use crate::level::{
    common,
    load_assets_and_setup,
    utils::{BASE_Y, TextureAssets}, // Use resources and constants
};
use bevy::prelude::*;

// --- Plugin Definition ---
pub struct GroundPlugin;

impl Plugin for GroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ground.after(load_assets_and_setup)); // Ensure assets are loaded
    }
}

// --- Constants ---
const GROUND_SIZE: Vec3 = Vec3::new(400.0, 1.0, 400.0); // Match the size needed
const GROUND_POS: Vec3 = Vec3::new(0.0, BASE_Y - GROUND_SIZE.y / 2.0, 0.0);
const TEX_GROUND: usize = 7; // Example texture index

// --- System ---
fn setup_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level_assets: Res<TextureAssets>,
) {
    info!("Setting up ground plane...");
    common::spawn_static_cuboid(
        &mut commands,
        &mut meshes,
        &mut materials,
        &level_assets,
        "Ground".to_string(),
        GROUND_SIZE,
        Transform::from_translation(GROUND_POS),
        TEX_GROUND,
    );
}
