//! Spawn the main level.

use bevy::{platform::collections::HashMap, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    audio::music::{GameplayMusic, MusicAssets, gameplay_music},
    camera::Letterboxing,
    demo::player::player,
    external::maybe::Maybe,
    screens::Screen,
};

use super::{
    indicator::drag_indicator, killer::Killer, level_data::LevelData, particle::SpawnParticle,
    player::PlayerConfig,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();

    app.load_resource::<LevelHandles>();
    app.add_systems(Update, initialize_level_assets);

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
pub struct LevelHandles {
    #[dependency]
    pub default: Vec<Handle<LevelData>>,
    #[dependency]
    pub custom: Vec<Handle<LevelData>>,
}

impl FromWorld for LevelHandles {
    fn from_world(world: &mut World) -> Self {
        let default_levels: Vec<&'static str> = vec!["0", "1"];
        let custom_levels: Vec<&'static str> = vec![];

        let assets = world.resource::<AssetServer>();

        let default = default_levels
            .into_iter()
            .map(|lv| {
                let path = format!("levels/default/{}.ron", lv);
                assets.load(path)
            })
            .collect();

        let custom = custom_levels
            .into_iter()
            .map(|lv| {
                let path = format!("levels/custom/{}.ron", lv);
                assets.load(path)
            })
            .collect();

        Self { default, custom }
    }
}

#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    default: Vec<Handle<LevelData>>,
    custom: HashMap<String, Handle<LevelData>>,
}

// Initializes the LevelAssets resource from the raw LevelHandles resource.
fn initialize_level_assets(
    mut events: EventReader<AssetEvent<LevelHandles>>,
    mut commands: Commands,
    mut level_handles_assets: ResMut<Assets<LevelHandles>>,
    levels: Res<Assets<LevelData>>,
) {
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            let level_handles = level_handles_assets.get_mut(*id).unwrap();

            let default = std::mem::take(&mut level_handles.default);
            let custom = std::mem::take(&mut level_handles.default);

            // Load default levels as a sorted Vec<Handle<LevelData>>.
            let map_default = |handles: Vec<Handle<LevelData>>| {
                let mut folder = handles.clone();

                // Each level should be named with numbers.
                // This sorts them by their name.
                folder.sort_by(|h1, h2| {
                    let parse_name = |level: &LevelData| {
                        level
                            .name
                            .parse()
                            .expect("Default level names should be numbers.")
                    };

                    let level1 = levels.get(h1).unwrap();
                    let id1: usize = parse_name(level1);

                    let level2 = levels.get(h2).unwrap();
                    let id2: usize = parse_name(level2);

                    id1.cmp(&id2)
                });

                folder
            };

            // Load custom levels as a HashMap<String, Handle<LevelData>>.
            let map_custom = |handles: Vec<Handle<LevelData>>| {
                handles
                    .iter()
                    .map(|h| {
                        let level = levels.get(h).unwrap();

                        (level.name.clone(), h.clone())
                    })
                    .collect()
            };

            commands.remove_resource::<LevelHandles>();
            commands.insert_resource(LevelAssets {
                default: map_default(default),
                custom: map_custom(custom),
            });
        }
    }
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
