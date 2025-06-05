use bevy::prelude::*;

use crate::{AppSystems, asset_tracking::LoadResource, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MusicAssets>();
    app.load_resource::<MusicAssets>();
    app.add_systems(
        Update,
        update_gameplay_music
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Asset, Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct MusicAssets {
    #[dependency]
    main_theme_intro: Handle<AudioSource>,
    #[dependency]
    main_theme_loop: Handle<AudioSource>,
}

impl FromWorld for MusicAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        Self {
            main_theme_intro: assets.load::<AudioSource>("audio/music/main_theme_intro.ogg"),
            main_theme_loop: assets.load::<AudioSource>("audio/music/main_theme_loop.ogg"),
        }
    }
}

#[derive(Component)]
pub enum GameplayMusic {
    Intro,
    Loop,
}

pub fn gameplay_music(music_assets: &MusicAssets) -> impl Bundle {
    (
        GameplayMusic::Intro,
        AudioPlayer(music_assets.main_theme_intro.clone()),
        PlaybackSettings::ONCE,
    )
}

fn update_gameplay_music(
    mut audio_query: Query<(Entity, &mut GameplayMusic, &AudioSink)>,
    music_assets: Res<MusicAssets>,
    mut commands: Commands,
) {
    for (entity, mut music, sink) in audio_query.iter_mut() {
        if sink.empty() {
            #[allow(clippy::single_match)]
            match *music {
                GameplayMusic::Intro => {
                    *music = GameplayMusic::Loop;

                    commands
                        .entity(entity)
                        .remove::<AudioPlayer>()
                        .remove::<PlaybackSettings>()
                        .remove::<AudioSink>()
                        .insert(AudioPlayer(music_assets.main_theme_loop.clone()))
                        .insert(PlaybackSettings::LOOP);
                }
                _ => {}
            }
        }
    }
}
