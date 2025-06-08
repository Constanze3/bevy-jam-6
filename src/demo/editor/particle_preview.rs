use bevy::prelude::*;

use crate::demo::particle::Particle;

pub(super) fn plugin(app: &mut App) {}

pub fn particle_preview(
    translation: Vec2,
    particle: Particle,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> impl Bundle {
    let mesh = meshes.add(Circle::new(particle.radius));
    let material = materials.add(particle.color);

    (
        Name::new("Particle"),
        Transform::from_translation(translation.extend(2.0)),
        Mesh2d(mesh),
        MeshMaterial2d(material),
        particle,
    )
}

