//! Spawn the main level.

use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};
use bevy_rapier2d::prelude::{Collider, Restitution, RigidBody};

use crate::{asset_tracking::LoadResource, demo::player::player, screens::Screen};

use super::{
    indicator::drag_indicator,
    particle::{Particle, ParticleAssets, particle_bundle},
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelAssets>();
    app.add_systems(Update, update_screen_bounds);
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
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.single().expect("PrimaryWindow not found");

    // Spawn screen bounds first
    spawn_screen_bounds(&mut commands, window);

    let particle_radius = 15.0;
    let particle_mesh = meshes.add(Circle::new(particle_radius));
    let particle_material = materials.add(Color::Srgba(Srgba::hex("0f95e2").unwrap()));

    let particle_radius2 = 40.0;
    let particle_mesh2 = meshes.add(Circle::new(particle_radius2));

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
                            radius: particle_radius2,
                            initial_velocity: vec2(0.0, -200.0),
                            sub_particles: vec![],
                            mesh: particle_mesh2.clone(),
                            material: particle_material.clone()
                        },
                        Particle {
                            radius: particle_radius2,
                            initial_velocity: vec2(0.0, 200.0),
                            sub_particles: vec![],
                            mesh: particle_mesh2.clone(),
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
        Restitution::coefficient(0.5),
    ));
    commands.spawn((
        Name::new("Right Wall"),
        Transform::from_xyz(width / 2.0, 0.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(1.0, height / 2.0),
        Restitution::coefficient(0.5),
    ));
    commands.spawn((
        Name::new("Top Wall"),
        Transform::from_xyz(0.0, height / 2.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(width / 2.0, 1.0),
        Restitution::coefficient(0.5),
    ));
    commands.spawn((
        Name::new("Bottom Wall"),
        Transform::from_xyz(0.0, -height / 2.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(width / 2.0, 1.0),
        Restitution::coefficient(0.5),
    ));
}

// Add this system to handle window resizing
fn update_screen_bounds(
    mut resize_events: EventReader<WindowResized>,
    mut collider_query: Query<(&Name, &mut Transform, &mut Collider)>,
) {
    for event in resize_events.read() {
        let width = event.width;
        let height = event.height;
        let half_width = width / 2.0;
        let half_height = height / 2.0;

        for (name, mut transform, mut collider) in collider_query.iter_mut() {
            match name.as_str() {
                "Left Wall" | "Right Wall" => {
                    // Update vertical walls
                    *collider = Collider::cuboid(1.0, half_height);
                    // Update positions
                    if name.as_str() == "Left Wall" {
                        transform.translation.x = -half_width;
                    } else {
                        transform.translation.x = half_width;
                    }
                }
                "Top Wall" | "Bottom Wall" => {
                    // Update horizontal walls
                    *collider = Collider::cuboid(half_width, 1.0);
                    // Update positions
                    if name.as_str() == "Top Wall" {
                        transform.translation.y = half_height;
                    } else {
                        transform.translation.y = -half_height;
                    }
                }
                _ => {}
            }
        }
    }
}
