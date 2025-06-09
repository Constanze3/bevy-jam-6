use bevy::{
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith},
    input::ButtonState,
    math::FloatOrd,
    picking::pointer::{Location, PointerAction, PointerId, PointerInput},
    prelude::*,
    render::camera::NormalizedRenderTarget,
    window::WindowEvent,
};
use bevy_inspector_egui::{
    bevy_egui::{EguiContextPass, EguiContexts},
    egui::{self, InnerResponse, Ui},
};
use bevy_mod_picking::pointer::Uuid;

use crate::{
    camera::{GameplayCamera, GameplayRenderTarget, Letterboxing, Size, letterbox},
    demo::{
        level::{
            SpawnRawLevel,
            level_data::{LevelData, ObstacleData, ParticleData},
        },
        player::{PlayerConfig, player},
    },
    external::maybe::Maybe,
    screens::Screen,
};

use super::particle::{
    Particle, ParticleConfig, ParticleKind,
    arrows::{Arrows, ArrowsAssets, ArrowsConfig, ArrowsOf, arrows},
};

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
    app.add_systems(
        EguiContextPass,
        (editor_ui, refresh_level_preview)
            .chain()
            .run_if(in_state(Screen::Editor)),
    );

    app.add_systems(
        Update,
        (
            handle_editor_event_exit,
            handle_editor_event_print,
            handle_editor_event_clear,
            handle_editor_event_play,
        )
            .run_if(in_state(Screen::Editor)),
    );

    app.add_systems(Update, (object_placement).run_if(in_state(Screen::Editor)));

    app.add_systems(OnEnter(Screen::Editor), spawn_editor_pointer);
    app.add_systems(
        PreUpdate,
        editor_pointer_picking.run_if(in_state(Screen::Editor)),
    );

    app.add_observer(spawn_particle_preview);
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
        ))
        .id();

    let player = commands
        .spawn((
            player(
                editor_state.level.player_spawn,
                &mut meshes,
                &mut materials,
                &player_config,
            ),
            PreviewIndex::Player,
        ))
        .observe(select)
        .id();

    commands.entity(level_preview).add_child(player);

    for (i, obstacle_data) in editor_state.level.obstacles.iter().enumerate() {
        let obstacle_data = *obstacle_data;

        let material = materials.add(obstacle_data.color);
        let mesh = meshes.add(Rectangle::new(obstacle_data.width, obstacle_data.height));

        let obstacle = commands
            .spawn((
                obstacle_preview(obstacle_data.transform, material, mesh),
                PreviewIndex::Obstacle(i),
            ))
            .observe(select)
            .id();

        commands.entity(level_preview).add_child(obstacle);
    }

    for (i, particle_data) in editor_state.level.particles.iter().enumerate() {
        commands.trigger(SpawnParticlePreview {
            index: i,
            translation: particle_data.spawn_position,
            particle: particle_data.particle.clone(),
            parent: Some(level_preview),
        });
    }
}

#[derive(Event)]
pub struct SpawnParticlePreview {
    pub index: usize,
    pub translation: Vec2,
    pub particle: Particle,
    pub parent: Option<Entity>,
}

fn spawn_particle_preview(
    mut trigger: Trigger<SpawnParticlePreview>,
    particle_config: Res<ParticleConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    arrows_config: Res<ArrowsConfig>,
    arrows_assets: Res<ArrowsAssets>,
    mut commands: Commands,
) {
    commands
        .spawn((
            PreviewIndex::Particle(trigger.index),
            particle_preview_bundle(
                trigger.translation,
                std::mem::take(&mut trigger.particle),
                &particle_config,
                meshes.as_mut(),
                materials.as_mut(),
                &arrows_config,
                &arrows_assets,
            ),
            // The subparticle will have the same parent as the particle if it has a parent.
            Maybe(trigger.parent.map(ChildOf)),
        ))
        .observe(select);
}

pub fn particle_preview_bundle(
    translation: Vec2,
    particle: Particle,
    particle_config: &ParticleConfig,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    arrows_config: &ArrowsConfig,
    arrows_assets: &ArrowsAssets,
) -> impl Bundle {
    let spawn_list = {
        let arrows_config = *arrows_config;
        let arrows_assets = arrows_assets.clone();
        let particle = particle.clone();

        move |spawner: &mut RelatedSpawner<ArrowsOf>| {
            spawner.spawn(arrows(
                translation,
                &particle,
                &arrows_config,
                &arrows_assets,
            ));
        }
    };

    (
        Arrows::spawn(SpawnWith(spawn_list)),
        particle_preview(translation, particle, particle_config, meshes, materials),
    )
}

pub fn particle_preview(
    translation: Vec2,
    particle: Particle,
    particle_config: &ParticleConfig,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> impl Bundle {
    let mesh = meshes.add(Circle::new(particle.radius));
    let material = materials.add(particle.color);

    (
        Name::new("Particle"),
        Transform::from_translation(translation.extend(particle_config.local_z)),
        Mesh2d(mesh),
        MeshMaterial2d(material),
        particle,
    )
}

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

#[derive(Component, Clone, Copy)]
enum PreviewIndex {
    Player,
    Particle(usize),
    Obstacle(usize),
}

#[derive(Resource, Default)]
pub struct EditorState {
    pub level: LevelData,
    mode: EditorMode,
    placement: Object,
    selected: Option<PreviewIndex>,
    pub editing: bool,
}

#[derive(Event, PartialEq, Eq)]
enum EditorEvent {
    Exit,
    Print,
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

fn handle_editor_event_print(
    mut events: EventReader<EditorEvent>,
    editor_state: Res<EditorState>,
    mut contexts: EguiContexts,
) {
    for event in events.read() {
        if *event == EditorEvent::Print {
            let string: String =
                ron::ser::to_string_pretty(&editor_state.level, ron::ser::PrettyConfig::default())
                    .unwrap();

            let ctx = contexts.ctx_mut();
            ctx.copy_text(string);
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
    if let Some(event) = events.read().next() {
        if *event == EditorEvent::Play {
            commands.trigger(SpawnRawLevel {
                data: editor_state.level.clone(),
                level: None,
            });
            next_screen.set(Screen::Gameplay);
        }
    }
}

fn vec2_input_ui(ui: &mut Ui, vec2: &mut Vec2) -> InnerResponse<()> {
    ui.horizontal(|ui| {
        ui.label("x:");
        ui.add(egui::DragValue::new(&mut vec2.x));
        ui.label("y:");
        ui.add(egui::DragValue::new(&mut vec2.y));
    })
}

fn vec2_angle_magnitude_input_ui(ui: &mut Ui, vec2: &mut Vec2) {
    let mut magnitude = vec2.length();
    let mut angle = vec2.y.atan2(vec2.x).to_degrees();

    ui.horizontal(|ui| {
        ui.label("Angle:");
        ui.add(egui::DragValue::new(&mut angle));
    });

    ui.horizontal(|ui| {
        ui.label("Magnitude:");
        ui.add(egui::DragValue::new(&mut magnitude));
    });

    let angle = angle.to_radians();
    *vec2 = Vec2::new(angle.cos(), angle.sin()) * magnitude;
}

fn particle_ui(
    ui: &mut Ui,
    superparticle: bool,
    id: usize,
    particle: &mut Particle,
) -> Option<usize> {
    let mut to_delete = None;

    let name = {
        if superparticle {
            String::from("Particle")
        } else {
            format!("Particle {}", id)
        }
    };

    egui::CollapsingHeader::new(name)
        .default_open(false)
        .show(ui, |ui| {
            if ui.button("Delete").clicked() {
                to_delete = Some(id);
            }

            egui::Grid::new(format!("{}_grid", id))
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Color:");
                    let color = particle.color.to_srgba().to_u8_array();
                    let mut color = [color[0], color[1], color[2]];
                    egui::color_picker::color_edit_button_srgb(ui, &mut color);
                    particle.color = Color::srgb_u8(color[0], color[1], color[2]);
                    ui.end_row();

                    ui.label("Radius:");
                    ui.add(egui::DragValue::new(&mut particle.radius));
                    ui.end_row();

                    ui.label("Velocity:");
                    vec2_angle_magnitude_input_ui(ui, &mut particle.initial_velocity);
                    ui.end_row();

                    let mut is_killer = particle.kind == ParticleKind::Killer;
                    ui.checkbox(&mut is_killer, "Is Killer");
                    if is_killer {
                        particle.kind = ParticleKind::Killer;
                    } else {
                        particle.kind = ParticleKind::Normal;
                    }
                    ui.end_row();
                });

            ui.label("Subparticles:");
            egui::CollapsingHeader::new("Subparticles")
                .default_open(false)
                .show(ui, |ui| {
                    let mut deleted = None;
                    for (i, subparticle) in particle.subparticles.iter_mut().enumerate() {
                        deleted = particle_ui(ui, false, i, subparticle);
                    }

                    if let Some(deleted) = deleted {
                        particle.subparticles.remove(deleted);
                    }
                });

            if ui.button("Add").clicked() {
                particle.subparticles.push(Particle::default());
            }
        });

    to_delete
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
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Copy to Clipboard").clicked() {
                        events.write(EditorEvent::Print);
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
                        ui.add(
                            egui::TextEdit::singleline(&mut state.level.name).desired_width(150.0),
                        );
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

                        state.level.author = (!author.is_empty()).then_some(author.clone());
                    });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.selectable_value(&mut state.mode, EditorMode::Place, "Place");
                    ui.selectable_value(&mut state.mode, EditorMode::Select, "Select");
                });

                ui.separator();
                ui.add_space(12.0);

                match state.mode {
                    EditorMode::Place => {
                        state.selected = None;

                        ui.horizontal(|ui| {
                            ui.selectable_value(
                                &mut state.placement,
                                Object::Particle,
                                "⚛  Particle",
                            );
                            ui.selectable_value(
                                &mut state.placement,
                                Object::Obstacle,
                                "📦 Obstacle",
                            );
                        });
                    }
                    EditorMode::Select => {
                        if let Some(selected) = state.selected {
                            match selected {
                                PreviewIndex::Player => {
                                    egui::Grid::new("player_grid")
                                        .num_columns(2)
                                        .spacing([10.0, 8.0])
                                        .show(ui, |ui| {
                                            ui.label("Position:");
                                            vec2_input_ui(ui, &mut state.level.player_spawn);
                                            ui.end_row();
                                        });
                                }
                                PreviewIndex::Particle(index) => {
                                    let particle_data =
                                        state.level.particles.get_mut(index).unwrap();

                                    ui.label("Position:");
                                    vec2_input_ui(ui, &mut particle_data.spawn_position);

                                    if particle_ui(ui, true, index, &mut particle_data.particle)
                                        .is_some()
                                    {
                                        state.level.particles.remove(index);
                                        state.selected = None;
                                        return;
                                    }
                                }
                                PreviewIndex::Obstacle(index) => {
                                    if ui.button("Delete").clicked() {
                                        state.level.obstacles.remove(index);
                                        state.selected = None;
                                        return;
                                    }

                                    let obstacle = state.level.obstacles.get_mut(index).unwrap();

                                    egui::Grid::new("obstacle_grid")
                                        .num_columns(2)
                                        .spacing([10.0, 8.0])
                                        .show(ui, |ui| {
                                            ui.label("Position:");
                                            let mut position = obstacle.transform.translation.xy();
                                            vec2_input_ui(ui, &mut position);
                                            obstacle.transform.translation = position.extend(0.0);
                                            ui.end_row();

                                            ui.label("Rotation:");
                                            let mut angle = obstacle
                                                .transform
                                                .rotation
                                                .to_euler(EulerRot::XYZ)
                                                .2
                                                .to_degrees();
                                            ui.add(egui::DragValue::new(&mut angle));
                                            obstacle.transform.rotation =
                                                Quat::from_rotation_z(angle.to_radians());
                                            ui.end_row();

                                            ui.label("Color:");
                                            let color = obstacle.color.to_srgba().to_u8_array();
                                            let mut color = [color[0], color[1], color[2]];
                                            egui::color_picker::color_edit_button_srgb(
                                                ui, &mut color,
                                            );
                                            obstacle.color =
                                                Color::srgb_u8(color[0], color[1], color[2]);
                                            ui.end_row();

                                            ui.label("Width:");
                                            ui.add(egui::DragValue::new(&mut obstacle.width));
                                            ui.end_row();

                                            ui.label("Height:");
                                            ui.add(egui::DragValue::new(&mut obstacle.height));
                                            ui.end_row();

                                            ui.checkbox(&mut obstacle.is_killer, "Is Killer");
                                            ui.end_row();
                                        });
                                }
                            }
                        } else {
                            ui.label("Nothing selected");
                        }
                    }
                }

                ui.add_space(12.0);
                ui.separator();

                if ui.button("Play").clicked() {
                    events.write(EditorEvent::Play);
                }

                ui.separator();

                if ui.button("Quit to title").clicked() {
                    events.write(EditorEvent::Exit);
                }
            });
        });
}

fn refresh_level_preview(mut commands: Commands) {
    commands.trigger(SpawnLevelPreview);
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

    let pos = window.cursor_position()?;

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

    // world pos
    camera
        .ndc_to_world(camera_transform, ndc.extend(1.0))
        .map(|p| p.xy())
}

fn object_placement(
    mut editor_state: ResMut<EditorState>,
    mut contexts: EguiContexts,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<GameplayCamera>>,
    letterboxing: Res<Letterboxing>,
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
}

#[derive(Component)]
pub struct EditorPointer;

fn spawn_editor_pointer(mut commands: Commands) {
    commands.spawn((
        StateScoped(Screen::Editor),
        EditorPointer,
        PointerId::Custom(Uuid::new_v4()),
    ));
}

pub fn editor_pointer_picking(
    mut window_events: EventReader<WindowEvent>,
    pointer_query: Query<&PointerId, With<EditorPointer>>,
    mut contexts: EguiContexts,
    window_query: Query<&Window>,
    letterboxing: Res<Letterboxing>,
    gameplay_render_target: Query<&GameplayRenderTarget>,
    mut cursor_last: Local<Vec2>,
    mut pointer_events: EventWriter<PointerInput>,
) {
    let pointer_id = pointer_query.single().unwrap();
    let handle = &gameplay_render_target.single().unwrap().0;

    for window_event in window_events.read() {
        match window_event {
            WindowEvent::CursorMoved(event) => {
                let ctx = contexts.ctx_mut();
                if ctx.is_pointer_over_area() {
                    continue;
                }

                let window = window_query.single().unwrap();
                let window_size = Size::new(window.width(), window.height());
                let actual_size = letterbox(window_size, letterboxing.aspect_ratio);

                let horizontal_band = (window_size.width - actual_size.width) / 2.0;
                let vertical_band = (window_size.height - actual_size.height) / 2.0;

                let pos = event.position;

                if pos.x < horizontal_band || horizontal_band + actual_size.width < pos.x {
                    continue;
                }

                if pos.y < vertical_band || vertical_band + actual_size.height < pos.y {
                    continue;
                }

                let actual_pos = vec2(pos.x - horizontal_band, pos.y - vertical_band);

                let normalized = actual_pos / vec2(actual_size.width, actual_size.height);

                let attempt = vec2(
                    normalized.x * letterboxing.texture_size.width as f32,
                    normalized.y * letterboxing.texture_size.height as f32,
                );

                let location = Location {
                    target: NormalizedRenderTarget::Image(
                        bevy::render::camera::ImageRenderTarget {
                            handle: handle.clone(),
                            scale_factor: FloatOrd(1.0),
                        },
                    ),
                    position: attempt,
                };

                pointer_events.write(PointerInput::new(
                    *pointer_id,
                    location,
                    PointerAction::Move {
                        delta: event.position - *cursor_last,
                    },
                ));

                *cursor_last = event.position;
            }
            WindowEvent::MouseButtonInput(input) => {
                let location = Location {
                    target: NormalizedRenderTarget::Image(
                        bevy::render::camera::ImageRenderTarget {
                            handle: handle.clone(),
                            scale_factor: FloatOrd(1.0),
                        },
                    ),
                    position: *cursor_last,
                };

                let button = match input.button {
                    MouseButton::Left => PointerButton::Primary,
                    MouseButton::Right => PointerButton::Secondary,
                    MouseButton::Middle => PointerButton::Middle,
                    MouseButton::Other(_) | MouseButton::Back | MouseButton::Forward => continue,
                };

                let action = match input.state {
                    ButtonState::Pressed => PointerAction::Press(button),
                    ButtonState::Released => PointerAction::Release(button),
                };

                pointer_events.write(PointerInput::new(*pointer_id, location, action));
            }

            _ => {}
        }
    }
}

fn select(
    trigger: Trigger<Pointer<Pressed>>,
    mut editor_state: ResMut<EditorState>,
    preview_index_query: Query<&PreviewIndex>,
) {
    let preview_index = preview_index_query.get(trigger.target).unwrap();
    editor_state.selected = Some(*preview_index);
}
