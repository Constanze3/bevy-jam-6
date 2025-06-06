use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::{EguiContextPass, EguiContexts},
    egui,
};

use crate::{
    Pause,
    demo::particle::{Particle, particle_bundle},
};

use super::{
    level::obstacle,
    particle::{ParticleAssets, ParticleKind},
    player::player,
};
// Removed atom::atom_seed import because the atom module does not exist or is not accessible.

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum EditorState {
    #[default]
    Disabled,
    Enabled,
}

#[derive(Resource)]
pub struct EditorSettings {
    selected_tool: EditorTool,
    atom_radius: f32,
    atom_color: [f32; 3],
    atom_velocity: f32,
    obstacle_size: f32,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            selected_tool: EditorTool::default(),
            atom_radius: 30.0,
            atom_color: [0.2, 0.7, 1.0],
            atom_velocity: 200.0,
            obstacle_size: 50.0,
        }
    }
}

#[derive(Debug, Default, PartialEq)]
pub enum EditorTool {
    #[default]
    Select,
    PlaceAtom,
    PlaceObstacle,
    PlacePlayer,
}

#[derive(Resource, Default)]
pub struct PlacementState {
    pub preview_entity: Option<Entity>,
    pub placing: Option<EditorTool>,
}

pub(super) fn plugin(app: &mut App) {
    app.init_state::<EditorState>()
        .init_resource::<EditorSettings>()
        .init_resource::<PlacementState>()
        .add_systems(
            EguiContextPass,
            (
                toggle_editor,
                editor_ui.run_if(in_state(EditorState::Enabled)),
                update_placement_preview.run_if(in_state(EditorState::Enabled)),
            ),
        );

    app.add_systems(
        Update,
        handle_editor_input.run_if(in_state(EditorState::Enabled)),
    );
}

fn update_placement_preview(
    mut commands: Commands,
    mut placement_state: ResMut<PlacementState>,
    editor_settings: Res<EditorSettings>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    particle_assets: Res<ParticleAssets>,
) {
    if let Some(tool) = placement_state.placing.as_ref() {
        let (camera, camera_transform) = camera_q
            .single()
            .expect("Expected exactly one camera in the scene");
        let window = windows
            .single()
            .expect("Expected exactly one window in the scene");

        if let Some(cursor_pos) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
            .map(|ray| ray.origin.truncate())
        {
            // Spawn or move the preview entity
            let entity = if let Some(entity) = placement_state.preview_entity {
                // Move existing preview
                commands
                    .entity(entity)
                    .insert(Transform::from_translation(cursor_pos.extend(10.0)));
                entity
            } else {
                let particle_mesh = meshes.add(Circle::new(editor_settings.atom_radius));
                let color = Color::srgb(
                    editor_settings.atom_color[0],
                    editor_settings.atom_color[1],
                    editor_settings.atom_color[2],
                );
                let particle_material = materials.add(color);

                // Spawn new preview
                let entity = match tool {
                    EditorTool::PlaceAtom => {
                        let bundle = particle_bundle(
                            vec2(-100.0, 0.0),
                            false,
                            Particle {
                                kind: ParticleKind::Normal,
                                radius: editor_settings.atom_radius,
                                initial_velocity: Vec2::ZERO,
                                subparticles: vec![
                                    Particle {
                                        kind: ParticleKind::Normal,
                                        radius: editor_settings.atom_radius,
                                        initial_velocity: vec2(0.0, -200.0),
                                        subparticles: vec![],
                                        mesh: particle_mesh.clone(),
                                        material: particle_material.clone(),
                                    },
                                    Particle {
                                        kind: ParticleKind::Normal,
                                        radius: editor_settings.atom_radius,
                                        initial_velocity: vec2(0.0, 200.0),
                                        subparticles: vec![],
                                        mesh: particle_mesh.clone(),
                                        material: particle_material.clone(),
                                    },
                                ],
                                mesh: particle_mesh.clone(),
                                material: particle_material.clone(),
                            },
                            particle_assets.as_ref(),
                        );
                        commands.spawn(bundle).id()
                    }
                    EditorTool::PlaceObstacle => {
                        let bundle = obstacle(
                            cursor_pos,
                            editor_settings.obstacle_size,
                            &mut meshes,
                            &mut materials,
                        );
                        commands.spawn((bundle, Name::new("Preview"))).id()
                    }
                    EditorTool::PlacePlayer => {
                        let bundle = player(20.0, 7000.0, &mut meshes, &mut materials);
                        commands.spawn((bundle, Name::new("Preview"))).id()
                    }
                    _ => return,
                };
                placement_state.preview_entity = Some(entity);
                entity
            };
            // Make sure it's visible and at the right position
            commands
                .entity(entity)
                .insert(Transform::from_translation(cursor_pos.extend(10.0)));
        }
    } else if let Some(entity) = placement_state.preview_entity.take() {
        // Remove preview if not placing
        commands.entity(entity).despawn();
    }
}

fn toggle_editor(
    input: Res<ButtonInput<KeyCode>>,
    mut editor_state: ResMut<NextState<EditorState>>,
    mut pause_state: ResMut<NextState<Pause>>,
    state: Res<State<EditorState>>,
) {
    if input.just_pressed(KeyCode::Tab) {
        match state.get() {
            EditorState::Disabled => {
                println!("Enabling editor mode"); // Debug print
                editor_state.set(EditorState::Enabled);
                pause_state.set(Pause(true)); // Pause the game when editor is enabled
            }
            EditorState::Enabled => {
                println!("Disabling editor mode"); // Debug print
                editor_state.set(EditorState::Disabled);
                pause_state.set(Pause(false)); // Unpause when editor is disabled
            }
        }
    }
}

fn editor_ui(mut contexts: EguiContexts, mut editor_settings: ResMut<EditorSettings>) {
    egui::Window::new("Level Editor")
        .default_pos([10.0, 10.0])
        .collapsible(true)
        .interactable(true)
        .movable(true)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Tools");

            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut editor_settings.selected_tool,
                    EditorTool::Select,
                    "🖱 Select",
                );
                ui.selectable_value(
                    &mut editor_settings.selected_tool,
                    EditorTool::PlaceAtom,
                    "⚛  Atom",
                );
                ui.selectable_value(
                    &mut editor_settings.selected_tool,
                    EditorTool::PlaceObstacle,
                    "⬜ Obstacle",
                );
                ui.selectable_value(
                    &mut editor_settings.selected_tool,
                    EditorTool::PlacePlayer,
                    "● Player",
                );
            });

            ui.separator();
            ui.heading("Properties");

            match editor_settings.selected_tool {
                EditorTool::PlaceAtom => {
                    ui.add(
                        egui::Slider::new(&mut editor_settings.atom_radius, 10.0..=100.0)
                            .text("Atom Radius"),
                    );
                    ui.color_edit_button_rgb(&mut editor_settings.atom_color);
                    ui.add(
                        egui::Slider::new(&mut editor_settings.atom_velocity, 0.0..=500.0)
                            .text("Sub-Particle Velocity"),
                    );
                }
                EditorTool::PlaceObstacle => {
                    ui.add(
                        egui::Slider::new(&mut editor_settings.obstacle_size, 10.0..=200.0)
                            .text("Obstacle Size"),
                    );
                }
                _ => {}
            }

            ui.separator();
            ui.heading("Level");

            if ui.button("Save Level").clicked() {
                // TODO: Implement save
            }
            if ui.button("Load Level").clicked() {
                // TODO: Implement load
            }
            if ui.button("New Level").clicked() {
                // TODO: Implement clear
            }
        });
}

fn handle_editor_input(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    editor_settings: Res<EditorSettings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut contexts: EguiContexts,
    particle_assets: Res<ParticleAssets>,
) {
    let ctx = contexts.ctx_mut();
    if buttons.just_pressed(MouseButton::Left) && !ctx.is_pointer_over_area() {
        println!("Left click detected in editor mode - not over UI"); // Debug print

        let (camera, camera_transform) = camera_q
            .single()
            .expect("Expected exactly one camera in the scene");
        let window = windows
            .single()
            .expect("Expected exactly one window in the scene");

        if let Some(cursor_pos) = window.cursor_position() {
            println!("Cursor position: {:?}", cursor_pos); // Debug print
        }

        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
            .map(|ray| ray.origin.truncate())
        {
            println!("World position: {:?}", world_position); // Debug print
            println!("Editor tool selected: {:?}", editor_settings.selected_tool); // Debug print
            match editor_settings.selected_tool {
                EditorTool::PlaceAtom => {
                    println!("Spawning atom at: {:?}", world_position); // Debug print
                    let particle_mesh = meshes.add(Circle::new(editor_settings.atom_radius));
                    let color = Color::srgb(
                        editor_settings.atom_color[0],
                        editor_settings.atom_color[1],
                        editor_settings.atom_color[2],
                    );
                    let particle_material = materials.add(color);

                    let bundle = particle_bundle(
                        vec2(-100.0, 0.0),
                        false,
                        Particle {
                            kind: ParticleKind::Normal,
                            radius: editor_settings.atom_radius,
                            initial_velocity: Vec2::ZERO,
                            subparticles: vec![
                                Particle {
                                    kind: ParticleKind::Normal,
                                    radius: editor_settings.atom_radius,
                                    initial_velocity: vec2(0.0, -200.0),
                                    subparticles: vec![],
                                    mesh: particle_mesh.clone(),
                                    material: particle_material.clone(),
                                },
                                Particle {
                                    kind: ParticleKind::Normal,
                                    radius: editor_settings.atom_radius,
                                    initial_velocity: vec2(0.0, 200.0),
                                    subparticles: vec![],
                                    mesh: particle_mesh.clone(),
                                    material: particle_material.clone(),
                                },
                            ],
                            mesh: particle_mesh.clone(),
                            material: particle_material.clone(),
                        },
                        particle_assets.as_ref(),
                    );
                    commands.spawn(bundle);
                }
                EditorTool::PlaceObstacle => {
                    commands.spawn(obstacle(
                        world_position,
                        editor_settings.obstacle_size,
                        &mut meshes,
                        &mut materials,
                    ));
                }
                EditorTool::PlacePlayer => {
                    commands.spawn(player(20.0, 7000.0, &mut meshes, &mut materials));
                }
                EditorTool::Select => {
                    // TODO: Implement selection
                } // ... rest of the match cases ...
            }
        }
    }
}
