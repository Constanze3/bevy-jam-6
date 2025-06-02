use bevy::{
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith},
    prelude::*,
};
use bevy_rapier2d::prelude::*;

use crate::{AppSystems, PausableSystems, asset_tracking::LoadResource};

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
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct ParticleAssets {
    #[dependency]
    arrow_image: Handle<Image>,
    arrow_offset: f32,
    arrow_scale: f32,
}

impl FromWorld for ParticleAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            arrow_image: assets.load("images/arrow.png"),
            arrow_offset: 20.0,
            arrow_scale: 0.02,
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
            )
        ],
    )
}

#[derive(Event)]
#[allow(dead_code)]
pub struct PlayerParticleCollisionEvent {
    pub player: Entity,
    pub particle: Entity,
}

#[derive(Event)]
pub struct ParticleParticleCollisionEvent {
    pub particle1: Entity,
    pub particle2: Entity,
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

fn player_particle_collision(
    trigger: Trigger<PlayerParticleCollisionEvent>,
    mut particle_query: Query<(&ChildOf, &Transform, &mut Particle)>,
    mut commands: Commands,
    particle_assets: Res<ParticleAssets>,
) {
    split_particle(
        trigger.particle,
        &mut particle_query,
        &mut commands,
        particle_assets.as_ref(),
    );
}

fn particle_particle_collision(
    trigger: Trigger<ParticleParticleCollisionEvent>,
    mut particle_query: Query<(&ChildOf, &Transform, &mut Particle)>,
    mut commands: Commands,
    particle_assets: Res<ParticleAssets>,
) {
    split_particle(
        trigger.particle1,
        &mut particle_query,
        &mut commands,
        particle_assets.as_ref(),
    );

    split_particle(
        trigger.particle2,
        &mut particle_query,
        &mut commands,
        particle_assets.as_ref(),
    );
}

fn split_particle(
    entity: Entity,
    particle_query: &mut Query<(&ChildOf, &Transform, &mut Particle)>,
    commands: &mut Commands,
    particle_assets: &ParticleAssets,
) {
    let (bundle, transform, mut particle) = particle_query.get_mut(entity).unwrap();

    let position = transform.translation;

    let sub_particles = std::mem::take(&mut particle.sub_particles);
    for sub_particle in sub_particles {
        let offset =
            sub_particle.initial_velocity.normalize() * (particle.radius + sub_particle.radius);

        let spawn_position = position.xy() + offset;
        commands.spawn(particle_bundle(
            spawn_position,
            sub_particle,
            particle_assets,
        ));
    }

    commands.entity(bundle.0).despawn();
}
