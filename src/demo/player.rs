//! Player-specific behavior.

use bevy::prelude::*;

use crate::demo::movement::{MovementController, ScreenWrap};

use super::input::InputEvent;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();

    app.add_observer(handle_input);
}

/// The player character.
pub fn player(
    max_speed: f32,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) -> impl Bundle {
    let mesh = meshes.add(Circle::new(25.0));
    let material = materials.add(Color::hsl(0.0, 0.95, 0.7));

    (
        Name::new("Player"),
        Player,
        Mesh2d(mesh),
        MeshMaterial2d(material),
        Transform::default(),
        MovementController {
            max_speed,
            ..default()
        },
        ScreenWrap,
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
struct Player;

fn handle_input(
    trigger: Trigger<InputEvent>,
    mut controller_query: Query<&mut MovementController, With<Player>>,
) {
    // Apply movement intent to controllers.
    for mut controller in &mut controller_query {
        controller.intent = 0.05 * trigger.vector;
    }
}
