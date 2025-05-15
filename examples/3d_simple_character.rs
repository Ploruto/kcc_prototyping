use avian3d::{PhysicsPlugins, prelude::PhysicsDebugPlugin};
use bevy::prelude::*;
use examples_common::ExampleCommonPlugin;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ExampleCommonPlugin,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
        ))
        .run()
}
