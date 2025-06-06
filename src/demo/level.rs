//! Spawn the main level.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    audio::music::{GameplayMusic, MusicAssets, gameplay_music},
    camera::Letterboxing,
    demo::player::player,
    screens::Screen,
};

use super::{
    indicator::drag_indicator,
    particle::{Particle, SpawnParticle},
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();

    app.add_observer(spawn_level);
    app.add_systems(
        Update,
        restart_level
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/Fluffing A Duck.ogg"),
        }
    }
}

#[derive(Component)]
pub struct Level;

#[derive(Event)]
pub struct SpawnLevel;

/// A system that spawns the main level.
pub fn spawn_level(
    _: Trigger<SpawnLevel>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    music_query: Query<Entity, With<GameplayMusic>>,
    music_assets: Res<MusicAssets>,
    letterboxing: Res<Letterboxing>,
    mut commands: Commands,
) {
    // Spawn screen bounds first
    commands.spawn(screen_bounds(letterboxing.as_ref()));

    if music_query.is_empty() {
        commands.spawn((
            gameplay_music(music_assets.as_ref()),
            StateScoped(Screen::Gameplay),
        ));
    }

    let level = commands
        .spawn((
            Name::new("Level"),
            Level,
            Transform::default(),
            Visibility::default(),
            StateScoped(Screen::Gameplay),
            children![
                player(
                    Vec2::new(0.0, -100.0),
                    20.0,
                    7000.0,
                    &mut meshes,
                    &mut materials
                ),
                drag_indicator(
                    6.0,
                    0.4,
                    Color::hsl(0.0, 0.0, 0.6),
                    Color::Srgba(Srgba::hex("7aad81").unwrap()),
                    &mut meshes,
                    &mut materials
                ),
                obstacle(vec2(100.0, 0.0), 50.0, &mut meshes, &mut materials),
            ],
        ))
        .id();

    commands.trigger(SpawnParticle {
        translation: vec2(-100.0, 0.0),
        particle: Particle {
            subparticles: vec![
                Box::new(Particle {
                    initial_velocity: vec2(0.0, -200.0),
                    ..default()
                }),
                Box::new(Particle {
                    initial_velocity: vec2(0.0, 200.0),
                    ..default()
                }),
            ],
            ..default()
        },
        spawn_with_invincible: false,
        parent: Some(level),
    });
}

pub fn obstacle(
    translation: Vec2,
    size: f32,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> impl Bundle {
    let mesh = meshes.add(Rectangle::new(size, size));
    let material = materials.add(Color::linear_rgb(1.0, 1.0, 1.0));
    (
        Name::new("Obstacle"),
        Transform::from_translation(translation.extend(0.0)),
        Mesh2d(mesh),
        MeshMaterial2d(material),
        RigidBody::Fixed,
        Collider::cuboid(size / 2.0, size / 2.0),
        CollisionGroups::new(Group::GROUP_1, Group::all()),
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

fn restart_level(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    level_query: Query<Entity, With<Level>>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let level = level_query.single().unwrap();
        commands.entity(level).despawn();
        commands.trigger(SpawnLevel);
    }
}
