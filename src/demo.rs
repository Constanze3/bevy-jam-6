//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use bevy::prelude::*;

mod drag_indicator;
mod drag_input;
pub mod editor;
mod killer;
pub mod level;
mod particle;
pub mod particle_effect;
pub mod player;
pub mod time_scale;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        editor::plugin,
        level::plugin,
        player::plugin,
        drag_input::plugin,
        drag_indicator::plugin,
        particle::plugin,
        killer::plugin,
        time_scale::plugin,
    ));
}
