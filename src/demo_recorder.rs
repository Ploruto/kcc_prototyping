use std::{collections::HashMap, fs::File, io::Write, iter::Map};

use avian3d::prelude::*;
use bevy::{ecs::system::command::insert_resource, prelude::*, tasks::AsyncComputeTaskPool};
use bevy_enhanced_input::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    input::DefaultContext,
    movement::{Character, Frozen},
};

// Input part

#[derive(Debug, Clone, Copy, InputAction)]
#[input_action(output = bool)]
// F5
struct RecordDemo;

#[derive(Debug, Clone, Copy, InputAction)]
#[input_action(output = bool)]
// F6
struct SaveDemo;

fn bind_default_context_actions(
    trigger: Trigger<OnAdd, Actions<DefaultContext>>,
    mut players: Query<&mut Actions<DefaultContext>>,
) {
    // I have no clue how this crate works, and it is 10:10pm.
    if let Ok(mut actions) = players.get_mut(trigger.target()) {
        info!(
            "Binding DefaultContext actions for entity {:?}",
            trigger.target()
        );
        actions.bind::<RecordDemo>().to(KeyCode::F5);
        actions.bind::<SaveDemo>().to(KeyCode::F6);
    } else {
        warn!(
            "Failed to get Actions<DefaultContext> for entity {:?} during binding",
            trigger.target()
        );
    }
}

const FRAME_TIME: f32 = 1. / 30.;
#[derive(Resource)]
struct RecorderTimer(Timer);
#[derive(Deserialize, Serialize)]
struct Snapshot {
    velocity: Vec3,
    transform: Transform,
}
#[derive(Deserialize, Serialize)]
struct Demo {
    frame_time: f32,
    snapshots: Vec<Snapshot>,
}
impl Default for Demo {
    fn default() -> Self {
        Self {
            frame_time: FRAME_TIME,
            snapshots: vec![],
        }
    }
}
// I geniuenly do no not know if it is overkill.
// This should give us the option to record multiple entities.
#[derive(Resource, Default)]
struct DemoHandler {
    demos: HashMap<Entity, Demo>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, States)]
enum RecorderState {
    #[default]
    Stopped,
    Saving,
    Recording,
}

pub struct DemoPlugin;

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RecorderTimer(Timer::from_seconds(
            0.33,
            TimerMode::Repeating,
        )))
        .init_state::<RecorderState>()
        .insert_resource(DemoHandler::default())
        .add_systems(
            Update,
            record_demo.run_if(in_state(RecorderState::Recording)),
        )
        .add_systems(FixedLast, save_demo.run_if(in_state(RecorderState::Saving)))
        .add_observer(bind_default_context_actions)
        .add_observer(save_input)
        .add_observer(record_input);
    }
}

fn save_input(
    _trigger: Trigger<Completed<SaveDemo>>, // Triggered by DefaultContext action
    // I use states. They are nice :)
    state: Res<State<RecorderState>>,
    mut next_state: ResMut<NextState<RecorderState>>,
) {
    match state.get() {
        RecorderState::Recording => next_state.set(RecorderState::Saving),
        _ => (),
    }
}
fn record_input(
    _trigger: Trigger<Completed<RecordDemo>>, // Triggered by DefaultContext action
    // I use states. They are nice :)
    state: Res<State<RecorderState>>, 
    mut next_state: ResMut<NextState<RecorderState>>,
) {
    match state.get() {
        RecorderState::Stopped => next_state.set(RecorderState::Recording),
        _ => (),
    }
}
const DEMOS_FOLDER: &'static str = "./recordings/";
// Here I save the Snapshots for each entity in their own file.
// This is async, so it does not lag the game.
// I take the DemoHandler instance, but it should not be a problem, 
// because it will be a new one anyways.
fn save_demo(mut demo_handler: ResMut<DemoHandler>, state: Res<State<RecorderState>>, mut next_state: ResMut<NextState<RecorderState>>) {
    
    let task_pool = AsyncComputeTaskPool::get();

    let demo_handler_taken = std::mem::take(&mut *demo_handler);

    task_pool.spawn(async move {
        for (entity, demo) in demo_handler_taken.demos {
            let file_name = format!("{}demo_{}.ron",DEMOS_FOLDER, entity.to_string());
            if let Ok(mut file) = File::create(file_name) {
                if let Ok(serialized) = ron::to_string(&demo) {
                    let _ = file.write_all(serialized.as_bytes());
                } else {
                    warn!("Failed to serialize demo for entity {:?}", entity);
                }
            } else {
                warn!("Failed to create file for entity {:?}", entity);
            }
        }
    }).detach();
    
    *demo_handler = DemoHandler::default();

    match state.get() {
        RecorderState::Saving => next_state.set(RecorderState::Stopped),
        RecorderState::Recording => next_state.set(RecorderState::Saving),
        _ => warn!("This state should not be possible!"),
    }
}
// Every FRAME_TIME we take a snapshot and push it into the vector.
fn record_demo(
    time: Res<Time>,
    mut timer: ResMut<RecorderTimer>,
    q_kcc: Query<
        (
            Entity,
            &Actions<DefaultContext>,
            &Transform,
            &Character,
            &Collider,
            &CollisionLayers,
        ),
        Without<Frozen>,
    >,
    mut demo_handler: ResMut<DemoHandler>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        // Collider, and layers might change over the course of a demo.
        // For example if crouching is implemented it will most probably change the height of the collider.
        for (entity, actions, transform, character, collider, layers) in q_kcc {
            let snapshot = Snapshot {
                velocity: character.get_velocity(),
                transform: *transform,
            };
            let record = demo_handler.demos.entry(entity).or_insert(Demo::default());

            // We insert the demo here into the HashMap.
            record.snapshots.push(snapshot);
        }
    }
}
