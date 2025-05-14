// Declare the modules for each track plugin

pub mod angled_walls;
pub mod capsule_forest;
pub mod crevices;
pub mod cylinder_bridge;
pub mod debris_field;
pub mod ground;
pub mod half_height_obstacles;
pub mod moving_platforms;
pub mod narrow_beams;
pub mod ramps;
pub mod ridges;
pub mod shape_obstacles;
pub mod stairs;
pub mod uneven_patches;

// Re-export the plugins for easier use in level/mod.rs
pub use angled_walls::AngledWallsTrackPlugin;
pub use capsule_forest::CapsuleForestTrackPlugin;
pub use crevices::CrevicesTrackPlugin;
pub use cylinder_bridge::CylinderBridgeTrackPlugin;
pub use debris_field::DebrisFieldTrackPlugin;
pub use ground::GroundPlugin;
pub use half_height_obstacles::HalfHeightObstaclesTrackPlugin;
pub use moving_platforms::MovingPlatformsTrackPlugin;
pub use narrow_beams::NarrowBeamsTrackPlugin;
pub use ramps::RampsTrackPlugin;
pub use ridges::RidgesTrackPlugin;
pub use shape_obstacles::ShapeObstaclesTrackPlugin;
pub use stairs::StairsTrackPlugin;
pub use uneven_patches::UnevenPatchesTrackPlugin;
