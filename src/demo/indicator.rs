use bevy::prelude::*;
use bevy::render::mesh::Mesh;
use bevy_rapier2d::plugin::PhysicsSet;

use crate::{PausableSystems, Pause};

use super::input::InputController;
use super::player::Player;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        update_drag_indicator
            .in_set(PausableSystems)
            .after(PhysicsSet::Writeback),
    );

    app.add_systems(OnEnter(Pause(true)), hide_drag_indicator);
}

pub fn drag_indicator(
    thickness: f32,
    length_scalar: f32,
    color: Color,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> impl Bundle {
    let mesh = meshes.reserve_handle();
    let material = materials.add(color);

    (
        Name::new("Drag Indicator"),
        Mesh2d(mesh.clone()),
        MeshMaterial2d(material),
        DragIndicator {
            mesh,
            thickness,
            length_scalar,
        },
    )
}

#[derive(Component)]
pub struct DragIndicator {
    pub mesh: Handle<Mesh>,
    pub thickness: f32,
    pub length_scalar: f32,
}

fn update_drag_indicator(
    input_controller: Res<InputController>,
    player_query: Query<(&Player, &Transform)>,
    mut indicator_query: Query<
        (&mut DragIndicator, &mut Transform, &mut Visibility),
        Without<Player>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let success = 'blk: {
        let Some(vector) = &input_controller.vector else {
            break 'blk false;
        };

        if player_query.is_empty() {
            break 'blk false;
        }

        let (player, player_transform) = player_query.single().unwrap();

        if !player.can_move {
            break 'blk false;
        }

        for (indicator, mut indicator_transform, mut indicator_visibility) in
            indicator_query.iter_mut()
        {
            let length = vector.length() * indicator.length_scalar;
            if 0.0 < length {
                let start = player_transform.translation.truncate();
                let angle = vector.y.atan2(vector.x);

                let offset = Vec2::new(length / 2.0, 0.0).rotate(Vec2::from_angle(angle));

                // Create a thin rectangle mesh for the line.
                let new_mesh: Mesh = Rectangle::new(length, indicator.thickness).into();
                let mesh_id = indicator.mesh.id();

                if let Some(mesh) = meshes.get_mut(indicator.mesh.id()) {
                    *mesh = new_mesh;
                } else {
                    meshes.insert(mesh_id, new_mesh);
                }

                // Adjust its position and rotation.
                // z = -5.0 to draw behind the player.
                indicator_transform.translation = (start + offset).extend(-5.0);
                indicator_transform.rotation = Quat::from_rotation_z(angle);

                *indicator_visibility = Visibility::Inherited;
            }
        }

        true
    };

    if !success {
        hide_drag_indicator(indicator_query.transmute_lens_filtered().query());
    }
}

fn hide_drag_indicator(mut indicator_query: Query<&mut Visibility, With<DragIndicator>>) {
    for mut indicator_visibility in indicator_query.iter_mut() {
        *indicator_visibility = Visibility::Hidden;
    }
}
