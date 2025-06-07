//!
//! Collision groups are set up as follows:
//!
//! group_1 = terrain
//! group_2 = player & invincible particles
//! group_3 = normal particles
//!
//! group_2 collides with group_1 and group_2.
//! group_3 collides only with group_1.
//!
//! The player and the particles have a group_3 sensor.

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use super::input::InputEvent;
use crate::{
    AppSystems, PausableSystems, asset_tracking::LoadResource, audio::sound_effect, screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();
    app.register_type::<TimeSpeed>();

    app.load_resource::<PlayerAssets>();
    app.init_resource::<TimeSpeed>();

    app.add_systems(
        Update,
        handle_input
            .in_set(AppSystems::Update)
            .in_set(PausableSystems)
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Asset, Resource, Clone, Reflect)]
#[reflect(Resource)]
struct PlayerAssets {
    #[dependency]
    shoot_sound: Handle<AudioSource>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        Self {
            shoot_sound: assets.load::<AudioSource>("audio/sound_effects/shoot.ogg"),
        }
    }
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
    translation: Vec2,
    radius: f32,
    force_scalar: f32,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) -> impl Bundle {
    let mesh = meshes.add(Circle::new(radius));
    let material = materials.add(Color::hsl(0.0, 0.0, 0.0));

    (
        Name::new("Player"),
        Transform::from_translation(translation.extend(0.0)),
        Player {
            radius,
            force_scalar,
            can_move: true,
        },
        (
            Mesh2d(mesh),
            MeshMaterial2d(material),
            RigidBody::Dynamic,
            Ccd::enabled(),
            Sleeping::disabled(),
            Collider::ball(radius),
            children![(
                Name::new("Player Sensor"),
                ActiveEvents::COLLISION_EVENTS,
                CollisionGroups::new(Group::GROUP_3, Group::GROUP_3),
                Collider::ball(radius),
                Sensor
            )],
            CollisionGroups::new(Group::GROUP_2, Group::GROUP_1 | Group::GROUP_2),
            Restitution::coefficient(0.5),
            Velocity::default(),
            ExternalImpulse::default(),
        ),
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
    player_assets: Res<PlayerAssets>,
    mut commands: Commands,
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

        commands.spawn(sound_effect(player_assets.shoot_sound.clone()));

        player.can_move = false;

        if let TimestepMode::Variable { time_scale, .. } = timestep_mode.as_mut() {
            *time_scale = time_speed.normal;
        }
    }
}
