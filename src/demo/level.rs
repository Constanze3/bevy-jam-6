//! Spawn the main level.

use std::time::Duration;

use bevy::audio::Volume;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use level_data::LevelData;
use level_loading::LevelAssets;

pub mod level_data;
pub mod level_loading;

use crate::asset_tracking::LoadResource;
use crate::audio::{SoundEffect, sound_effect};
use crate::demo::{
    drag_indicator::drag_indicator, killer::Killer, particle::SpawnParticle, player::PlayerConfig,
};
use crate::{
    AppSystems, PausableSystems,
    audio::music::{GameplayMusic, MusicAssets, gameplay_music},
    camera::Letterboxing,
    demo::particle::{ParticleDespawned, ParticleSpawned},
    demo::player::player,
    external::maybe::Maybe,
    screens::Screen,
};

use super::editor::EditorState;
use super::player::Player;
use super::time_scale::{SetTimeScale, SetTimeScaleOverride, TimeScaleKind};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAudioAssets>();
    app.register_type::<ParticleCount>();

    app.load_resource::<LevelAudioAssets>();

    app.add_plugins((level_data::plugin, level_loading::plugin));

    app.add_observer(spawn_level);
    app.add_observer(spawn_raw_level);

    app.add_systems(
        Update,
        restart_level
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );

    app.add_event::<EndLevel>();
    app.add_event::<EndGame>();
    app.add_systems(
        Update,
        (
            (increase_particle_count, decrease_particle_count).chain(),
            (tick_end_level_timer, end_level, end_game).chain(),
        )
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Update),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
struct LevelAudioAssets {
    #[dependency]
    restart_sound: Handle<AudioSource>,
    #[dependency]
    level_completed_sound: Handle<AudioSource>,
}

impl FromWorld for LevelAudioAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        Self {
            restart_sound: assets.load("audio/sound_effects/restart.ogg"),
            level_completed_sound: assets.load("audio/sound_effects/level_completed.ogg"),
        }
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct ParticleCount(usize);

#[derive(Component, Default, PartialEq, Eq)]
pub enum LevelState {
    #[default]
    Playing,
    Ended,
}

// TODO Add custom levels to level selection menu.
#[allow(dead_code)]
#[derive(Component, Clone)]
pub enum Level {
    Default(usize),
    Custom(String),
}

#[derive(Event)]
pub struct SpawnLevel(pub Level);

pub fn spawn_level(
    trigger: Trigger<SpawnLevel>,
    levels: Res<Assets<LevelData>>,
    level_assets: Res<LevelAssets>,
    mut commands: Commands,
) {
    let level_handle = match &trigger.0 {
        Level::Default(id) => level_assets.default.get(*id).unwrap(),
        Level::Custom(name) => level_assets.custom.get(name).unwrap(),
    };

    let level_data = levels.get(level_handle).unwrap();

    commands.trigger(SpawnRawLevel {
        data: level_data.clone(),
        level: Some(trigger.0.clone()),
    });
}

#[derive(Event)]
pub struct SpawnRawLevel {
    pub data: LevelData,
    pub level: Option<Level>,
}

#[derive(Component)]
#[require(ParticleCount, LevelState)]
pub struct RawLevel(pub LevelData);

/// A system that spawns the main level.
pub fn spawn_raw_level(
    mut trigger: Trigger<SpawnRawLevel>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_config: Res<PlayerConfig>,
    music_query: Query<Entity, With<GameplayMusic>>,
    music_assets: Res<MusicAssets>,
    letterboxing: Res<Letterboxing>,
    mut commands: Commands,
) {
    let level_data = std::mem::take(&mut trigger.data);

    // Spawn screen bounds first
    commands.spawn(screen_bounds(&letterboxing));

    if music_query.is_empty() {
        commands.spawn((gameplay_music(&music_assets), StateScoped(Screen::Gameplay)));
    }

    let level = commands
        .spawn((
            Name::new("Level"),
            Maybe(trigger.level.clone()),
            Transform::default(),
            Visibility::default(),
            StateScoped(Screen::Gameplay),
            children![
                player(
                    level_data.player_spawn,
                    &mut meshes,
                    &mut materials,
                    &player_config
                ),
                drag_indicator(
                    6.0,
                    0.4,
                    Color::hsl(0.0, 0.0, 0.6),
                    Color::Srgba(Srgba::hex("7aad81").unwrap()),
                    &mut meshes,
                    &mut materials
                ),
            ],
        ))
        .id();

    for obstacle_data in level_data.obstacles.iter() {
        let material = materials.add(obstacle_data.color);
        let mesh = meshes.add(Rectangle::new(obstacle_data.width, obstacle_data.height));

        let obstacle = commands
            .spawn(obstacle(
                obstacle_data.transform,
                material,
                mesh,
                Collider::cuboid(obstacle_data.width / 2.0, obstacle_data.height / 2.0),
                obstacle_data.is_killer,
            ))
            .id();

        commands.entity(level).add_child(obstacle);
    }

    for particle_data in level_data.particles.iter() {
        commands.trigger(SpawnParticle {
            translation: particle_data.spawn_position,
            particle: particle_data.particle.clone(),
            spawn_with_invincible: false,
            parent: Some(level),
        });
    }

    commands.entity(level).insert(RawLevel(level_data));
}

pub fn obstacle(
    transform: Transform,
    material: Handle<ColorMaterial>,
    mesh: Handle<Mesh>,
    collider: Collider,
    is_killer: bool,
) -> impl Bundle {
    (
        Name::new("Obstacle"),
        transform,
        Mesh2d(mesh),
        MeshMaterial2d(material),
        RigidBody::Fixed,
        collider,
        {
            if !is_killer {
                CollisionGroups::new(Group::GROUP_1, Group::all())
            } else {
                CollisionGroups::new(Group::GROUP_1 | Group::GROUP_3, Group::all())
            }
        },
        Maybe(is_killer.then_some(Killer)),
    )
}

fn screen_bounds(letterboxing: &Letterboxing) -> impl Bundle {
    let width = letterboxing.projection_size.width;
    let height = letterboxing.projection_size.height;

    let halfwidth = width / 2.0;
    let halfheight = height / 2.0;

    let thickness = 1.0;
    let restitution = 0.5;

    (
        Name::new("Screen Bounds"),
        Transform::default(),
        children![
            (
                Name::new("Left Wall"),
                Transform::from_xyz(-(halfwidth + thickness), 0.0, 0.0),
                RigidBody::Fixed,
                Collider::cuboid(thickness, halfheight),
                CollisionGroups::new(Group::GROUP_1, Group::all()),
                Restitution::coefficient(restitution),
            ),
            (
                Name::new("Right Wall"),
                Transform::from_xyz(halfwidth + thickness, 0.0, 0.0),
                RigidBody::Fixed,
                Collider::cuboid(thickness, halfheight),
                CollisionGroups::new(Group::GROUP_1, Group::all()),
                Restitution::coefficient(restitution),
            ),
            (
                Name::new("Top Wall"),
                Transform::from_xyz(0.0, halfheight + thickness, 0.0),
                RigidBody::Fixed,
                Collider::cuboid(halfwidth, thickness),
                CollisionGroups::new(Group::GROUP_1, Group::all()),
                Restitution::coefficient(restitution),
            ),
            (
                Name::new("Bottom Wall"),
                Transform::from_xyz(0.0, -(halfheight + thickness), 0.0),
                RigidBody::Fixed,
                Collider::cuboid(halfwidth, thickness),
                CollisionGroups::new(Group::GROUP_1, Group::all()),
                Restitution::coefficient(restitution),
            )
        ],
    )
}

fn increase_particle_count(
    mut events: EventReader<ParticleSpawned>,
    mut level_query: Query<(&LevelState, &mut ParticleCount)>,
) {
    let (level_state, mut particle_count) = level_query.single_mut().unwrap();

    if *level_state == LevelState::Ended {
        return;
    }

    for _ in events.read() {
        particle_count.0 += 1;
    }
}

fn decrease_particle_count(
    mut events: EventReader<ParticleDespawned>,
    mut level_query: Query<(Entity, &mut LevelState, &mut ParticleCount), With<RawLevel>>,
    mut player_query: Query<&mut Player, Without<RawLevel>>,
    audio_assets: Res<LevelAudioAssets>,
    mut time_events: EventWriter<SetTimeScale>,
    mut time_override_events: EventWriter<SetTimeScaleOverride>,
    mut commands: Commands,
) {
    let (level_entity, mut level_state, mut particle_count) = level_query.single_mut().unwrap();
    if *level_state == LevelState::Ended {
        return;
    }

    for _ in events.read() {
        particle_count.0 -= 1;
    }

    if particle_count.0 == 0 {
        commands.entity(level_entity).with_children(|parent| {
            parent.spawn(EndLevelTimer::new());
            parent.spawn(sound_effect(audio_assets.level_completed_sound.clone()));
        });

        *level_state = LevelState::Ended;

        if let Ok(mut player) = player_query.single_mut() {
            player.can_move = false;
        }

        time_override_events.write(SetTimeScaleOverride(None));
        time_events.write(SetTimeScale(TimeScaleKind::Normal));
    }
}

#[derive(Component)]
struct EndLevelTimer(Timer);

impl EndLevelTimer {
    pub fn new() -> Self {
        Self(Timer::new(Duration::from_secs_f32(2.0), TimerMode::Once))
    }
}

fn tick_end_level_timer(
    mut query: Query<(Entity, &mut EndLevelTimer)>,
    time: Res<Time>,
    mut end_level_events: EventWriter<EndLevel>,
    mut commands: Commands,
) {
    for (entity, mut timer) in query.iter_mut() {
        timer.0.tick(time.delta());

        if timer.0.just_finished() {
            end_level_events.write(EndLevel);
            commands.entity(entity).despawn();
        }
    }
}

fn restart_level(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut level_query: Query<(Entity, &mut RawLevel, Option<&Level>)>,
    audio_assets: Res<LevelAudioAssets>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let (entity, mut raw_level, level) = level_query.single_mut().unwrap();

        commands.entity(entity).despawn();
        commands.trigger(SpawnRawLevel {
            data: std::mem::take(&mut raw_level.0),
            level: level.cloned(),
        });

        commands.spawn((
            AudioPlayer(audio_assets.restart_sound.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(2.5)),
            SoundEffect,
        ));
    }
}

#[derive(Event)]
struct EndLevel;

fn end_level(
    mut events: EventReader<EndLevel>,
    level_query: Query<(Entity, Option<&Level>), With<RawLevel>>,
    level_assets: Res<LevelAssets>,
    mut end_game_events: EventWriter<EndGame>,
    mut commands: Commands,
    editor_state: Res<EditorState>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    if !events.is_empty() {
        let (entity, level) = level_query.single().unwrap();

        let Some(level) = level else {
            if editor_state.editing {
                next_screen.set(Screen::Editor);
            } else {
                next_screen.set(Screen::Levels);
            }

            return;
        };

        if let Level::Default(id) = level {
            let new_id = id + 1;

            if level_assets.default.len() <= new_id {
                end_game_events.write(EndGame);
                return;
            }

            // Spawn next level.
            commands.trigger(SpawnLevel(Level::Default(id + 1)));
        } else {
            panic!("Not implemented.");
        }

        commands.entity(entity).despawn();
    }
    events.clear();
}

#[derive(Event)]
struct EndGame;

fn end_game(mut events: EventReader<EndGame>, mut next_screen: ResMut<NextState<Screen>>) {
    if !events.is_empty() {
        next_screen.set(Screen::End);
    }
    events.clear();
}
