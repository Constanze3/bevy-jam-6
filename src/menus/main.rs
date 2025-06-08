//! The main menu (seen on the title screen).

use bevy::prelude::*;

use crate::{
    menus::Menu,
    screens::Screen,
    theme::{BoldFont, palette::HEADER_TEXT, widget},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Main), spawn_main_menu);
}

fn spawn_main_menu(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Main Menu"),
        GlobalZIndex(2),
        StateScoped(Menu::Main),
        #[cfg(not(target_family = "wasm"))]
        children![
            (
                Name::new("Header"),
                Text("Antim4tter".into()),
                TextFont::from_font_size(80.0),
                BoldFont,
                TextColor(HEADER_TEXT),
            ),
            Node {
                height: Val::Px(20.0),
                ..default()
            },
            widget::button("Play", enter_levels_screen),
            widget::button("Editor", enter_editor_screen),
            widget::button("Settings", open_settings_menu),
            widget::button("Credits", open_credits_menu),
            widget::button("Exit", exit_app),
        ],
        #[cfg(target_family = "wasm")]
        children![
            (
                Name::new("Header"),
                Text("Antim4tter".into()),
                TextFont::from_font_size(80.0),
                BoldFont,
                TextColor(HEADER_TEXT),
            ),
            Node {
                height: Val::Px(20.0),
                ..default()
            },
            widget::button("Play", enter_levels_screen),
            widget::button("Settings", open_settings_menu),
            widget::button("Credits", open_credits_menu),
        ],
    ));
}

fn enter_levels_screen(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Levels);
}

fn enter_editor_screen(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Editor);
}

fn open_settings_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn open_credits_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Credits);
}

#[cfg(not(target_family = "wasm"))]
fn exit_app(_: Trigger<Pointer<Click>>, mut app_exit: EventWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
