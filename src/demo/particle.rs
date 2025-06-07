use std::time::Duration;

use arrows::{Arrows, ArrowsAssets, ArrowsConfig, ArrowsOf, arrows};
use bevy::{
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith, system::QueryLens},
    prelude::*,
};
// use bevy_hanabi::{EffectProperties, EffectSpawner};
use bevy_rapier2d::prelude::*;
use invincible::{Invincible, InvincibleRemoved};
use serde::{Deserialize, Serialize};

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    audio::sound_effect,
    external::maybe::Maybe,
    physics::{CollisionHandlerSystems, find_rigidbody_ancestor},
    screens::Screen,
};

use super::{
    killer::Killer,
    player::{Player, PlayerConfig},
    time_scale::{SetTimeScale, TimeScaleKind},
};

mod arrows;
mod invincible;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((invincible::plugin, arrows::plugin));

    app.init_resource::<ParticleConfig>();

    app.register_type::<ParticleAssets>();
    app.load_resource::<ParticleAssets>();

    app.add_event::<ParticleSplitEvent>();

    // Collision handling

    app.add_systems(
        PostUpdate,
        (
            particle_collision_handler
                .in_set(CollisionHandlerSystems)
                .in_set(PausableSystems)
                .run_if(in_state(Screen::Gameplay)),
            split_particle
                .after(CollisionHandlerSystems)
                .run_if(in_state(Screen::Gameplay)),
        ),
    );

    app.add_observer(player_particle_collision);
    app.add_observer(particle_particle_collision);
    app.add_observer(spawn_particle);

    // Invincibility

    app.add_systems(
        Update,
        (
            invincibility_added
                .in_set(AppSystems::Update)
                .run_if(in_state(Screen::Gameplay)),
            invincibility_removed
                .in_set(AppSystems::Update)
                .run_if(in_state(Screen::Gameplay)),
        ),
    );
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ParticleConfig {
    local_z: f32,
    invincibility_duration: Duration,
}

impl Default for ParticleConfig {
    fn default() -> Self {
        Self {
            local_z: -2.0,
            invincibility_duration: Duration::from_secs_f32(0.5),
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct ParticleAssets {
    #[dependency]
    pop_sound: Handle<AudioSource>,
    invincible_material: Handle<ColorMaterial>,
}

impl FromWorld for ParticleAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let pop_sound = assets.load("audio/sound_effects/pop.ogg");

        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
        let invincible_material = materials.add(ColorMaterial::from_color(Color::Srgba(
            Srgba::hex("f7bd1d").unwrap(),
        )));

        Self {
            pop_sound,
            invincible_material,
        }
    }
}

#[derive(Debug, Clone, Reflect, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ParticleKind {
    #[default]
    Normal,
    Killer,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Particle {
    pub kind: ParticleKind,
    pub radius: f32,
    pub color: Color,
    pub initial_velocity: Vec2,
    pub subparticles: Vec<Particle>,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            kind: ParticleKind::default(),
            radius: 20.0,
            color: Color::Srgba(Srgba::hex("0f95e2").unwrap()),
            initial_velocity: Vec2::ZERO,
            subparticles: Vec::new(),
        }
    }
}

pub fn particle(
    translation: Vec2,
    particle: Particle,
    particle_config: &ParticleConfig,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> impl Bundle {
    // TODO crate a cache for these
    let mesh = meshes.add(Circle::new(particle.radius));
    let material = materials.add(particle.color);

    (
        Name::new("Particle"),
        Transform::from_translation(translation.extend(particle_config.local_z)),
        Mesh2d(mesh),
        MeshMaterial2d(material),
        RigidBody::Dynamic,
        Collider::ball(particle.radius),
        children![(
            Name::new("Particle Sensor"),
            ActiveEvents::COLLISION_EVENTS,
            CollisionGroups::new(Group::GROUP_3, Group::GROUP_3),
            Collider::ball(particle.radius),
            Sensor
        )],
        Velocity {
            linvel: particle.initial_velocity,
            angvel: 0.0,
        },
        CollisionGroups::new(Group::GROUP_3, Group::GROUP_1),
        Maybe((particle.kind == ParticleKind::Killer).then_some(Killer)),
        particle,
    )
}

pub fn particle_bundle(
    translation: Vec2,
    particle: Particle,
    spawn_as_invincible: bool,
    particle_config: &ParticleConfig,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    arrows_config: &ArrowsConfig,
    arrows_assets: &ArrowsAssets,
) -> impl Bundle {
    let spawn_list = {
        let arrows_config = *arrows_config;
        let arrows_assets = arrows_assets.clone();
        let particle = particle.clone();

        move |spawner: &mut RelatedSpawner<ArrowsOf>| {
            spawner.spawn(arrows(
                translation,
                &particle,
                &arrows_config,
                &arrows_assets,
            ));
        }
    };

    (
        Arrows::spawn(SpawnWith(spawn_list)),
        Maybe(
            spawn_as_invincible.then_some(Invincible::new(particle_config.invincibility_duration)),
        ),
        self::particle(translation, particle, particle_config, meshes, materials),
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
    particle_assets: Res<ParticleAssets>,
    mut split_events: EventWriter<ParticleSplitEvent>,
    mut time_events: EventWriter<SetTimeScale>,
    mut commands: Commands,
) {
    let invincible = particle_query.get(trigger.particle).unwrap();
    if invincible.is_some() {
        return;
    }

    let (mut player, mut velocity) = player_query.single_mut().unwrap();
    player.can_move = true;

    // velocity.linvel = Vec2::ZERO;

    time_events.write(SetTimeScale(TimeScaleKind::Slowed));

    split_events.write(ParticleSplitEvent(trigger.particle));
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
    mut split_events: EventWriter<ParticleSplitEvent>,
    mut commands: Commands,
) {
    split_events.write(ParticleSplitEvent(trigger.particle1));
    split_events.write(ParticleSplitEvent(trigger.particle2));

    commands.spawn(sound_effect(particle_assets.pop_sound.clone()));
}

#[derive(Event)]
pub struct ParticleSplitEvent(pub Entity);

fn split_particle(
    mut events: EventReader<ParticleSplitEvent>,
    mut particle_query: Query<
        (
            Entity,
            Option<&Invincible>,
            &Transform,
            &mut Particle,
            Option<&ChildOf>,
        ),
        Without<Player>,
    >,
    player_config: Res<PlayerConfig>,
    mut commands: Commands,
    // mut effect: Query<
    //     (&mut EffectProperties, &mut EffectSpawner, &mut Transform),
    //     Without<Particle>,
    // >,
) {
    for event in events.read() {
        let (entity, invincible, transform, mut particle, parent) =
            particle_query.get_mut(event.0).unwrap();

        if invincible.is_some() {
            return;
        }

        let position = transform.translation;

        // let Ok((mut properties, mut effect_spawner, mut effect_transform)) = effect.single_mut()
        // else {
        //     return;
        // };

        // // This isn't the most accurate place to spawn the particle effect,
        // // but this is just for demonstration, so whatever.
        // effect_transform.translation = position;

        // // Pick a random particle color
        // let r = rand::random::<u8>();
        // let g = rand::random::<u8>();
        // let b = rand::random::<u8>();
        // let color = 0xFF000000u32 | (b as u32) << 16 | (g as u32) << 8 | (r as u32);
        // properties.set("spawn_color", color.into());

        // // Spawn the particles
        // effect_spawner.reset();

        let sub_particles = std::mem::take(&mut particle.subparticles);
        for subparticle in sub_particles {
            let offset_distance = particle.radius + 2.0 * player_config.radius + subparticle.radius;
            let offset = subparticle.initial_velocity.normalize() * offset_distance;

            let spawn_position = position.xy() + offset;

            commands.trigger(SpawnParticle {
                translation: spawn_position,
                particle: subparticle,
                spawn_with_invincible: true,
                parent: parent.map(|x| x.0),
            });
        }

        commands.entity(entity).despawn();
    }
}

#[derive(Event)]
pub struct SpawnParticle {
    pub translation: Vec2,
    pub particle: Particle,
    pub spawn_with_invincible: bool,
    pub parent: Option<Entity>,
}

fn spawn_particle(
    mut trigger: Trigger<SpawnParticle>,
    particle_config: Res<ParticleConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    arrows_config: Res<ArrowsConfig>,
    arrows_assets: Res<ArrowsAssets>,
    mut commands: Commands,
) {
    commands.spawn((
        particle_bundle(
            trigger.translation,
            std::mem::take(&mut trigger.particle),
            trigger.spawn_with_invincible,
            &particle_config,
            meshes.as_mut(),
            materials.as_mut(),
            &arrows_config,
            &arrows_assets,
        ),
        // The subparticle will have the same parent as the particle if it has a parent.
        Maybe(trigger.parent.map(ChildOf)),
    ));
}

fn invincibility_added(
    mut query: Query<
        (Entity, &mut MeshMaterial2d<ColorMaterial>),
        (With<Particle>, Added<Invincible>),
    >,
    particle_assets: Res<ParticleAssets>,
    mut commands: Commands,
) {
    for (entity, mut material) in query.iter_mut() {
        // Move particle back to collision group 2 so that it collides with the player.
        commands
            .entity(entity)
            .remove::<CollisionGroups>()
            .insert(CollisionGroups::new(
                Group::GROUP_2,
                Group::GROUP_1 | Group::GROUP_2,
            ));
        material.0 = particle_assets.invincible_material.clone();
    }
}

fn invincibility_removed(
    mut events: EventReader<InvincibleRemoved>,
    mut query: Query<(Entity, &mut MeshMaterial2d<ColorMaterial>, &Particle)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    for event in events.read() {
        let entity = event.0;
        let (entity, mut material, particle) = query.get_mut(entity).unwrap();

        // TODO crate a cache for these
        material.0 = materials.add(particle.color);

        commands
            .entity(entity)
            .remove::<CollisionGroups>()
            .insert(CollisionGroups::new(Group::GROUP_3, Group::GROUP_1));
    }
}
