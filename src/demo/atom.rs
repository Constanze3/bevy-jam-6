use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use super::{indicator::DragIndicator, input::InputController, player::Player};

#[derive(Component)]
pub struct Atom;

#[derive(Component)]
pub struct AtomPart {
    pub parent: Entity,
}

#[derive(Component)]
pub struct Breakable;

#[derive(Component)]
pub struct BreakDirection(pub Vec2);

#[derive(Bundle)]
pub struct DirectionArrowBundle {
    name: Name,
    sprite: Sprite,
    texture: Handle<Image>,
    transform: Transform,
    visibility: Visibility,
    global_transform: GlobalTransform,
}

// Add timer component and system for arrow cleanup
#[derive(Component)]
struct ExpirationTimer(Timer);

fn cleanup_arrows(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ExpirationTimer)>,
) {
    for (entity, mut timer) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn spawn_atom(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    position: Vec2,
    num_parts: usize,
    radius: f32,
    part_radius: f32,
) -> Entity {
    let atom_entity = commands
        .spawn((
            Name::new("Atom"),
            Atom,
            Transform::from_translation(position.extend(0.0)),
            GlobalTransform::default(),
        ))
        .id();

    let angle_step = std::f32::consts::TAU / num_parts as f32;
    for i in 0..num_parts {
        let angle = i as f32 * angle_step;
        let offset = Vec2::new(angle.cos(), angle.sin()) * radius;
        let part_entity = commands
            .spawn((
                Name::new("AtomPart"),
                AtomPart { parent: atom_entity },
                Breakable,
                Mesh2d(meshes.add(Circle::new(part_radius))),
                MeshMaterial2d(materials.add(Color::hsl(200.0, 0.7, 0.6))),
                Transform::from_translation(offset.extend(0.0)),
                RigidBody::Dynamic,
                Collider::ball(part_radius),
                // Add other physics components as needed
            ))
            .id();
        commands.entity(atom_entity).add_child(part_entity);
    }

    atom_entity
}


fn atom_chain_reaction(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut collision_events: EventReader<CollisionEvent>,
    atom_part_query: Query<(Entity, &AtomPart, &Transform, &BreakDirection), With<Breakable>>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            for (entity, atom_part, transform, break_dir) in &atom_part_query {
                if *e1 == entity || *e2 == entity {
                    let pos = transform.translation.truncate();
                    let angle = break_dir.0.y.atan2(break_dir.0.x);

                    // Spawn the arrow sprite
                    spawn_direction_arrow(
                        &mut commands,
                        &asset_server,
                        pos,
                        angle,
                        Vec2::new(30.0, 15.0), // Adjust size as needed
                    );

                    // Break the atom part
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}


fn spawn_direction_arrow(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec2,
    angle: f32,
    size: Vec2,
) -> Entity {
    commands.spawn((
        DirectionArrowBundle {
            name: Name::new("Direction Arrow"),
            sprite: Sprite {
                custom_size: Some(size),
                ..default()
            },
            texture: asset_server.load("images/arrow.png"),
            transform: Transform {
                translation: position.extend(1.0),
                rotation: Quat::from_rotation_z(angle),
                ..default()
            },
            visibility: Visibility::Visible,
            global_transform: GlobalTransform::default(),
        },
        ExpirationTimer(Timer::from_seconds(0.5, TimerMode::Once)),
    )).id()
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (atom_chain_reaction, cleanup_arrows));
}
