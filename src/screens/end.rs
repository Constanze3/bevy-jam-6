use bevy::prelude::*;

use crate::{menus::Menu, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::End), open_end_menu);
    app.add_systems(OnExit(Screen::End), close_menu);
}

fn open_end_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::End);
}

fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}
