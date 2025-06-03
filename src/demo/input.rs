use crate::{AppSystems, PausableSystems, Pause, screens::Screen};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<InputController>();
    app.init_resource::<InputController>();

    app.add_event::<InputEvent>();

    app.add_systems(
        Update,
        record_input
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );

    app.add_systems(OnEnter(Pause(true)), reset_input);
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct InputController {
    pub initial_position: Option<Vec2>,
    pub vector: Option<Vec2>,
}

#[derive(Event)]
pub struct InputEvent {
    pub vector: Vec2,
}

fn record_input(
    input: Res<ButtonInput<MouseButton>>,
    mut input_controller: ResMut<InputController>,
    window_query: Query<&Window>,
    mut events: EventWriter<InputEvent>,
) {
    let window = window_query.single().unwrap();

    // Record initial mouse position.
    if input.just_pressed(MouseButton::Left) {
        input_controller.initial_position = window.cursor_position();
    }

    // Update vector of input controller.
    if input.pressed(MouseButton::Left) {
        input_controller.vector =
            calculate_vector(input_controller.initial_position, window.cursor_position());
    }

    // Send input event.
    if input.just_released(MouseButton::Left) {
        let vector = calculate_vector(input_controller.initial_position, window.cursor_position());

        if let Some(vector) = vector {
            events.write(InputEvent { vector });
        }

        input_controller.initial_position = None;
        input_controller.vector = None;
    }
}

fn calculate_vector(
    initial_position: Option<Vec2>,
    current_position: Option<Vec2>,
) -> Option<Vec2> {
    if let Some(initial_position) = initial_position {
        if let Some(position) = current_position {
            let mut vector = initial_position - position;
            // In screen coordinates the y-axis is reversed.
            vector.y *= -1.0;

            return Some(vector);
        }
    }

    None
}

fn reset_input(mut input_controller: ResMut<InputController>) {
    *input_controller = InputController::default();
}
