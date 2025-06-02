//! Spawn the main level.

use bevy::prelude::*;
use bevy_rapier2d::prelude::{Collider, RigidBody};

use crate::{asset_tracking::LoadResource, demo::player::player, screens::Screen};

use super::{atom::{atom, atom_part}, indicator::drag_indicator};

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
) {
    let atom_radius = 10.0;
    let num_parts = 6;
    let part_radius = 10.0;
    let angle_step = std::f32::consts::TAU / num_parts as f32;

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
            (
                atom(Vec2 { x: 100.0, y: 0.0 }, atom_radius, num_parts, part_radius, &mut meshes, &mut materials),
                children![
                    atom_part(0.0 * angle_step, atom_radius, part_radius, &mut meshes, &mut materials),
                    atom_part(1.0 * angle_step, atom_radius, part_radius, &mut meshes, &mut materials),
                    atom_part(2.0 * angle_step, atom_radius, part_radius, &mut meshes, &mut materials),
                    atom_part(3.0 * angle_step, atom_radius, part_radius, &mut meshes, &mut materials),
                    atom_part(4.0 * angle_step, atom_radius, part_radius, &mut meshes, &mut materials),
                    atom_part(5.0 * angle_step, atom_radius, part_radius, &mut meshes, &mut materials),
                ]
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
