//! The game's main screen states and transitions between them.

mod end;
pub mod gameplay;
mod levels;
mod loading;
mod splash;
mod title;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>();

    app.add_plugins((
        end::plugin,
        gameplay::plugin,
        levels::plugin,
        loading::plugin,
        splash::plugin,
        title::plugin,
    ));
}

/// The game's main screen states.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
pub enum Screen {
    #[default]
    Splash,
    Title,
    Editor,
    Levels,
    Loading,
    Gameplay,
    End,
}
