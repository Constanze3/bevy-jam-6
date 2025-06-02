use std::time::Duration;

use bevy::{
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith},
    prelude::*,
};
use bevy_rapier2d::prelude::*;

use crate::{AppSystems, PausableSystems, asset_tracking::LoadResource, screens::Screen};

use super::player::Player;

const PARTICLE_LOCAL_Z: f32 = -2.0;
const ARROWS_LOCAL_Z: f32 = -3.0;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<ParticleAssets>();
    app.load_resource::<ParticleAssets>();

    app.register_type::<Particle>();

    app.add_systems(
        Update,
        particle_collision_handler
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );

    app.add_observer(player_particle_collision);
    app.add_observer(particle_particle_collision);
    app.add_observer(split_particle);

    // invincibility
    app.add_systems(
        Update,
        (
            setup_invincibility
                .in_set(AppSystems::Update)
                .run_if(in_state(Screen::Gameplay)),
            tick_invincibility
                .in_set(AppSystems::Update)
                .in_set(PausableSystems),
        ),
    );
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct ParticleAssets {
    #[dependency]
    arrow_image: Handle<Image>,
    arrow_offset: f32,
    arrow_scale: f32,
    invincibility_duration: Duration,
    invincible_material: Handle<ColorMaterial>,
}

impl FromWorld for ParticleAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        let arrow_image = assets.load("images/arrow.png");

        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
        let invincible_material = materials.add(ColorMaterial::from_color(Color::Srgba(
            Srgba::hex("f7bd1d").unwrap(),
        )));

        Self {
            arrow_image,
            arrow_offset: 20.0,
            arrow_scale: 0.02,
            invincibility_duration: Duration::from_secs_f32(0.5),
            invincible_material,
        }
    }
}

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(no_field_bounds)]
pub struct Particle {
    pub radius: f32,
    pub initial_velocity: Vec2,
    pub sub_particles: Vec<Particle>,
    pub mesh: Handle<Mesh>,
    pub material: Handle<ColorMaterial>,
}

#[derive(Component)]
pub struct Invincible(Timer);

impl Invincible {
    fn new(duration: Duration) -> Self {
        Self(Timer::new(duration, TimerMode::Once))
    }
}

#[derive(Component)]
pub struct Arrows;

pub fn particle_bundle(
    translation: Vec2,
    particle: Particle,
    particle_assets: &ParticleAssets,
) -> impl Bundle {
    let arrow_transforms = {
        let mut result = Vec::new();
        for sub_particle in particle.sub_particles.iter() {
            let direction = sub_particle.initial_velocity.normalize();
            let angle = direction.y.atan2(direction.x);
            let position = Vec2::ZERO + direction * particle_assets.arrow_offset;

            result.push(Transform {
                translation: position.extend(0.0),
                rotation: Quat::from_rotation_z(angle),
                scale: Vec3::ONE * particle_assets.arrow_scale,
            });
        }

        result
    };

    (
        Name::new("Particle Bundle"),
        Transform::default(),
        Visibility::default(),
        children![
            (
                Name::new("Arrows"),
                Transform::from_translation(translation.extend(ARROWS_LOCAL_Z)),
                Visibility::default(),
                Children::spawn(SpawnWith({
                    let arrow = particle_assets.arrow_image.clone();

                    move |parent: &mut RelatedSpawner<ChildOf>| {
                        for transform in arrow_transforms {
                            parent.spawn((
                                Name::new("Arrow"),
                                Sprite::from_image(arrow.clone()),
                                transform,
                            ));
                        }
                    }
                })),
                Arrows,
            ),
            (
                Name::new("Particle"),
                Transform::from_translation(translation.extend(PARTICLE_LOCAL_Z)),
                Mesh2d(particle.mesh.clone()),
                MeshMaterial2d(particle.material.clone()),
                RigidBody::Dynamic,
                Collider::ball(particle.radius),
                ActiveEvents::COLLISION_EVENTS,
                Velocity {
                    linvel: particle.initial_velocity,
                    angvel: 0.0,
                },
                particle,
                Invincible::new(particle_assets.invincibility_duration)
            )
        ],
    )
}

// System that triggers specialized collision events.
fn particle_collision_handler(
    mut collision_events: EventReader<CollisionEvent>,
    query: Query<(Option<&Particle>, Option<&Player>), With<RigidBody>>,
    mut commands: Commands,
) {
    for event in collision_events.read() {
        let CollisionEvent::Started(e1, e2, _) = *event else {
            return;
        };

        let Ok((e1_particle, e1_player)) = query.get(e1) else {
            return;
        };

        let Ok((e2_particle, e2_player)) = query.get(e2) else {
            return;
        };

        if e1_player.is_some() && e2_particle.is_some() {
            commands.trigger(PlayerParticleCollisionEvent {
                player: e1,
                particle: e2,
            });
            return;
        }

        if e2_player.is_some() && e1_particle.is_some() {
            commands.trigger(PlayerParticleCollisionEvent {
                player: e2,
                particle: e1,
            });
            return;
        }

        if e1_particle.is_some() && e2_particle.is_some() {
            commands.trigger(ParticleParticleCollisionEvent {
                particle1: e1,
                particle2: e2,
            });
            return;
        }
    }
}

#[derive(Event)]
#[allow(dead_code)]
pub struct PlayerParticleCollisionEvent {
    pub player: Entity,
    pub particle: Entity,
}

fn player_particle_collision(
    trigger: Trigger<PlayerParticleCollisionEvent>,
    mut commands: Commands,
) {
    commands.trigger(ParticleSplitEvent(trigger.particle));
}

#[derive(Event)]
pub struct ParticleParticleCollisionEvent {
    pub particle1: Entity,
    pub particle2: Entity,
}

fn particle_particle_collision(
    trigger: Trigger<ParticleParticleCollisionEvent>,
    mut commands: Commands,
) {
    commands.trigger(ParticleSplitEvent(trigger.particle1));
    commands.trigger(ParticleSplitEvent(trigger.particle2));
}

#[derive(Event)]
pub struct ParticleSplitEvent(pub Entity);

fn split_particle(
    trigger: Trigger<ParticleSplitEvent>,
    mut particle_query: Query<
        (Option<&Invincible>, &ChildOf, &Transform, &mut Particle),
        Without<Player>,
    >,
    player_query: Query<&Player>,
    mut commands: Commands,
    particle_assets: Res<ParticleAssets>,
) {
    let (invincible, bundle, transform, mut particle) = particle_query.get_mut(trigger.0).unwrap();

    if invincible.is_some() {
        return;
    }

    let player = player_query.single().unwrap();

    let position = transform.translation;

    let sub_particles = std::mem::take(&mut particle.sub_particles);
    for sub_particle in sub_particles {
        let offset_distance = particle.radius + sub_particle.radius.max(player.radius);
        let offset = sub_particle.initial_velocity.normalize() * offset_distance;

        let spawn_position = position.xy() + offset;
        commands.spawn(particle_bundle(
            spawn_position,
            sub_particle,
            particle_assets.as_ref(),
        ));
    }

    commands.entity(bundle.0).despawn();
}

fn setup_invincibility(
    mut query: Query<&mut MeshMaterial2d<ColorMaterial>, Added<Invincible>>,
    particle_assets: Res<ParticleAssets>,
) {
    for mut material in query.iter_mut() {
        material.0 = particle_assets.invincible_material.clone();
    }
}

fn tick_invincibility(
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut Invincible,
        &Particle,
        &mut MeshMaterial2d<ColorMaterial>,
    )>,
    mut commands: Commands,
) {
    for (entity, mut invincible, particle, mut material) in query.iter_mut() {
        invincible.0.tick(time.delta());

        if invincible.0.just_finished() {
            commands.entity(entity).remove::<Invincible>();
            material.0 = particle.material.clone();
        }
    }
}
