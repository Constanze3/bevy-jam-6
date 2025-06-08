#![allow(unused)]

use bevy::{
    asset::{AssetLoader, LoadContext, RenderAssetUsages, io::Reader},
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
};
use bevy_rapier2d::prelude::Collider;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::demo::particle::{Particle, ParticleKind};

pub(super) fn plugin(app: &mut App) {
    app.init_asset::<LevelData>();
    app.init_asset_loader::<LevelDataLoader>();
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FlatColorMesh {
    color: Color,
    positions: Vec<Vec3>,
    indices: Vec<u32>,
}

impl FlatColorMesh {
    pub fn new(color: Color, mesh: impl Into<Mesh>) -> Self {
        let mut mesh = mesh.into();

        let positions = {
            if let VertexAttributeValues::Float32x3(values) = mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .expect("The mesh should have the position attribute.")
            {
                values.iter().map(|v| Vec3::from(*v)).collect()
            } else {
                panic!("The mesh should have the Float32x3 position attribute.");
            }
        };

        let Indices::U32(indices) = mesh
            .remove_indices()
            .expect("The mesh should have indices.")
        else {
            panic!("The mesh should have u32 indices.");
        };

        Self {
            color,
            positions,
            indices,
        }
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn into_mesh(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_indices(Indices::U32(self.indices))
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ParticleData {
    pub spawn_position: Vec2,
    pub particle: Particle,
}

impl ParticleData {
    pub fn new(spawn_position: Vec2, particle: Particle) -> Self {
        Self {
            spawn_position,
            particle,
        }
    }

    pub fn default_at(translation: Vec2) -> Self {
        Self {
            spawn_position: translation,
            particle: Particle::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ObstacleData {
    pub transform: Transform,
    pub flat_color_mesh: FlatColorMesh,
    pub collider: Collider,
    pub is_killer: bool,
}

impl ObstacleData {
    pub fn rectangle(
        transform: Transform,
        color: Color,
        width: f32,
        height: f32,
        killer: bool,
    ) -> Self {
        Self {
            transform,
            flat_color_mesh: FlatColorMesh::new(color, Rectangle::new(width, height)),
            collider: Collider::cuboid(width / 2.0, height / 2.0),
            is_killer: killer,
        }
    }

    pub fn default_at(translation: Vec2) -> Self {
        let transform = Transform::from_translation(translation.extend(0.0));
        let color = Color::WHITE;
        let width = 50.0;
        let height = 50.0;
        let killer = false;

        Self::rectangle(transform, color, width, height, killer)
    }
}

#[derive(Asset, TypePath, Clone, Serialize, Deserialize, Default)]
pub struct LevelData {
    pub name: String,
    pub author: Option<String>,
    pub particles: Vec<ParticleData>,
    pub obstacles: Vec<ObstacleData>,
    pub player_spawn: Vec2,
}

impl LevelData {
    pub fn example() -> Self {
        Self {
            name: String::from("Example"),
            author: None,
            particles: vec![
                ParticleData::new(
                    vec2(-100.0, 0.0),
                    Particle {
                        subparticles: vec![
                            Particle {
                                initial_velocity: vec2(0.0, -200.0),
                                subparticles: vec![
                                    Particle {
                                        initial_velocity: vec2(200.0, 0.0),
                                        ..default()
                                    },
                                    Particle {
                                        initial_velocity: vec2(-200.0, 0.0),
                                        ..default()
                                    },
                                ],
                                ..default()
                            },
                            Particle {
                                kind: ParticleKind::Killer,
                                radius: 40.0,
                                initial_velocity: vec2(0.0, 200.0),
                                color: Color::srgb(1.0, 0.0, 0.0),
                                ..default()
                            },
                        ],
                        ..default()
                    },
                ),
                ParticleData::new(
                    vec2(-300.0, 0.0),
                    Particle {
                        subparticles: vec![
                            Particle {
                                initial_velocity: vec2(0.0, -200.0),
                                ..default()
                            },
                            Particle {
                                initial_velocity: vec2(0.0, 200.0),
                                ..default()
                            },
                        ],
                        ..default()
                    },
                ),
            ],
            obstacles: vec![ObstacleData::rectangle(
                Transform::from_translation(vec3(100.0, 0.0, 0.0)),
                Color::linear_rgb(1.0, 1.0, 1.0),
                50.0,
                50.0,
                false,
            )],
            player_spawn: vec2(0.0, 0.0),
        }
    }
}

#[derive(Default)]
struct LevelDataLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
enum LevelAssetLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for LevelDataLoader {
    type Asset = LevelData;
    type Settings = ();
    type Error = LevelAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let level_data = ron::de::from_bytes::<LevelData>(&bytes)?;

        Ok(level_data)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}
