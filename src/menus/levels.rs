//! The level selection screen.

use bevy::prelude::*;

use crate::{
    demo::level::Level,
    menus::Menu,
    screens::{Screen, gameplay::SelectedLevel},
    theme::widget,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Levels), spawn_levels_screen);
}

#[derive(Component)]
struct LevelButton(Level);

fn spawn_levels_screen(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Levels Screen"),
        GlobalZIndex(2),
        StateScoped(Menu::Levels),
        children![
            widget::header("Levels"),
            (
                widget::button("0", enter_gameplay_screen),
                LevelButton(Level::Default(0))
            ),
            (
                widget::button("1", enter_gameplay_screen),
                LevelButton(Level::Default(1))
            ),
            widget::button("Back", go_back)
        ],
    ));
}

fn enter_gameplay_screen(
    trigger: Trigger<Pointer<Click>>,
    parent_query: Query<&ChildOf>,
    level_button_query: Query<&LevelButton>,
    mut selected_level: ResMut<SelectedLevel>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    let entity = trigger.target();
    let parent = parent_query.get(entity).unwrap();
    let level_button = level_button_query.get(parent.0).unwrap();

    selected_level.0 = Some(level_button.0.clone());

    next_screen.set(Screen::Gameplay);
}

fn go_back(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
