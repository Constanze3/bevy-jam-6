use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiContexts, egui::{self, debug_text::print}};

use crate::{demo::particle::{self, particle_bundle, Particle}, Pause};

use super::{level::obstacle, particle::ParticleAssets, player::player};
// Removed atom::atom_seed import because the atom module does not exist or is not accessible.

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum EditorState {
    #[default]
    Disabled,
    Enabled,
}

#[derive(Resource, Default)]
pub struct EditorSettings {
    selected_tool: EditorTool,
    atom_radius: f32,
    obstacle_size: f32,
}

#[derive(Debug, Default, PartialEq)]
enum EditorTool {
    #[default]
    Select,
    PlaceAtom,
    PlaceObstacle,
    PlacePlayer,
}

pub(super) fn plugin(app: &mut App) {
    app.init_state::<EditorState>()
        .init_resource::<EditorSettings>()
        .add_systems(Update, (
            toggle_editor,
            editor_ui.run_if(in_state(EditorState::Enabled)),
            handle_editor_input.run_if(in_state(EditorState::Enabled)),
        ));
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
                print!("Enabling editor mode\n"); // Debug print
                editor_state.set(EditorState::Enabled);
                pause_state.set(Pause(true)); // Pause the game when editor is enabled
            }
            EditorState::Enabled => {
                print!("Disabling editor mode\n"); // Debug print
                editor_state.set(EditorState::Disabled);
                pause_state.set(Pause(false)); // Unpause when editor is disabled
            }
        }
    }
}

fn editor_ui(
    mut contexts: EguiContexts,
    mut editor_settings: ResMut<EditorSettings>,
) {
    egui::Window::new("Level Editor")
        .default_pos([10.0, 10.0])
        .collapsible(true)
        .interactable(true)
        .movable(true)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Tools");

            ui.horizontal(|ui| {
                ui.selectable_value(&mut editor_settings.selected_tool, EditorTool::Select, "🖱 Select");
                ui.selectable_value(&mut editor_settings.selected_tool, EditorTool::PlaceAtom, "⚛ Atom");
                ui.selectable_value(&mut editor_settings.selected_tool, EditorTool::PlaceObstacle, "⬜ Obstacle");
                ui.selectable_value(&mut editor_settings.selected_tool, EditorTool::PlacePlayer, "● Player");
            });

            ui.separator();
            ui.heading("Properties");

            match editor_settings.selected_tool {
                EditorTool::PlaceAtom => {
                    ui.add(egui::Slider::new(&mut editor_settings.atom_radius, 10.0..=100.0)
                        .text("Atom Radius"));
                }
                EditorTool::PlaceObstacle => {
                    ui.add(egui::Slider::new(&mut editor_settings.obstacle_size, 10.0..=200.0)
                        .text("Obstacle Size"));
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
    if buttons.just_pressed(MouseButton::Left) && ctx.is_pointer_over_area() {

        println!("Left click detected in editor mode - not over UI"); // Debug print

        let (camera, camera_transform) = camera_q.single()
            .expect("Expected exactly one camera in the scene");
        let window = windows.single()
            .expect("Expected exactly one window in the scene");

        if let Some(cursor_pos) = window.cursor_position() {
            println!("Cursor position: {:?}", cursor_pos); // Debug print
        }

        if let Some(world_position) = window.cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
            .map(|ray| ray.origin.truncate())
        {
            println!("World position: {:?}", world_position); // Debug print
            println!("Editor tool selected: {:?}", editor_settings.selected_tool); // Debug print
            match editor_settings.selected_tool {
                EditorTool::PlaceAtom => {
                    println!("Spawning atom at: {:?}", world_position);
                    let particle_mesh = meshes.add(Circle::new(editor_settings.atom_radius));
    let particle_material = materials.add(Color::Srgba(Srgba::hex("0f95e2").unwrap()));
// Debug print
                    commands.spawn(particle_bundle(
                vec2(-100.0, 0.0),
                Particle {
                    radius: editor_settings.atom_radius,
                    initial_velocity: Vec2::ZERO,
                    sub_particles: vec![
                        Particle {
                            radius: editor_settings.atom_radius,
                            initial_velocity: vec2(0.0, -200.0),
                            sub_particles: vec![],
                            mesh: particle_mesh.clone(),
                            material: particle_material.clone()
                        },
                        Particle {
                            radius: editor_settings.atom_radius,
                            initial_velocity: vec2(0.0, 200.0),
                            sub_particles: vec![],
                            mesh: particle_mesh.clone(),
                            material: particle_material.clone()
                        }
                    ],
                    mesh: particle_mesh.clone(),
                    material: particle_material.clone()
                },
                particle_assets.as_ref()
            ));
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
                    commands.spawn(player(
                        20.0,
                        7000.0,
                        &mut meshes,
                        &mut materials,
                    ));
                }
                EditorTool::Select => {
                    // TODO: Implement selection
                }
                // ... rest of the match cases ...
            }
        }
    }
}
