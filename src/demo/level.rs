//! Spawn the main level.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use level_data::LevelData;
use level_loading::LevelAssets;

pub mod level_data;
pub mod level_loading;

use crate::demo::{
    drag_indicator::drag_indicator, killer::Killer, particle::SpawnParticle, player::PlayerConfig,
};
use crate::{
    AppSystems, PausableSystems,
    audio::music::{GameplayMusic, MusicAssets, gameplay_music},
    camera::Letterboxing,
    demo::player::player,
    external::maybe::Maybe,
    screens::Screen,
};

use super::particle::{ParticleDespawned, ParticleSpawned};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<ParticleCount>();

    app.add_plugins((level_data::plugin, level_loading::plugin));
    app.add_observer(spawn_level);
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
            increase_particle_count,
            decrease_particle_count,
            end_level,
            end_game,
        )
            .chain()
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Update),
    );
}

// TODO Add custom levels to level selection menu.
#[allow(dead_code)]
#[derive(Component, Clone)]
#[require(ParticleCount)]
pub enum Level {
    Default(usize),
    Custom(String),
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct ParticleCount(usize);

#[derive(Event)]
pub struct SpawnLevel(pub Level);

/// A system that spawns the main level.
pub fn spawn_level(
    trigger: Trigger<SpawnLevel>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    player_config: Res<PlayerConfig>,
    levels: Res<Assets<LevelData>>,
    level_assets: Res<LevelAssets>,
    music_query: Query<Entity, With<GameplayMusic>>,
    music_assets: Res<MusicAssets>,
    letterboxing: Res<Letterboxing>,
    mut commands: Commands,
) {
    // Spawn screen bounds first
    commands.spawn(screen_bounds(&letterboxing));

    if music_query.is_empty() {
        commands.spawn((gameplay_music(&music_assets), StateScoped(Screen::Gameplay)));
    }

    let level_handle = match &trigger.0 {
        Level::Default(id) => level_assets.default.get(*id).unwrap(),
        Level::Custom(name) => level_assets.custom.get(name).unwrap(),
    };

    let level_data = levels.get(level_handle).unwrap();

    let level = commands
        .spawn((
            Name::new("Level"),
            trigger.0.clone(),
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
        let obstacle_data = obstacle_data.clone();

        let material = materials.add(obstacle_data.flat_color_mesh.color());
        let mesh = meshes.add(obstacle_data.flat_color_mesh.into_mesh());

        let obstacle = commands
            .spawn(obstacle(
                obstacle_data.transform,
                material,
                mesh,
                obstacle_data.collider,
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
    mut particle_count_query: Query<&mut ParticleCount>,
) {
    let mut particle_count = particle_count_query.single_mut().unwrap();
    for _ in events.read() {
        particle_count.0 += 1;
    }
}

fn decrease_particle_count(
    mut events: EventReader<ParticleDespawned>,
    mut particle_count_query: Query<&mut ParticleCount>,
    mut end_level_events: EventWriter<EndLevel>,
) {
    let mut particle_count = particle_count_query.single_mut().unwrap();
    for _ in events.read() {
        particle_count.0 -= 1;
    }

    if particle_count.0 == 0 {
        end_level_events.write(EndLevel);
    }
}

fn restart_level(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    level_query: Query<(Entity, &Level)>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let (entity, level) = level_query.single().unwrap();
        commands.entity(entity).despawn();
        commands.trigger(SpawnLevel(level.clone()));
    }
}

#[derive(Event)]
struct EndLevel;

fn end_level(
    events: EventReader<EndLevel>,
    level_query: Query<(Entity, &Level)>,
    level_assets: Res<LevelAssets>,
    mut end_game_events: EventWriter<EndGame>,
    mut commands: Commands,
) {
    if !events.is_empty() {
        let (entity, level) = level_query.single().unwrap();

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
}

#[derive(Event)]
struct EndGame;

fn end_game(events: EventReader<EndGame>, mut next_screen: ResMut<NextState<Screen>>) {
    if !events.is_empty() {
        next_screen.set(Screen::End);
    }
}
