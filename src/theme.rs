//! Reusable UI widgets & theming.

// Unused utilities may trigger this lints undesirably.
#![allow(dead_code)]

pub mod interaction;
pub mod palette;
pub mod widget;

#[allow(unused_imports)]
pub mod prelude {
    pub use super::{interaction::InteractionPalette, palette as ui_palette, widget};
}

use bevy::prelude::*;

use crate::{asset_tracking::LoadResource, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(interaction::plugin);

    app.register_type::<Fonts>();
    app.load_resource::<Fonts>();

    app.add_systems(
        Update,
        (inject_regular_font, inject_bold_font).run_if(not(in_state(Screen::Splash))),
    );
}

#[derive(Component)]
#[require(TextFont)]
pub struct RegularFont;

#[derive(Component)]
#[require(TextFont)]
pub struct BoldFont;

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct Fonts {
    #[dependency]
    pub regular: Handle<Font>,
    #[dependency]
    pub bold: Handle<Font>,
}

impl FromWorld for Fonts {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        Self {
            regular: assets.load("fonts/ttf/Oxanium-Regular.ttf"),
            bold: assets.load("fonts/ttf/Oxanium-Bold.ttf"),
        }
    }
}

fn inject_regular_font(mut query: Query<&mut TextFont, With<RegularFont>>, fonts: Res<Fonts>) {
    for mut text_font in query.iter_mut() {
        text_font.font = fonts.regular.clone();
    }
}

fn inject_bold_font(mut query: Query<&mut TextFont, With<BoldFont>>, fonts: Res<Fonts>) {
    for mut text_font in query.iter_mut() {
        text_font.font = fonts.bold.clone();
    }
}
