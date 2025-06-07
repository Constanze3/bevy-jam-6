use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues},
};
use bevy_rapier2d::prelude::Collider;
use serde::{Deserialize, Serialize};

use super::particle::Particle;

#[derive(Serialize, Deserialize)]
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
                values.into_iter().map(|v| Vec3::from(*v)).collect()
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

#[derive(Serialize, Deserialize)]
pub struct ParticleData {
    pub spawn_position: Vec2,
    pub particle: Particle,
}

impl ParticleData {
    fn new(spawn_position: Vec2, particle: Particle) -> Self {
        Self {
            spawn_position,
            particle,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ObstacleData {
    pub transform: Transform,
    pub flat_color_mesh: FlatColorMesh,
    pub collider: Collider,
}

impl ObstacleData {
    pub fn rectangle(transform: Transform, color: Color, width: f32, height: f32) -> Self {
        Self {
            transform,
            flat_color_mesh: FlatColorMesh::new(color, Rectangle::new(width, height)),
            collider: Collider::cuboid(width / 2.0, height / 2.0),
        }
    }
}

#[derive(Serialize, Deserialize)]
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
            particles: vec![ParticleData::new(
                vec2(-100.0, 0.0),
                Particle {
                    subparticles: vec![
                        Box::new(Particle {
                            initial_velocity: vec2(0.0, -200.0),
                            ..default()
                        }),
                        Box::new(Particle {
                            initial_velocity: vec2(0.0, 200.0),
                            ..default()
                        }),
                    ],
                    ..default()
                },
            )],
            obstacles: vec![ObstacleData::rectangle(
                Transform::from_translation(vec3(100.0, 0.0, 0.0)),
                Color::linear_rgb(1.0, 1.0, 1.0),
                50.0,
                50.0,
            )],
            player_spawn: vec2(0.0, 0.0),
        }
    }
}

// impl FromWorld for LevelData {
//     fn from_world(world: &mut World) -> Self {
//         let assets = world.resource::<AssetServer>();
//
//         Self {}
//     }
// }
