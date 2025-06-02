//! Spawn the main level.

use bevy::prelude::*;
use bevy_rapier2d::prelude::{Collider, RigidBody};

use crate::{asset_tracking::LoadResource, demo::player::player, screens::Screen};

use super::{
    indicator::drag_indicator,
    particle::{Particle, ParticleAssets, particle_bundle},
};

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
    particle_assets: Res<ParticleAssets>,
) {
    let particle_radius = 15.0;
    let particle_mesh = meshes.add(Circle::new(particle_radius));
    let particle_material = materials.add(Color::Srgba(Srgba::hex("0f95e2").unwrap()));

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
            obstacle(vec2(100.0, 0.0), 50.0, &mut meshes, &mut materials),
            particle_bundle(
                vec2(-100.0, 0.0),
                Particle {
                    radius: particle_radius,
                    initial_velocity: Vec2::ZERO,
                    sub_particles: vec![
                        Particle {
                            radius: particle_radius,
                            initial_velocity: vec2(0.0, -200.0),
                            sub_particles: vec![],
                            mesh: particle_mesh.clone(),
                            material: particle_material.clone()
                        },
                        Particle {
                            radius: particle_radius,
                            initial_velocity: vec2(0.0, 200.0),
                            sub_particles: vec![],
                            mesh: particle_mesh.clone(),
                            material: particle_material.clone()
                        }
                    ],
                    mesh: particle_mesh.clone(),
                    material: particle_material.clone()
                },
                particle_assets.as_ref()
            )
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
