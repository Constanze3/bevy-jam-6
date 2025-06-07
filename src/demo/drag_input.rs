use crate::{
    AppSystems, PausableSystems, Pause, asset_tracking::LoadResource, audio::sound_effect,
    screens::Screen,
};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<DragInputController>();
    app.init_resource::<DragInputController>();
    app.load_resource::<DragInputAssets>();

    app.add_event::<StretchInputEvent>();

    app.add_systems(
        Update,
        record_drag_input
            .run_if(in_state(Screen::Gameplay))
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );

    app.add_systems(OnEnter(Pause(true)), reset_drag_input);
}

#[derive(Asset, Resource, Clone, Reflect)]
#[reflect(Resource)]
struct DragInputAssets {
    #[dependency]
    drag_sound: Handle<AudioSource>,
}

impl FromWorld for DragInputAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        Self {
            drag_sound: assets.load::<AudioSource>("audio/sound_effects/drag.ogg"),
        }
    }
}

#[derive(Component)]
struct DragSound;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct DragInputController {
    pub initial_position: Option<Vec2>,
    pub vector: Option<Vec2>,
    pub min_length: f32,
    pub max_length: f32,
}

impl Default for DragInputController {
    fn default() -> Self {
        Self {
            initial_position: None,
            vector: None,
            min_length: 80.0,
            max_length: 250.0,
        }
    }
}

#[derive(Event)]
pub struct StretchInputEvent {
    pub vector: Vec2,
}

fn record_drag_input(
    input: Res<ButtonInput<MouseButton>>,
    mut input_controller: ResMut<DragInputController>,
    window_query: Query<&Window>,
    mut events: EventWriter<StretchInputEvent>,
    input_assets: Res<DragInputAssets>,
    drag_sound_query: Query<Entity, With<DragSound>>,
    mut commands: Commands,
) {
    let window = window_query.single().unwrap();

    // Record initial mouse position.
    if input.just_pressed(MouseButton::Left) {
        input_controller.initial_position = window.cursor_position();
        commands.spawn((DragSound, sound_effect(input_assets.drag_sound.clone())));
    }

    // Update vector of input controller.
    if input.pressed(MouseButton::Left) {
        let vector = calculate_vector(input_controller.initial_position, window.cursor_position());

        let vector = vector.map(|v| {
            if input_controller.max_length < v.length() {
                v.normalize() * input_controller.max_length
            } else {
                v
            }
        });

        input_controller.vector = vector;
    }

    // Send input event.
    if input.just_released(MouseButton::Left) {
        for drag_sound in drag_sound_query.iter() {
            commands.entity(drag_sound).despawn();
        }

        let vector = calculate_vector(input_controller.initial_position, window.cursor_position());

        if let Some(vector) = vector {
            if input_controller.min_length <= vector.length() {
                events.write(StretchInputEvent { vector });
            }
        }

        input_controller.initial_position = None;
        input_controller.vector = None;
    }
}

fn calculate_vector(
    initial_position: Option<Vec2>,
    current_position: Option<Vec2>,
) -> Option<Vec2> {
    if let Some(initial_position) = initial_position {
        if let Some(position) = current_position {
            let mut vector = initial_position - position;
            // In screen coordinates the y-axis is reversed.
            vector.y *= -1.0;

            return Some(vector);
        }
    }

    None
}

fn reset_drag_input(mut input_controller: ResMut<DragInputController>) {
    *input_controller = DragInputController::default();
}
