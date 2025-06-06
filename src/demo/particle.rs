use std::time::Duration;

use bevy::{
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith, system::QueryLens},
    prelude::*,
};
use bevy_mod_picking::{prelude::On, PickableBundle};
use bevy_rapier2d::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    audio::sound_effect,
    external::maybe::Maybe,
    physics::{CollisionHandlers, find_rigidbody_ancestor},
    screens::Screen,
};

use super::{
    killer::Killer,
    player::{Player, TimeSpeed},
};

const PARTICLE_LOCAL_Z: f32 = -2.0;
const ARROWS_LOCAL_Z: f32 = -3.0;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<ParticleAssets>();
    app.load_resource::<ParticleAssets>();

    app.register_type::<Particle>();

    app.add_event::<ParticleSplitEvent>();

    app.add_systems(
        PostUpdate,
        (
            particle_collision_handler
                .in_set(CollisionHandlers)
                .in_set(PausableSystems)
                .run_if(in_state(Screen::Gameplay)),
            split_particle
                .after(CollisionHandlers)
                .run_if(in_state(Screen::Gameplay)),
        ),
    );

    app.add_observer(player_particle_collision);
    app.add_observer(particle_particle_collision);

    // Invincibility

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

    // Arrows

    app.register_type::<Arrows>();
    app.register_type::<ArrowsOf>();

    app.add_systems(
        Update,
        (
            setup_arrows_relationship
                .in_set(AppSystems::Update)
                .run_if(in_state(Screen::Gameplay)),
            move_arrows
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
    #[dependency]
    pop_sound: Handle<AudioSource>,
}

impl FromWorld for ParticleAssets {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
        let invincible_material = materials.add(ColorMaterial::from_color(Color::Srgba(
            Srgba::hex("f7bd1d").unwrap(),
        )));

        let assets = world.resource::<AssetServer>();

        Self {
            arrow_image: assets.load("images/arrow.png"),
            arrow_offset: 3.0,
            arrow_scale: 0.02,
            invincibility_duration: Duration::from_secs_f32(0.5),
            invincible_material,
            pop_sound: assets.load("audio/sound_effects/pop.ogg"),
        }
    }
}

#[derive(Debug, Clone, Reflect, PartialEq, Eq)]
pub enum ParticleKind {
    Normal,
    Killer,
}

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(no_field_bounds)]
pub struct Particle {
    pub kind: ParticleKind,
    pub radius: f32,
    pub initial_velocity: Vec2,
    pub subparticles: Vec<Particle>,
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

#[derive(Component, Reflect)]
#[reflect(Component)]
#[relationship_target(relationship = ArrowsOf)]
pub struct Arrows(Entity);

#[derive(Component, Reflect)]
#[reflect(Component)]
#[relationship(relationship_target = Arrows)]
pub struct ArrowsOf(Entity);

pub fn particle_bundle(
    translation: Vec2,
    with_invincibility: bool,
    particle: Particle,
    particle_assets: &ParticleAssets,
) -> impl Bundle {
    let arrow_transforms = {
        let mut result = Vec::new();
        for sub_particle in particle.subparticles.iter() {
            let direction = sub_particle.initial_velocity.normalize();
            let angle = direction.y.atan2(direction.x);

            let offset = particle.radius + particle_assets.arrow_offset;
            let position = Vec2::ZERO + direction * offset;

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
        // PickableBundle::default(),
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
            ),
            (
                Name::new("Particle"),
                Transform::from_translation(translation.extend(PARTICLE_LOCAL_Z)),
                Mesh2d(particle.mesh.clone()),
                MeshMaterial2d(particle.material.clone()),
                RigidBody::Dynamic,
                Collider::ball(particle.radius),
                children![(
                    Name::new("Particle Sensor"),
                    ActiveEvents::COLLISION_EVENTS,
                    ActiveCollisionTypes::DYNAMIC_DYNAMIC,
                    CollisionGroups::new(Group::GROUP_3, Group::GROUP_3),
                    Collider::ball(particle.radius),
                    Sensor
                )],
                Velocity {
                    linvel: particle.initial_velocity,
                    angvel: 0.0,
                },
                Maybe({
                    if with_invincibility {
                        Some((
                            Invincible::new(particle_assets.invincibility_duration),
                            CollisionGroups::new(Group::GROUP_2, Group::GROUP_1 | Group::GROUP_2),
                        ))
                    } else {
                        None
                    }
                }),
                Maybe({
                    if with_invincibility {
                        None
                    } else {
                        Some(CollisionGroups::new(Group::GROUP_3, Group::GROUP_1))
                    }
                }),
                Maybe((particle.kind == ParticleKind::Killer).then_some(Killer)),
                particle,
            )
        ],
    )
}

// System that triggers specialized collision events.
pub fn particle_collision_handler(
    mut collision_events: EventReader<CollisionEvent>,
    mut query: Query<(
        Option<&Particle>,
        Option<&Player>,
        Option<&RigidBody>,
        &ChildOf,
    )>,
    mut commands: Commands,
) {
    for event in collision_events.read() {
        let CollisionEvent::Started(e1, e2, _) = *event else {
            return;
        };

        let mut helper_lens: QueryLens<(Option<&RigidBody>, &ChildOf)> = query.transmute_lens();
        let helper_query = helper_lens.query();
        let e1 = find_rigidbody_ancestor(e1, &helper_query).unwrap();
        let e2 = find_rigidbody_ancestor(e2, &helper_query).unwrap();

        let (e1_particle, e1_player, _, _) = query.get(e1).unwrap();
        let (e2_particle, e2_player, _, _) = query.get(e2).unwrap();

        if e1_player.is_some() && e2_particle.is_some() {
            commands.trigger(PlayerParticleCollisionEvent { particle: e2 });
            return;
        }

        if e2_player.is_some() && e1_particle.is_some() {
            commands.trigger(PlayerParticleCollisionEvent { particle: e1 });
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
pub struct PlayerParticleCollisionEvent {
    pub particle: Entity,
}

#[allow(unused)]
fn player_particle_collision(
    trigger: Trigger<PlayerParticleCollisionEvent>,
    mut player_query: Query<(&mut Player, &mut Velocity)>,
    mut particle_query: Query<Option<&Invincible>, (With<Particle>, Without<Player>)>,
    mut timestep_mode: ResMut<TimestepMode>,
    time_speed: Res<TimeSpeed>,
    particle_assets: Res<ParticleAssets>,
    mut events: EventWriter<ParticleSplitEvent>,
    mut commands: Commands,
) {
    let invincible = particle_query.get(trigger.particle).unwrap();
    if invincible.is_some() {
        return;
    }

    let (mut player, mut velocity) = player_query.single_mut().unwrap();
    player.can_move = true;

    // velocity.linvel = Vec2::ZERO;

    if let TimestepMode::Variable { time_scale, .. } = timestep_mode.as_mut() {
        *time_scale = time_speed.slow;
    }

    events.write(ParticleSplitEvent(trigger.particle));
    commands.spawn(sound_effect(particle_assets.pop_sound.clone()));
}

#[derive(Event)]
pub struct ParticleParticleCollisionEvent {
    pub particle1: Entity,
    pub particle2: Entity,
}

fn particle_particle_collision(
    trigger: Trigger<ParticleParticleCollisionEvent>,
    particle_assets: Res<ParticleAssets>,
    mut events: EventWriter<ParticleSplitEvent>,
    mut commands: Commands,
) {
    events.write(ParticleSplitEvent(trigger.particle1));
    events.write(ParticleSplitEvent(trigger.particle2));

    commands.spawn(sound_effect(particle_assets.pop_sound.clone()));
}

#[derive(Event)]
pub struct ParticleSplitEvent(pub Entity);

fn split_particle(
    mut events: EventReader<ParticleSplitEvent>,
    mut particle_query: Query<
        (Option<&Invincible>, &ChildOf, &Transform, &mut Particle),
        Without<Player>,
    >,
    parent_query: Query<Option<&ChildOf>, (Without<Player>, Without<Particle>)>,
    player_query: Query<&Player>,
    mut commands: Commands,
    particle_assets: Res<ParticleAssets>,
) {
    for event in events.read() {
        let (invincible, bundle, transform, mut particle) =
            particle_query.get_mut(event.0).unwrap();

        let bundle = bundle.0;

        if invincible.is_some() {
            return;
        }

        let player = player_query.single().unwrap();

        let position = transform.translation;
        let parent = parent_query.get(bundle).unwrap();

        let sub_particles = std::mem::take(&mut particle.subparticles);
        for sub_particle in sub_particles {
            let offset_distance = particle.radius + 2.0 * player.radius + sub_particle.radius;
            let offset = sub_particle.initial_velocity.normalize() * offset_distance;

            let spawn_position = position.xy() + offset;
            commands.spawn((
                particle_bundle(spawn_position, true, sub_particle, particle_assets.as_ref()),
                // The subparticle will have the same parent as the particle if it has a parent.
                Maybe(parent.map(|parent| ChildOf(parent.0))),
            ));
        }

        commands.entity(bundle).despawn();
    }
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
            commands
                .entity(entity)
                .remove::<Invincible>()
                .remove::<CollisionGroups>()
                .insert(CollisionGroups::new(Group::GROUP_3, Group::GROUP_1));

            material.0 = particle.material.clone();
        }
    }
}

fn setup_arrows_relationship(
    particle_query: Query<(Entity, &ChildOf), Added<Particle>>,
    parent_query: Query<&Children>,
    mut commands: Commands,
) {
    for (particle_entity, parent_entity) in particle_query.iter() {
        let children = parent_query.get(parent_entity.0).unwrap();

        for child in children.iter() {
            if child != particle_entity {
                commands.entity(child).insert(ArrowsOf(particle_entity));
            }
        }
    }
}

fn move_arrows(
    particle_query: Query<(&Transform, &Arrows), With<Particle>>,
    mut arrows_query: Query<&mut Transform, Without<Particle>>,
) {
    for (particle_transform, arrows) in particle_query.iter() {
        let mut arrows_transform = arrows_query.get_mut(arrows.0).unwrap();
        arrows_transform.translation = particle_transform.translation.xy().extend(ARROWS_LOCAL_Z);
    }
}
