use bevy::prelude::*;

use crate::{
    menus::Menu,
    screens::Screen,
    theme::{BoldFont, palette::HEADER_TEXT, widget},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::End), spawn_end_menu);
}

fn spawn_end_menu(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("End Menu"),
        GlobalZIndex(2),
        StateScoped(Menu::End),
        children![
            (
                Name::new("Header"),
                Text("You Win!".into()),
                TextFont::from_font_size(80.0),
                BoldFont,
                TextColor(HEADER_TEXT),
            ),
            Node {
                height: Val::Px(20.0),
                ..default()
            },
            widget::button("Levels", quit_to_levels),
            widget::button("Quit to title", quit_to_title),
        ],
    ));
}

fn quit_to_levels(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Levels);
}

fn quit_to_title(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
