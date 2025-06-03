//! Player-specific behavior.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use super::input::InputEvent;
use crate::{AppSystems, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();
    app.register_type::<TimeSpeed>();

    app.init_resource::<TimeSpeed>();

    app.add_systems(
        Update,
        handle_input
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct TimeSpeed {
    pub slow: f32,
    pub normal: f32,
}

impl Default for TimeSpeed {
    fn default() -> Self {
        Self {
            slow: 0.1,
            normal: 1.0,
        }
    }
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
            can_move: true,
        },
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::default(),
        RigidBody::Dynamic,
        Ccd::enabled(),
        Sleeping::disabled(),
        Collider::ball(radius),
        Restitution::coefficient(0.5),
        ActiveEvents::COLLISION_EVENTS,
        ActiveCollisionTypes::default() | ActiveCollisionTypes::DYNAMIC_DYNAMIC,
        Velocity::default(),
        ExternalImpulse::default(),
    )
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Player {
    pub radius: f32,
    pub force_scalar: f32,
    pub can_move: bool,
}

fn handle_input(
    mut events: EventReader<InputEvent>,
    mut query: Query<(&mut Player, &mut ExternalImpulse, &mut Velocity)>,
    mut timestep_mode: ResMut<TimestepMode>,
    time_speed: Res<TimeSpeed>,
) {
    if query.is_empty() {
        return;
    }

    let (mut player, mut external_impulse, mut velocity) = query.single_mut().unwrap();

    if !player.can_move {
        return;
    }

    if let Some(event) = events.read().last() {
        velocity.linvel = Vec2::ZERO;
        external_impulse.impulse = player.force_scalar * event.vector;

        player.can_move = false;

        if let TimestepMode::Variable { time_scale, .. } = timestep_mode.as_mut() {
            *time_scale = time_speed.normal;
        }
    }
}
