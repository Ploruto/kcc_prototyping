use crate::level::{
    common::{self, Param, spawn_static_cuboid},
    utils::{BASE_Y, TextureAssets, TrackOffsets},
};
use bevy::{
    animation::{AnimationTarget, AnimationTargetId, animated_field},
    prelude::*,
};
use std::{collections::HashMap, f32::consts::PI};

// --- Plugin Definition ---
pub struct MovingPlatformsTrackPlugin;

impl Plugin for MovingPlatformsTrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_moving_platforms_track.after(super::super::load_assets_and_setup),
        );
    }
}

// --- Constants ---
const TRACK_NAME: &str = "MovingPlatforms";
const TRACK_Z: f32 = 20.0;
const TEX_PLATFORM: usize = 4 * 13 + 3;
const TEX_OBSTACLE_WALL: usize = 3 * 13;

// --- Parameter Ranges (Simplified for fewer instances by default) ---
// Vertical (1 instance)
const V_PARAMS: &[(&str, Param)] = &[
    (
        "v_dist",
        Param::Float {
            start: 4.0,
            end: 10.0,
            step: 6.0,
        },
    ),
    (
        "v_dur",
        Param::Float {
            start: 1.0,
            end: 3.0,
            step: 2.0,
        },
    ),
];
const V_PLAT_SIZE: Vec3 = Vec3::new(3.0, 0.3, 3.0);

// Horizontal (1 instance)
const H_PARAMS: &[(&str, Param)] = &[
    (
        "h_dist",
        Param::Float {
            start: 6.0,
            end: 6.0,
            step: 1.0,
        },
    ),
    (
        "h_dur",
        Param::Float {
            start: 1.5,
            end: 1.5,
            step: 2.0,
        },
    ),
];
const H_PLAT_SIZE: Vec3 = Vec3::new(4.0, 0.3, 2.5);

// Rotating (1 instance, uses UnevenSampleAutoCurve for full 360 loop)
const R_PARAMS: &[(&str, Param)] = &[(
    "r_cycle_dur",
    Param::Float {
        start: 2.5,
        end: 2.5,
        step: 2.0,
    },
)];
const R_PLAT_SIZE: Vec3 = Vec3::new(3.5, 0.3, 3.5);

// Translate & Rotate (TR) Platform (1 instance)
const TR_PARAMS: &[(&str, Param)] = &[
    (
        "tr_dist_x",
        Param::Float {
            start: 7.0,
            end: 7.0,
            step: 1.0,
        },
    ),
    (
        "tr_trans_dur",
        Param::Float {
            start: 2.0,
            end: 2.0,
            step: 1.0,
        },
    ),
    (
        "tr_rot_cycle_dur",
        Param::Float {
            start: 2.0,
            end: 2.0,
            step: 1.0,
        },
    ),
];
const TR_PLAT_SIZE: Vec3 = Vec3::new(3.0, 0.3, 3.0);

// Crash Test (CT) Platform (1 instance)
const CT_PARAMS: &[(&str, Param)] = &[
    (
        "ct_dist_to_wall",
        Param::Float {
            start: -3.0,
            end: 0.0,
            step: 3.0,
        },
    ),
    (
        "ct_move_dur",
        Param::Float {
            start: 1.0,
            end: 1.0,
            step: 1.0,
        },
    ),
];
const CT_PLAT_SIZE: Vec3 = Vec3::new(2.5, 0.3, 4.0);
const CT_WALL_SIZE: Vec3 = Vec3::new(CT_PLAT_SIZE.x + 1.0, 3.0, 0.5);

// --- Setup System ---
fn setup_moving_platforms_track(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut track_offsets: ResMut<TrackOffsets>,
    level_assets: Res<TextureAssets>,
    mut animation_clips: ResMut<Assets<AnimationClip>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    info!("Generating track: {}", TRACK_NAME);

    // --- Vertical Platforms ---
    let vertical_generator =
        |permutation: &HashMap<String, f64>,
         cmds: &mut Commands,
         mshs: &mut ResMut<Assets<Mesh>>,
         mats: &mut ResMut<Assets<StandardMaterial>>,
         offsets: &mut ResMut<TrackOffsets>,
         assets: &Res<TextureAssets>,
         clips: &mut ResMut<Assets<AnimationClip>>,
         graphs: &mut ResMut<Assets<AnimationGraph>>| {
            let v_dist = permutation["v_dist"] as f32;
            let v_dur = permutation["v_dur"] as f32;
            let name = format!("VPlatform_d{:.1}_t{:.1}", v_dist, v_dur);
            spawn_moving_platform_vertical_instance(
                cmds,
                mshs,
                mats,
                offsets,
                assets,
                clips,
                graphs,
                &name,
                V_PLAT_SIZE,
                v_dist,
                v_dur,
                TEX_PLATFORM,
            );
        };
    common::generate_permutations(
        V_PARAMS,
        vertical_generator,
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut track_offsets,
        &level_assets,
        &mut animation_clips,
        &mut animation_graphs,
    );

    // --- Horizontal Platforms ---
    let horizontal_generator =
        |permutation: &HashMap<String, f64>,
         cmds: &mut Commands,
         mshs: &mut ResMut<Assets<Mesh>>,
         mats: &mut ResMut<Assets<StandardMaterial>>,
         offsets: &mut ResMut<TrackOffsets>,
         assets: &Res<TextureAssets>,
         clips: &mut ResMut<Assets<AnimationClip>>,
         graphs: &mut ResMut<Assets<AnimationGraph>>| {
            let h_dist = permutation["h_dist"] as f32;
            let h_dur = permutation["h_dur"] as f32;
            let name = format!("HPlatform_d{:.1}_t{:.1}", h_dist, h_dur);
            spawn_moving_platform_horizontal_instance(
                cmds,
                mshs,
                mats,
                offsets,
                assets,
                clips,
                graphs,
                &name,
                H_PLAT_SIZE,
                h_dist,
                h_dur,
                TEX_PLATFORM,
            );
        };
    common::generate_permutations(
        H_PARAMS,
        horizontal_generator,
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut track_offsets,
        &level_assets,
        &mut animation_clips,
        &mut animation_graphs,
    );

    // --- Rotating Platforms ---
    let rotating_generator =
        |permutation: &HashMap<String, f64>,
         cmds: &mut Commands,
         mshs: &mut ResMut<Assets<Mesh>>,
         mats: &mut ResMut<Assets<StandardMaterial>>,
         offsets: &mut ResMut<TrackOffsets>,
         assets: &Res<TextureAssets>,
         clips: &mut ResMut<Assets<AnimationClip>>,
         graphs: &mut ResMut<Assets<AnimationGraph>>| {
            let r_cycle_dur = permutation["r_cycle_dur"] as f32;
            let name = format!("RPlatform_t{:.1}", r_cycle_dur);
            spawn_moving_platform_rotating_instance(
                cmds,
                mshs,
                mats,
                offsets,
                assets,
                clips,
                graphs,
                &name,
                R_PLAT_SIZE,
                r_cycle_dur,
                TEX_PLATFORM,
            );
        };
    common::generate_permutations(
        R_PARAMS,
        rotating_generator,
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut track_offsets,
        &level_assets,
        &mut animation_clips,
        &mut animation_graphs,
    );

    // --- Translate & Rotate Platforms ---
    let tr_generator = |permutation: &HashMap<String, f64>,
                        cmds: &mut Commands,
                        mshs: &mut ResMut<Assets<Mesh>>,
                        mats: &mut ResMut<Assets<StandardMaterial>>,
                        offsets: &mut ResMut<TrackOffsets>,
                        assets: &Res<TextureAssets>,
                        clips: &mut ResMut<Assets<AnimationClip>>,
                        graphs: &mut ResMut<Assets<AnimationGraph>>| {
        let tr_dist_x = permutation["tr_dist_x"] as f32;
        let tr_trans_dur = permutation["tr_trans_dur"] as f32;
        let tr_rot_cycle_dur = permutation["tr_rot_cycle_dur"] as f32;
        let name = format!(
            "TRPlatform_dx{:.1}_td{:.1}_rd{:.1}",
            tr_dist_x, tr_trans_dur, tr_rot_cycle_dur
        );
        spawn_platform_translate_rotate_instance(
            cmds,
            mshs,
            mats,
            offsets,
            assets,
            clips,
            graphs,
            &name,
            TR_PLAT_SIZE,
            tr_dist_x,
            tr_trans_dur,
            tr_rot_cycle_dur,
            TEX_PLATFORM,
        );
    };
    common::generate_permutations(
        TR_PARAMS,
        tr_generator,
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut track_offsets,
        &level_assets,
        &mut animation_clips,
        &mut animation_graphs,
    );

    // --- Crash Test Platforms ---
    let ct_generator = |permutation: &HashMap<String, f64>,
                        cmds: &mut Commands,
                        mshs: &mut ResMut<Assets<Mesh>>,
                        mats: &mut ResMut<Assets<StandardMaterial>>,
                        offsets: &mut ResMut<TrackOffsets>,
                        assets: &Res<TextureAssets>,
                        clips: &mut ResMut<Assets<AnimationClip>>,
                        graphs: &mut ResMut<Assets<AnimationGraph>>| {
        let ct_dist_to_wall = permutation["ct_dist_to_wall"] as f32;
        let ct_move_dur = permutation["ct_move_dur"] as f32;
        let name = format!("CrashTest_d{:.1}_t{:.1}", ct_dist_to_wall, ct_move_dur);
        spawn_platform_crash_test_instance(
            cmds,
            mshs,
            mats,
            offsets,
            assets,
            clips,
            graphs,
            &name,
            CT_PLAT_SIZE,
            ct_dist_to_wall,
            ct_move_dur,
            TEX_PLATFORM,
        );
    };
    common::generate_permutations(
        CT_PARAMS,
        ct_generator,
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut track_offsets,
        &level_assets,
        &mut animation_clips,
        &mut animation_graphs,
    );
}

// --- Instance Spawners ---

fn spawn_moving_platform_vertical_instance(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    track_offsets: &mut ResMut<TrackOffsets>,
    level_assets: &Res<TextureAssets>,
    animation_clips: &mut ResMut<Assets<AnimationClip>>,
    animation_graphs: &mut ResMut<Assets<AnimationGraph>>,
    name: &str,
    size: Vec3,
    vertical_distance: f32,
    animation_duration_one_way: f32,
    texture_index: usize,
) {
    let section_center_x = track_offsets.get_and_advance(TRACK_NAME, size.x);
    let platform_start_pos = Vec3::new(section_center_x, BASE_Y + size.y / 2.0, TRACK_Z);
    let platform_end_pos = platform_start_pos + Vec3::Y * vertical_distance;
    let platform_name_component = Name::new(name.to_string());
    let target_id = AnimationTargetId::from_name(&platform_name_component);

    let mut clip = AnimationClip::default();
    if let Ok(interval) = Interval::new(0.0, animation_duration_one_way) {
        if let Ok(curve) = EasingCurve::new(
            platform_start_pos,
            platform_end_pos,
            EaseFunction::SineInOut,
        )
        .reparametrize_linear(interval)
        {
            if let Ok(ping_pong_curve) = curve.ping_pong() {
                clip.add_curve_to_target(
                    target_id,
                    AnimatableCurve::new(animated_field!(Transform::translation), ping_pong_curve),
                );
            } else {
                warn!("VPlatform: Failed ping-pong for translation in {}", name);
            }
        } else {
            warn!(
                "VPlatform: Failed reparametrize for translation in {}",
                name
            );
        }
    } else {
        warn!(
            "VPlatform: Failed interval creation for translation in {}",
            name
        );
    }

    let clip_handle = animation_clips.add(clip);
    let (graph, node_index) = AnimationGraph::from_clip(clip_handle);
    let graph_handle = animation_graphs.add(graph);
    let mut player = AnimationPlayer::default();
    player.play(node_index).repeat();

    let platform_entity = common::spawn_kinematic_cuboid(
        commands,
        meshes,
        materials,
        level_assets,
        name.to_string(),
        size,
        Transform::from_translation(platform_start_pos),
        texture_index,
    );
    commands.entity(platform_entity).insert((
        platform_name_component,
        AnimationGraphHandle(graph_handle),
        player,
        AnimationTarget {
            id: target_id,
            player: platform_entity,
        },
    ));
}

fn spawn_moving_platform_horizontal_instance(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    track_offsets: &mut ResMut<TrackOffsets>,
    level_assets: &Res<TextureAssets>,
    animation_clips: &mut ResMut<Assets<AnimationClip>>,
    animation_graphs: &mut ResMut<Assets<AnimationGraph>>,
    name: &str,
    size: Vec3,
    horizontal_distance: f32,
    animation_duration_one_way: f32,
    texture_index: usize,
) {
    let footprint_x = size.x + horizontal_distance;
    let section_center_x = track_offsets.get_and_advance(TRACK_NAME, footprint_x);
    let platform_start_pos = Vec3::new(
        section_center_x - horizontal_distance / 2.0,
        BASE_Y + size.y / 2.0,
        TRACK_Z,
    );
    let platform_end_pos = platform_start_pos + Vec3::X * horizontal_distance;
    let platform_name_component = Name::new(name.to_string());
    let target_id = AnimationTargetId::from_name(&platform_name_component);

    let mut clip = AnimationClip::default();
    if let Ok(interval) = Interval::new(0.0, animation_duration_one_way) {
        if let Ok(curve) = EasingCurve::new(
            platform_start_pos,
            platform_end_pos,
            EaseFunction::SineInOut,
        )
        .reparametrize_linear(interval)
        {
            if let Ok(ping_pong_curve) = curve.ping_pong() {
                clip.add_curve_to_target(
                    target_id,
                    AnimatableCurve::new(animated_field!(Transform::translation), ping_pong_curve),
                );
            } else {
                warn!("HPlatform: Failed ping-pong for translation in {}", name);
            }
        } else {
            warn!(
                "HPlatform: Failed reparametrize for translation in {}",
                name
            );
        }
    } else {
        warn!(
            "HPlatform: Failed interval creation for translation in {}",
            name
        );
    }

    let clip_handle = animation_clips.add(clip);
    let (graph, node_index) = AnimationGraph::from_clip(clip_handle);
    let graph_handle = animation_graphs.add(graph);
    let mut player = AnimationPlayer::default();
    player.play(node_index).repeat();

    let platform_entity = common::spawn_kinematic_cuboid(
        commands,
        meshes,
        materials,
        level_assets,
        name.to_string(),
        size,
        Transform::from_translation(platform_start_pos),
        texture_index,
    );
    commands.entity(platform_entity).insert((
        platform_name_component,
        AnimationGraphHandle(graph_handle),
        player,
        AnimationTarget {
            id: target_id,
            player: platform_entity,
        },
    ));
}

fn spawn_moving_platform_rotating_instance(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    track_offsets: &mut ResMut<TrackOffsets>,
    level_assets: &Res<TextureAssets>,
    animation_clips: &mut ResMut<Assets<AnimationClip>>,
    animation_graphs: &mut ResMut<Assets<AnimationGraph>>,
    name: &str,
    size: Vec3,
    animation_duration_cycle: f32,
    texture_index: usize,
) {
    let section_center_x = track_offsets.get_and_advance(TRACK_NAME, size.x);
    let platform_pos = Vec3::new(section_center_x, BASE_Y + size.y / 2.0, TRACK_Z);
    let platform_name_component = Name::new(name.to_string());
    let target_id = AnimationTargetId::from_name(&platform_name_component);

    let mut clip = AnimationClip::default();

    // --- Rotation Curve using UnevenSampleAutoCurve for full 360 loop ---
    // Define keyframes for a full 360-degree rotation around Y
    let times = vec![
        0.0,                             // Start
        animation_duration_cycle * 0.25, // 90 degrees
        animation_duration_cycle * 0.5,  // 180 degrees
        animation_duration_cycle * 0.75, // 270 degrees
        animation_duration_cycle,        // 360 degrees (back to start)
    ];
    let rotations = vec![
        Quat::IDENTITY,
        Quat::from_rotation_y(PI * 0.5),
        Quat::from_rotation_y(PI),
        Quat::from_rotation_y(PI * 1.5),
        Quat::from_rotation_y(PI * 2.0), // Numerically same as Quat::IDENTITY for seamless loop
    ];

    if let Ok(rotation_curve) = UnevenSampleAutoCurve::new(times.into_iter().zip(rotations)) {
        clip.add_curve_to_target(
            target_id,
            AnimatableCurve::new(animated_field!(Transform::rotation), rotation_curve),
        );
    } else {
        warn!("RPlatform: Failed to build rotation curve for {}", name);
    }

    let clip_handle = animation_clips.add(clip);
    let (graph, node_index) = AnimationGraph::from_clip(clip_handle);
    let graph_handle = animation_graphs.add(graph);
    let mut player = AnimationPlayer::default();
    player.play(node_index).repeat();

    let platform_entity = common::spawn_kinematic_cuboid(
        commands,
        meshes,
        materials,
        level_assets,
        name.to_string(),
        size,
        Transform::from_translation(platform_pos),
        texture_index,
    );
    commands.entity(platform_entity).insert((
        platform_name_component,
        AnimationGraphHandle(graph_handle),
        player,
        AnimationTarget {
            id: target_id,
            player: platform_entity,
        },
    ));
}

/// Spawns a platform that translates horizontally and rotates around its Y-axis simultaneously.
fn spawn_platform_translate_rotate_instance(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    track_offsets: &mut ResMut<TrackOffsets>,
    level_assets: &Res<TextureAssets>,
    animation_clips: &mut ResMut<Assets<AnimationClip>>,
    animation_graphs: &mut ResMut<Assets<AnimationGraph>>,
    name: &str,
    size: Vec3,
    translate_dist_x: f32,
    translate_duration_one_way: f32,
    rotate_cycle_duration: f32,
    texture_index: usize,
) {
    let footprint_x = size.x + translate_dist_x;
    let section_center_x = track_offsets.get_and_advance(TRACK_NAME, footprint_x);

    let platform_start_pos = Vec3::new(
        section_center_x - translate_dist_x / 2.0,
        BASE_Y + size.y / 2.0,
        TRACK_Z,
    );
    let platform_end_pos = platform_start_pos + Vec3::X * translate_dist_x;

    let platform_name_component = Name::new(name.to_string());
    let target_id = AnimationTargetId::from_name(&platform_name_component);

    let mut clip = AnimationClip::default();

    // --- Translation Curve (Ping-Pong) ---
    if let Ok(interval) = Interval::new(0.0, translate_duration_one_way) {
        if let Ok(trans_curve) = EasingCurve::new(
            platform_start_pos,
            platform_end_pos,
            EaseFunction::SineInOut,
        )
        .reparametrize_linear(interval)
        {
            if let Ok(ping_pong_trans_curve) = trans_curve.ping_pong() {
                clip.add_curve_to_target(
                    target_id,
                    AnimatableCurve::new(
                        animated_field!(Transform::translation),
                        ping_pong_trans_curve,
                    ),
                );
            } else {
                warn!("TRPlatform: Failed ping-pong for translation in {}", name);
            }
        } else {
            warn!(
                "TRPlatform: Failed reparametrize for translation in {}",
                name
            );
        }
    } else {
        warn!(
            "TRPlatform: Failed interval creation for translation in {}",
            name
        );
    }

    // --- Rotation Curve (Continuous 360 Loop using UnevenSampleAutoCurve) ---
    let rot_times = vec![
        0.0,
        rotate_cycle_duration * 0.25,
        rotate_cycle_duration * 0.5,
        rotate_cycle_duration * 0.75,
        rotate_cycle_duration,
    ];
    let rot_rotations = vec![
        Quat::IDENTITY,
        Quat::from_rotation_y(PI * 0.5),
        Quat::from_rotation_y(PI),
        Quat::from_rotation_y(PI * 1.5),
        Quat::from_rotation_y(PI * 2.0),
    ];
    if let Ok(rotation_curve) = UnevenSampleAutoCurve::new(rot_times.into_iter().zip(rot_rotations))
    {
        clip.add_curve_to_target(
            target_id,
            AnimatableCurve::new(animated_field!(Transform::rotation), rotation_curve),
        );
    } else {
        warn!("TRPlatform: Failed to build rotation curve for {}", name);
    }

    let clip_handle = animation_clips.add(clip);
    let (graph, node_index) = AnimationGraph::from_clip(clip_handle);
    let graph_handle = animation_graphs.add(graph);
    let mut player = AnimationPlayer::default();
    player.play(node_index).repeat();

    let platform_entity = common::spawn_kinematic_cuboid(
        commands,
        meshes,
        materials,
        level_assets,
        name.to_string(),
        size,
        Transform::from_translation(platform_start_pos),
        texture_index,
    );
    commands.entity(platform_entity).insert((
        platform_name_component,
        AnimationGraphHandle(graph_handle),
        player,
        AnimationTarget {
            id: target_id,
            player: platform_entity,
        },
    ));
}

/// Spawns a platform moving towards a static wall.
fn spawn_platform_crash_test_instance(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    track_offsets: &mut ResMut<TrackOffsets>,
    level_assets: &Res<TextureAssets>,
    animation_clips: &mut ResMut<Assets<AnimationClip>>,
    animation_graphs: &mut ResMut<Assets<AnimationGraph>>,
    name: &str,
    platform_size: Vec3,
    distance_to_wall: f32,
    move_duration: f32,
    texture_index: usize,
) {
    let total_section_width = platform_size.x + distance_to_wall + CT_WALL_SIZE.x;
    let section_center_x = track_offsets.get_and_advance(TRACK_NAME, total_section_width);

    let wall_pos_x = section_center_x - CT_WALL_SIZE.x / 2.0;
    let wall_pos = Vec3::new(wall_pos_x, BASE_Y + CT_WALL_SIZE.y / 2.0, TRACK_Z);

    spawn_static_cuboid(
        commands,
        meshes,
        materials,
        level_assets,
        format!("{}_Wall", name),
        CT_WALL_SIZE,
        Transform::from_translation(wall_pos),
        TEX_OBSTACLE_WALL,
    );

    let platform_start_x = section_center_x - total_section_width / 2.0 + platform_size.x / 2.0;
    let platform_start_pos = Vec3::new(platform_start_x, BASE_Y + platform_size.y / 2.0, TRACK_Z);

    let platform_end_target_x = wall_pos_x - CT_WALL_SIZE.x / 2.0 - platform_size.x / 2.0 + 0.1;
    let platform_end_pos = Vec3::new(
        platform_end_target_x,
        platform_start_pos.y,
        platform_start_pos.z,
    );

    let platform_name_component = Name::new(name.to_string());
    let target_id = AnimationTargetId::from_name(&platform_name_component);

    let mut clip = AnimationClip::default();
    if let Ok(interval) = Interval::new(0.0, move_duration) {
        if let Ok(curve) =
            EasingCurve::new(platform_start_pos, platform_end_pos, EaseFunction::Linear)
                .reparametrize_linear(interval)
        {
            if let Ok(ping_pong_curve) = curve.ping_pong() {
                clip.add_curve_to_target(
                    target_id,
                    AnimatableCurve::new(animated_field!(Transform::translation), ping_pong_curve),
                );
            } else {
                warn!("CrashTest: Failed ping-pong for {}", name);
            }
        } else {
            warn!("CrashTest: Failed reparametrize for {}", name);
        }
    } else {
        warn!("CrashTest: Failed interval creation for {}", name);
    }

    let clip_handle = animation_clips.add(clip);
    let (graph, node_index) = AnimationGraph::from_clip(clip_handle);
    let graph_handle = animation_graphs.add(graph);
    let mut player = AnimationPlayer::default();
    player.play(node_index).repeat();

    let platform_entity = common::spawn_kinematic_cuboid(
        commands,
        meshes,
        materials,
        level_assets,
        name.to_string(),
        platform_size,
        Transform::from_translation(platform_start_pos),
        texture_index,
    );
    commands.entity(platform_entity).insert((
        platform_name_component,
        AnimationGraphHandle(graph_handle),
        player,
        AnimationTarget {
            id: target_id,
            player: platform_entity,
        },
    ));
}
