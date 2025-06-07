use bevy::prelude::*;

use crate::{menus::Menu, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Levels), open_levels_menu);
    app.add_systems(OnExit(Screen::Levels), close_menu);
}

fn open_levels_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Levels);
}

fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
