//! Player-specific behavior.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use super::input::InputEvent;
use crate::{AppSystems, PausableSystems, demo::movement::ScreenWrap};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();

    app.add_systems(
        Update,
        handle_input
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

/// The player character.
pub fn player(
    radius: f32,
    force_scalar: f32,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) -> impl Bundle {
    let mesh = meshes.add(Circle::new(radius));
    let material = materials.add(Color::hsl(0.0, 0.95, 0.7));

    (
        Name::new("Player"),
        Player {
            radius,
            force_scalar,
        },
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::default(),
        RigidBody::Dynamic,
        Ccd::enabled(),
        Sleeping::disabled(),
        Collider::ball(radius),
        Velocity::default(),
        ExternalImpulse::default(),
        ScreenWrap,
    )
}

#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct Player {
    pub radius: f32,
    pub force_scalar: f32,
}

fn handle_input(
    mut events: EventReader<InputEvent>,
    mut query: Query<(&Player, &mut ExternalImpulse, &mut Velocity)>,
) {
    for event in events.read() {
        for (player, mut external_impulse, mut velocity) in query.iter_mut() {
            velocity.linvel = Vec2::ZERO;

            // apply force to player
            external_impulse.impulse = player.force_scalar * event.vector;
        }
    }
}
