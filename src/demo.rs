//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use bevy::prelude::*;

pub mod editor;
mod indicator;
mod input;
mod killer;
pub mod level;
pub mod level_data;
mod particle;
pub mod particle_effect;
pub mod player;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        level_data::plugin,
        level::plugin,
        player::plugin,
        input::plugin,
        indicator::plugin,
        particle::plugin,
        editor::plugin,
        killer::plugin,
    ));
}
