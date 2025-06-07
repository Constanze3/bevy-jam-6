//! The level selection screen.

use bevy::prelude::*;

use crate::{
    asset_tracking::ResourceHandles,
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
                widget::button("0", enter_loading_or_gameplay_screen),
                LevelButton(Level::Default(0))
            ),
            (
                widget::button("1", enter_loading_or_gameplay_screen),
                LevelButton(Level::Default(1))
            ),
            widget::button("Back", go_back)
        ],
    ));
}

fn enter_loading_or_gameplay_screen(
    trigger: Trigger<Pointer<Click>>,
    parent_query: Query<&ChildOf>,
    level_button_query: Query<&LevelButton>,
    resource_handles: Res<ResourceHandles>,
    mut selected_level: ResMut<SelectedLevel>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    let entity = trigger.target();
    let parent = parent_query.get(entity).unwrap();
    let level_button = level_button_query.get(parent.0).unwrap();

    selected_level.0 = Some(level_button.0.clone());

    if resource_handles.is_all_done() {
        next_screen.set(Screen::Gameplay);
    } else {
        next_screen.set(Screen::Loading);
    }
}

fn go_back(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main);
}
