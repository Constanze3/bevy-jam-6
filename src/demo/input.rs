use bevy::prelude::*;

use crate::{AppSystems, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<InputController>();
    app.init_resource::<InputController>();

    app.add_event::<InputEvent>();

    app.add_systems(
        Update,
        record_input
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct InputController {
    pub initial_position: Option<Vec2>,
}

#[derive(Event)]
pub struct InputEvent {
    pub vector: Vec2,
}

fn record_input(
    input: Res<ButtonInput<MouseButton>>,
    mut input_controller: ResMut<InputController>,
    window_query: Query<&Window>,
    mut commands: Commands,
) {
    let window = window_query.single().unwrap();

    // record initial mouse position
    if input.just_pressed(MouseButton::Left) {
        input_controller.initial_position = window.cursor_position();
    }

    // calculate vector and send InputEvent
    if input.just_released(MouseButton::Left) {
        if let Some(initial_position) = input_controller.initial_position {
            if let Some(position) = window.cursor_position() {
                let mut vector = initial_position - position;
                // in screen coordinates the y-axis is reversed
                vector.y *= -1.0;

                commands.trigger(InputEvent { vector });
            }
        }

        input_controller.initial_position = None;
    }
}
