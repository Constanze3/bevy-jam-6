//! Player-specific behavior.

use avian2d::prelude::{
    Collider, ExternalImpulse, GravityScale, LinearVelocity, RigidBody, TransformExtrapolation,
    TransformInterpolation,
};
use bevy::prelude::*;

use crate::demo::movement::ScreenWrap;

use super::input::InputEvent;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();

    app.add_observer(handle_input);
}

/// The player character.
pub fn player(
    radius: f32,
    force_scalar: f32,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) -> impl Bundle {
    let mesh = meshes.add(Circle::new(25.0));
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
        GravityScale(0.0),
        Collider::circle(radius),
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
    trigger: Trigger<InputEvent>,
    mut query: Query<(&Player, &mut ExternalImpulse, &mut LinearVelocity)>,
) {
    let (player, mut external_impulse, mut velocity) = query.single_mut().unwrap();

    **velocity = Vec2::ZERO;

    // apply force to player
    external_impulse.apply_impulse(player.force_scalar * trigger.vector);
}
