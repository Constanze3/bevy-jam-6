use bevy::prelude::*;
use bevy::render::mesh::Mesh;

use crate::AppSystems;

use super::input::InputController;

#[derive(Component)]
struct DragIndicator;

fn update_drag_indicator(
    input_controller: Res<InputController>,
    player_query: Query<(&Transform, &crate::demo::player::Player), ()>,
    mut indicator_query: Query<Entity, With<DragIndicator>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Only show the indicator if the player is dragging.
    if let (Some(vector), Ok((player_transform, player))) = (&input_controller.vector, player_query.single()) {
        let start = player_transform.translation.truncate();
        let end = start + *vector;

        // Calculate the midpoint and rotation for the rectangle
        let direction = end - start;
        let length = direction.length();
        if length > 0.0 {
            let direction_normalized = direction / length;
            let indicator_start = start + direction_normalized * player.radius;
            let indicator_length = (length - player.radius).max(0.0);
            let angle = direction.y.atan2(direction.x);

            // Remove old indicator if it exists
            for entity in indicator_query.iter_mut() {
                commands.entity(entity).despawn();
            }

            // Create a thin rectangle mesh for the line
            let thickness = 6.0;
            let mesh = meshes.add(Rectangle::new(indicator_length, thickness));
            let material = materials.add(Color::hsl(0.0, 0.0, 0.95));
            let offset = Vec2::new(indicator_length / 2.0, 0.0).rotate(Vec2::from_angle(angle));
            // Spawn the indicator
            commands.spawn((
                Name::new("Drag Indicator"),
                Mesh2d(mesh),
                MeshMaterial2d(material),
                Transform {
                    translation: indicator_start.extend(1.0) + offset.extend(0.0), // z=1.0 to draw above player
                    rotation: Quat::from_rotation_z(angle),
                    ..Default::default()
                },
                DragIndicator,
            ));
        }
    } else {
        // Remove the indicator if not dragging
        for entity in indicator_query.iter_mut() {
            commands.entity(entity).despawn();
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, update_drag_indicator.in_set(AppSystems::Update));
}
