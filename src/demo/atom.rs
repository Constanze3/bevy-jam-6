use bevy::{math::VectorSpace, prelude::*};
use bevy_rapier2d::prelude::*;


#[derive(Component)]
pub struct Atom;

#[derive(Component)]
pub struct AtomPart;

#[derive(Component)]
pub struct Breakable;

#[derive(Component)]
pub struct BreakDirection(pub Vec2);

// #[derive(Bundle)]
// pub struct DirectionArrowBundle {
//     name: Name,
//     sprite: Sprite,
//     texture: TextureAtlasLayout,
//     transform: Transform,
//     visibility: Visibility,
//     global_transform: GlobalTransform,
// }

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

pub fn atom_seed(
    translation: Vec2,
    radius: f32,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> impl Bundle {
    let mesh = meshes.add(Circle::new(radius));
    let material = materials.add(Color::hsl(200.0, 0.7, 0.6));
    (
        Name::new("AtomSeed"),
        Atom,
        AtomPart, // So it can be detected by the collision system
        Breakable,
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::from_translation(translation.extend(0.0)),
        RigidBody::Dynamic,
        Collider::ball(radius),
        Velocity::zero(),
        Visibility::Visible,
    )
}

pub fn atom_part_with_arrow(
    angle: f32,
    origin: Vec2,
    radius: f32,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    asset_server: &AssetServer,
) -> impl Bundle {
    let break_direction = Vec2::new(angle.cos(), angle.sin());
    let offset = break_direction * radius;
    (
        Name::new("AtomPart"),
        AtomPart,
        Breakable,
        BreakDirection(break_direction),
        Mesh2d(meshes.add(Circle::new(radius))),
        MeshMaterial2d(materials.add(Color::hsl(200.0, 0.7, 0.6))),
        Transform::from_translation((origin + offset).extend(0.0)),
        Collider::ball(radius),
        Velocity::zero(),
        Visibility::Visible,
        children![
            direction_arrow_bundle(angle, asset_server)
        ],
    )
}

// System to handle breaking the atom seed into parts
pub fn atom_chain_reaction(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut collision_events: EventReader<CollisionEvent>,
    atom_part_query: Query<(Entity, &Transform, Option<&BreakDirection>, Option<&Name>), With<Breakable>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let num_parts = 6;
    let part_radius = 10.0;
    let angle_step = std::f32::consts::TAU / num_parts as f32;

    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            for (entity, transform, break_dir, name) in &atom_part_query {
                if *e1 == entity || *e2 == entity {
                    let pos = transform.translation.truncate();

                    // If this is the AtomSeed, break it into parts
                    if let Some(name) = name {
                        if name.as_str() == "AtomSeed" {
                            commands.entity(entity).despawn();

                            for i in 0..num_parts {
                                let angle = i as f32 * angle_step;
                                let part = atom_part_with_arrow(
                                    angle,
                                    pos,
                                    part_radius,
                                    &mut meshes,
                                    &mut materials,
                                    &asset_server
                                );
                                commands.spawn(part);
                            }
                        }
                    }
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

pub fn direction_arrow_bundle(
    angle: f32,
    asset_server: &AssetServer,
) -> impl Bundle {
    let texture = asset_server.load("images/arrow.png");
    (
        Name::new("Direction Arrow"),
        Sprite::from_image(
            texture,
        ),
        Transform {
            translation: Vec3::new(0.0, 0.0, 1.0),
            rotation: Quat::from_rotation_z(angle),
            ..default()
        },
        Visibility::Visible,
    )
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (atom_chain_reaction, cleanup_arrows));
}
