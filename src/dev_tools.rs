#![allow(unused)]
//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    dev_tools::states::log_transitions, input::common_conditions::input_just_pressed, prelude::*,
    ui::UiDebugOptions,
};
use bevy_rapier2d::prelude::*;

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);

    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        (
            toggle_debug_ui.run_if(input_just_pressed(TOGGLE_KEY)),
            // debug_collision_events,
        ),
    );
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}

fn debug_collision_events(mut collision_events: EventReader<CollisionEvent>, query: Query<&Name>) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            let name1 = query.get(*e1).map(|n| n.as_str()).unwrap_or("?");
            let name2 = query.get(*e2).map(|n| n.as_str()).unwrap_or("?");
            println!("Collision: {} <-> {}", name1, name2);
        }
    }
}
