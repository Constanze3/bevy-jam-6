use bevy::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::{EguiContextPass, EguiContexts},
    egui,
};

use crate::{
    camera::{GameplayCamera, Letterboxing, Size, letterbox},
    screens::Screen,
};

use super::{
    level::{
        SpawnRawLevel,
        level_data::{LevelData, ObstacleData, ParticleData},
    },
    particle::SpawnParticle,
    player::{PlayerConfig, player},
};

mod particle_preview;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<EditorState>();
    app.add_event::<EditorEvent>();

    app.add_observer(spawn_level_preview);
    app.add_systems(
        OnEnter(Screen::Editor),
        |mut commands: Commands, mut editor_state: ResMut<EditorState>| {
            commands.trigger(SpawnLevelPreview);
            editor_state.editing = true;
        },
    );
    app.add_systems(EguiContextPass, editor_ui.run_if(in_state(Screen::Editor)));

    app.add_systems(
        Update,
        (
            handle_editor_event_exit,
            handle_editor_event_save,
            handle_editor_event_clear,
            handle_editor_event_play,
        )
            .run_if(in_state(Screen::Editor)),
    );

    app.add_systems(Update, (object_placement).run_if(in_state(Screen::Editor)));
}

#[derive(Component)]
pub struct LevelPreview;

#[derive(Event)]
pub struct SpawnLevelPreview;

// basically spawn level but with preview objects
pub fn spawn_level_preview(
    _: Trigger<SpawnLevelPreview>,
    level_preview_query: Query<Entity, With<LevelPreview>>,
    editor_state: Res<EditorState>,
    player_config: Res<PlayerConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    // Delete previous level preview.
    for previous_level_preview in level_preview_query.iter() {
        commands.entity(previous_level_preview).despawn();
    }

    let level_preview = commands
        .spawn((
            Name::new("Level Preview"),
            LevelPreview,
            Transform::default(),
            Visibility::default(),
            StateScoped(Screen::Editor),
            children![player(
                editor_state.level.player_spawn,
                &mut meshes,
                &mut materials,
                &player_config
            )],
        ))
        .id();

    for obstacle_data in editor_state.level.obstacles.iter() {
        let obstacle_data = obstacle_data.clone();

        let material = materials.add(obstacle_data.flat_color_mesh.color());
        let mesh = meshes.add(obstacle_data.flat_color_mesh.into_mesh());

        let obstacle = commands
            .spawn(obstacle_preview(obstacle_data.transform, material, mesh))
            .id();

        commands.entity(level_preview).add_child(obstacle);
    }

    for particle_data in editor_state.level.particles.iter() {
        commands.trigger(SpawnParticle {
            translation: particle_data.spawn_position,
            particle: particle_data.particle.clone(),
            spawn_with_invincible: false,
            parent: Some(level_preview),
        });
    }
}

// specialized preview for a subparticle
pub fn subparticle_preview() {}

pub fn obstacle_preview(
    transform: Transform,
    material: Handle<ColorMaterial>,
    mesh: Handle<Mesh>,
) -> impl Bundle {
    (
        Name::new("Obstacle"),
        transform,
        Mesh2d(mesh),
        MeshMaterial2d(material),
    )
}

#[derive(Default, PartialEq, Eq)]
enum EditorMode {
    #[default]
    Place,
    Select,
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
enum Object {
    #[default]
    Particle,
    Obstacle,
}

#[derive(Component)]
enum PreviewIndex {
    Particle(Vec<usize>),
    Obstacle(usize),
}

#[derive(Resource, Default)]
pub struct EditorState {
    level: LevelData,
    mode: EditorMode,
    placement: Object,
    selected: Option<PreviewIndex>,
    pub editing: bool,
}

// particle selected -> show translation, Particle
// obstacle selected -> show width, height, translation, rotation
// player selected -> show vec2

#[derive(Event, PartialEq, Eq)]
enum EditorEvent {
    Exit,
    Save,
    Load,
    Clear,
    Play,
}

fn handle_editor_event_exit(
    mut events: EventReader<EditorEvent>,
    mut editor_state: ResMut<EditorState>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    for event in events.read() {
        if *event == EditorEvent::Exit {
            next_screen.set(Screen::Title);
            editor_state.editing = false;
        }
    }
}

fn handle_editor_event_save(mut events: EventReader<EditorEvent>, editor_state: Res<EditorState>) {
    for event in events.read() {
        if *event == EditorEvent::Save {
            println!(
                "{}",
                ron::ser::to_string_pretty(&editor_state.level, ron::ser::PrettyConfig::default())
                    .unwrap(),
            );
        }
    }
}

fn handle_editor_event_clear(
    mut events: EventReader<EditorEvent>,
    mut editor_state: ResMut<EditorState>,
    mut commands: Commands,
) {
    for event in events.read() {
        if *event == EditorEvent::Clear {
            editor_state.level = LevelData::default();
            commands.trigger(SpawnLevelPreview);
        }
    }
}

fn handle_editor_event_play(
    mut events: EventReader<EditorEvent>,
    editor_state: Res<EditorState>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut commands: Commands,
) {
    for event in events.read() {
        if *event == EditorEvent::Play {
            commands.trigger(SpawnRawLevel {
                data: editor_state.level.clone(),
                level: None,
            });
            next_screen.set(Screen::Gameplay);
        }

        break;
    }
}

fn editor_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<EditorState>,
    mut events: EventWriter<EditorEvent>,
) {
    egui::Window::new("Editor")
        .default_pos([10.0, 10.0])
        .collapsible(true)
        .interactable(true)
        .movable(true)
        .show(contexts.ctx_mut(), |ui| {
            let style = ui.style_mut();
            style
                .text_styles
                .get_mut(&egui::TextStyle::Heading)
                .unwrap()
                .size = 24.0;

            style
                .text_styles
                .get_mut(&egui::TextStyle::Body)
                .unwrap()
                .size = 20.0;
            style
                .text_styles
                .get_mut(&egui::TextStyle::Button)
                .unwrap()
                .size = 20.0;

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    events.write(EditorEvent::Save);
                }

                if ui.button("Load").clicked() {
                    events.write(EditorEvent::Load);
                }

                if ui.button("Clear").clicked() {
                    events.write(EditorEvent::Clear);
                }
            });

            ui.separator();

            egui::Grid::new("name_author_grid")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Name:");
                    ui.add(egui::TextEdit::singleline(&mut state.level.name).desired_width(150.0));
                    ui.end_row();

                    let author = &mut String::new();
                    if let Some(a) = state.level.author.clone() {
                        *author = a;
                    }

                    ui.label("Author:");
                    ui.add(
                        egui::TextEdit::singleline(author)
                            .hint_text("None")
                            .desired_width(150.0),
                    );

                    state.level.author = (author != "").then_some(author.clone());
                });

            ui.separator();

            ui.horizontal(|ui| {
                ui.selectable_value(&mut state.mode, EditorMode::Place, "Place");
                ui.selectable_value(&mut state.mode, EditorMode::Select, "Select");
            });

            match state.mode {
                EditorMode::Place => {
                    state.selected = None;

                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut state.placement, Object::Particle, "⚛  Particle");
                        ui.selectable_value(&mut state.placement, Object::Obstacle, "📦 Obstacle");
                    });
                }
                EditorMode::Select => {
                    if state.selected.is_none() {
                        ui.label("Nothing selected");
                    }
                }
            }

            ui.separator();

            if ui.button("Play").clicked() {
                events.write(EditorEvent::Play);
            }

            ui.separator();

            if ui.button("Quit to title").clicked() {
                events.write(EditorEvent::Exit);
            }
        });
}

fn mouse_world_position(
    window_query: &Query<&Window>,
    camera_query: &Query<(&Camera, &GlobalTransform), With<GameplayCamera>>,
    letterboxing: &Letterboxing,
) -> Option<Vec2> {
    let window = window_query.single().unwrap();
    let (camera, camera_transform) = camera_query.single().unwrap();

    let window_size = Size::new(window.width(), window.height());
    let actual_size = letterbox(window_size, letterboxing.aspect_ratio);

    let horizontal_band = (window_size.width - actual_size.width) / 2.0;
    let vertical_band = (window_size.height - actual_size.height) / 2.0;

    let Some(pos) = window.cursor_position() else {
        return None;
    };

    if pos.x < horizontal_band || horizontal_band + actual_size.width < pos.x {
        return None;
    }

    if pos.y < vertical_band || vertical_band + actual_size.height < pos.y {
        return None;
    }

    let actual_pos = vec2(pos.x - horizontal_band, pos.y - vertical_band);

    let mut normalized = actual_pos / vec2(actual_size.width, actual_size.height);
    normalized.y = 1.0 - normalized.y;

    let ndc = 2.0 * normalized - Vec2::ONE;

    let world_pos = camera
        .ndc_to_world(camera_transform, ndc.extend(1.0))
        .map(|p| p.xy());

    world_pos
}

fn object_placement(
    mut editor_state: ResMut<EditorState>,
    mut contexts: EguiContexts,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GameplayCamera>>,
    letterboxing: Res<Letterboxing>,
    mut commands: Commands,
) {
    if editor_state.mode != EditorMode::Place {
        return;
    }

    if !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }

    // Ignore clicks over editor.
    let ctx = contexts.ctx_mut();
    if ctx.is_pointer_over_area() {
        return;
    }

    let Some(position) = mouse_world_position(&window_query, &camera_query, &letterboxing) else {
        return;
    };

    match editor_state.placement {
        Object::Particle => {
            editor_state
                .level
                .particles
                .push(ParticleData::default_at(position));
        }
        Object::Obstacle => {
            editor_state
                .level
                .obstacles
                .push(ObstacleData::default_at(position));
        }
    }

    commands.trigger(SpawnLevelPreview);
}
