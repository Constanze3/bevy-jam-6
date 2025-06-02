//! Spawn the main level.

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::egui::collapsing_header;
use bevy_rapier2d::prelude::{Collider, RigidBody};

use crate::{asset_tracking::LoadResource, demo::player::player, screens::Screen};

use super::{atom::atom_seed, indicator::drag_indicator};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();
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

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single().expect("PrimaryWindow not found");

    // Spawn screen bounds first
    spawn_screen_bounds(&mut commands, window);

    let atom_radius = 50.0;
    // let num_parts = 6;
    // let part_radius = 10.0;
    // let angle_step = std::f32::consts::TAU / num_parts as f32;

    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        StateScoped(Screen::Gameplay),
        children![
            player(20.0, 7000.0, &mut meshes, &mut materials),
            drag_indicator(
                6.0,
                0.4,
                Color::hsl(0.0, 0.0, 0.6),
                &mut meshes,
                &mut materials
            ),
            obstacle(Vec2 { x: 100.0, y: 0.0 }, 50.0, &mut meshes, &mut materials),
            atom_seed(
                Vec2 { x: -100.0, y: 0.0 },
                atom_radius,
                &mut meshes,
                &mut materials
            ),
        ],
    ));
}

fn obstacle(
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
    )
}

fn spawn_screen_bounds(commands: &mut Commands, window: &Window) {
    let width = window.width();
    let height = window.height();
    commands.spawn((
        Name::new("Left Wall"),
        Transform::from_xyz(-width / 2.0, 0.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(1.0, height / 2.0),
    ));
    commands.spawn((
        Name::new("Right Wall"),
        Transform::from_xyz(width / 2.0, 0.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(1.0, height / 2.0),
    ));
    commands.spawn((
        Name::new("Top Wall"),
        Transform::from_xyz(0.0, height / 2.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(width / 2.0, 1.0),
    ));
    commands.spawn((
        Name::new("Bottom Wall"),
        Transform::from_xyz(0.0, -height / 2.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(width / 2.0, 1.0),
    ));
}
