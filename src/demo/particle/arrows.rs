use bevy::{
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith},
    prelude::*,
};
use serde::{Deserialize, Serialize};

use crate::{AppSystems, PausableSystems, asset_tracking::LoadResource};

use super::Particle;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Arrows>();

    app.init_resource::<ArrowsConfig>();
    app.load_resource::<ArrowsAssets>();

    app.add_systems(
        Update,
        move_arrows
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

#[derive(Resource, Serialize, Deserialize, Reflect, Clone, Copy)]
#[reflect(Resource)]
pub struct ArrowsConfig {
    arrow_offset: f32,
    arrow_scale: f32,
    local_z: f32,
}

impl Default for ArrowsConfig {
    fn default() -> Self {
        Self {
            arrow_offset: 3.0,
            arrow_scale: 0.02,
            local_z: -3.0,
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct ArrowsAssets {
    #[dependency]
    arrow_image: Handle<Image>,
}

impl FromWorld for ArrowsAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        Self {
            arrow_image: assets.load("images/arrow.png"),
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[relationship_target(relationship = ArrowsOf, linked_spawn)]
pub struct Arrows(Entity);

#[derive(Component)]
#[relationship(relationship_target = Arrows)]
pub struct ArrowsOf(Entity);

pub fn arrows(
    translation: Vec2,
    particle: &Particle,
    arrows_config: &ArrowsConfig,
    arrows_assets: &ArrowsAssets,
) -> impl Bundle {
    let arrow_image = arrows_assets.arrow_image.clone();

    let particle = particle.clone();
    let arrows_config = arrows_config.clone();
    let arrow_spawn_list = move |parent: &mut RelatedSpawner<ChildOf>| {
        for sub_particle in particle.subparticles.iter() {
            let direction = sub_particle.initial_velocity.normalize();
            let angle = direction.y.atan2(direction.x);

            let offset = particle.radius + arrows_config.arrow_offset;
            let position = Vec2::ZERO + direction * offset;

            parent.spawn((
                Name::new("Arrow"),
                Sprite::from_image(arrow_image.clone()),
                Transform {
                    translation: position.extend(0.0),
                    rotation: Quat::from_rotation_z(angle),
                    scale: Vec3::ONE * arrows_config.arrow_scale,
                },
            ));
        }
    };

    (
        Name::new("Arrows"),
        Transform::from_translation(translation.extend(arrows_config.local_z)),
        Visibility::default(),
        Children::spawn(SpawnWith(arrow_spawn_list)),
    )
}

fn move_arrows(
    query: Query<(&Transform, &Arrows)>,
    mut arrows_query: Query<&mut Transform, (With<ArrowsOf>, Without<Arrows>)>,
    arrows_config: Res<ArrowsConfig>,
) {
    for (transform, arrows) in query.iter() {
        let mut arrows_transform = arrows_query.get_mut(arrows.0).unwrap();
        arrows_transform.translation = transform.translation.xy().extend(arrows_config.local_z);
    }
}
