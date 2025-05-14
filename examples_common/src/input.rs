use bevy::math::FloatPow;
use bevy::prelude::*;
use bevy::time::Virtual;
use bevy::window::{CursorGrabMode, Window};
use bevy_enhanced_input::prelude::*;

// --- General Actions (Likely used across contexts) ---

#[derive(Debug, Clone, Copy, InputAction)]
#[input_action(output = Vec2)]
pub struct Move; // Player/Character movement

#[derive(Debug, Clone, Copy, InputAction)]
#[input_action(output = Vec2)]
pub struct Look; // Camera look/rotation

#[derive(Debug, Clone, Copy, InputAction)]
#[input_action(output = bool)]
pub struct Jump;

#[derive(Debug, Clone, Copy, InputAction)]
#[input_action(output = bool)]
pub struct CaptureCursor;

#[derive(Debug, Clone, Copy, InputAction)]
#[input_action(output = bool)]
pub struct ReleaseCursor;

#[derive(Debug, Clone, Copy, InputAction)]
#[input_action(output = bool)]
pub struct ToggleViewPerspective;

#[derive(Debug, Clone, Copy, InputAction)]
#[input_action(output = bool)]
pub struct ToggleFlyCam;

// --- Fly Camera Specific Actions ---

#[derive(InputAction, Debug, Clone, Copy)]
#[input_action(output = f32)]
pub(crate) struct Fly;

// --- Orbit Camera Specific Actions  ---
#[derive(Debug, Clone, Copy, InputAction)]
#[input_action(output = Vec2)]
pub struct OrbitZoom;

// --- Input Contexts ---

/// Default context, primarily for FPS controls and global actions.
#[derive(InputContext, Default)]
pub struct DefaultContext;

/// Context for Fly Camera specific controls.
#[derive(InputContext, Default)]
pub struct FlyCameraContext;

/// Context for Orbit Camera specific controls.
#[derive(InputContext, Default)]
pub struct OrbitCameraContext;

// --- Plugin Setup ---

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnhancedInputPlugin)
            // Register contexts
            .add_input_context::<DefaultContext>()
            .add_input_context::<FlyCameraContext>()
            .add_input_context::<OrbitCameraContext>()
            // Add binding systems triggered when the corresponding Actions component is added
            .add_observer(bind_default_context_actions)
            .add_observer(bind_fly_camera_actions)
            .add_observer(bind_orbit_camera_actions)
            // Add action handlers
            .add_observer(capture_cursor)
            .add_observer(release_cursor);
    }
}

// --- Binding Systems ---

/// Binds actions for the DefaultContext (FPS, Global)
/// Triggered when Actions<DefaultContext> is added to an entity.
fn bind_default_context_actions(
    trigger: Trigger<OnAdd, Actions<DefaultContext>>,
    mut players: Query<&mut Actions<DefaultContext>>,
) {
    // Get the action map for the entity the component was added to
    if let Ok(mut actions) = players.get_mut(trigger.target()) {
        info!(
            "Binding DefaultContext actions for entity {:?}",
            trigger.target()
        );
        // --- Player Movement & Interaction ---
        actions
            .bind::<Move>()
            .to((Cardinal::wasd_keys(), Axial::left_stick()))
            .with_modifiers(DeadZone::default()); // Keep existing modifiers if needed

        actions.bind::<CaptureCursor>().to(MouseButton::Left);
        actions.bind::<ReleaseCursor>().to(KeyCode::Escape);

        actions
            .bind::<Jump>()
            .to((KeyCode::Space, GamepadButton::East))
            .with_conditions(JustPress::default());

        // --- Camera Look (Used by FPS, potentially others if not overridden) ---
        actions.bind::<Look>().to((
            Input::mouse_motion().with_modifiers((Scale::splat(0.05), Negate::all())),
            Axial::right_stick().with_modifiers_each((Negate::x(), AtLeast::default())),
        ));

        // --- Global Actions ---

        actions
            .bind::<ToggleViewPerspective>()
            .to((KeyCode::KeyC, GamepadButton::DPadDown))
            .with_conditions(JustPress::default());
        actions
            .bind::<ToggleFlyCam>()
            .to((KeyCode::KeyF, GamepadButton::DPadUp))
            .with_conditions(JustPress::default());
    } else {
        warn!(
            "Failed to get Actions<DefaultContext> for entity {:?} during binding",
            trigger.target()
        );
    }
}

/// Binds actions specific to the FlyCameraContext.
/// Triggered when Actions<FlyCameraContext> is added to an entity.
fn bind_fly_camera_actions(
    trigger: Trigger<OnAdd, Actions<FlyCameraContext>>,
    mut players: Query<&mut Actions<FlyCameraContext>>,
) {
    if let Ok(mut actions) = players.get_mut(trigger.target()) {
        info!(
            "Binding FlyCameraContext actions for entity {:?}",
            trigger.target()
        );

        // Bind vertical movement for FlyCam
        actions.bind::<Fly>().to((
            Bidirectional {
                positive: KeyCode::KeyE,
                negative: KeyCode::KeyQ,
            },
            Bidirectional {
                positive: GamepadButton::East,
                negative: GamepadButton::LeftThumb,
            },
        ));
    } else {
        warn!(
            "Failed to get Actions<FlyCameraContext> for entity {:?} during binding",
            trigger.target()
        );
    }
}

/// Binds actions specific to the OrbitCameraContext.
/// Triggered when Actions<OrbitCameraContext> is added to an entity.
fn bind_orbit_camera_actions(
    trigger: Trigger<OnAdd, Actions<OrbitCameraContext>>,
    mut players: Query<&mut Actions<OrbitCameraContext>>,
) {
    if let Ok(mut actions) = players.get_mut(trigger.target()) {
        info!(
            "Binding OrbitCameraContext actions for entity {:?}",
            trigger.target()
        );
        actions.bind::<OrbitZoom>().to(Input::mouse_wheel());
    } else {
        warn!(
            "Failed to get Actions<OrbitCameraContext> for entity {:?} during binding",
            trigger.target()
        );
    }
}

// --- Action Handlers ---

fn capture_cursor(
    _trigger: Trigger<Completed<CaptureCursor>>, // Triggered by DefaultContext action
    mut windows: Query<&mut Window>, // Use Query instead of Single if multiple windows possible
) {
    if let Ok(mut window) = windows.single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Confined;
        window.cursor_options.visible = false;
    }
}

fn release_cursor(
    _trigger: Trigger<Completed<ReleaseCursor>>, // Triggered by DefaultContext action
    mut windows: Query<&mut Window>,
) {
    if let Ok(mut window) = windows.single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }
}

// Helper
/// InputModifier that sets the value of an input to 0.0 if a threshold is not reached used to
/// prevent stick drift. It differs to DeadZone in that it doesn't remap the input range to [0, 1].
#[derive(Debug, Clone)]
struct AtLeast {
    value: f32,
    /// If set to true the absolute value of the ActionValue will be used in comparison with value.
    compare_absolute_value: bool,
}

impl Default for AtLeast {
    fn default() -> Self {
        Self {
            value: 0.2,
            compare_absolute_value: true,
        }
    }
}

impl InputModifier for AtLeast {
    fn apply(
        &mut self,
        _: &bevy_enhanced_input::action_map::ActionMap,
        _: &Time<Virtual>,
        value: ActionValue,
    ) -> ActionValue {
        match value {
            ActionValue::Bool(bool) => bool.into(),
            ActionValue::Axis1D(mut vec1) => {
                let sign = vec1.signum();
                if self.compare_absolute_value {
                    vec1 = vec1.abs();
                }
                (if vec1 < self.value { 0.0 } else { vec1 * sign }).into()
            }
            ActionValue::Axis2D(vec2) => (if vec2.length_squared() < self.value.squared() {
                Vec2::splat(0.0)
            } else {
                vec2
            })
            .into(),
            ActionValue::Axis3D(vec3) => (if vec3.length_squared() < self.value.squared() {
                Vec3::splat(0.0)
            } else {
                vec3
            })
            .into(),
        }
    }
}
