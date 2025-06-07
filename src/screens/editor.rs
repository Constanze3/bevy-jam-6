use bevy::prelude::*;

use crate::demo::editor::SpawnEditor;

use super::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Editor), |mut commands: Commands| {
        commands.trigger(SpawnEditor);
    });
}
