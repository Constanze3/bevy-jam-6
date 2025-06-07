use bevy::prelude::*;
use bevy_rapier2d::plugin::TimestepMode;

use crate::{AppSystems, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<TimeScale>();
    app.register_type::<TimeScaleOverride>();

    app.init_resource::<TimeScale>();
    app.init_resource::<TimeScaleOverride>();

    app.add_event::<SetTimeScale>();
    app.add_event::<SetTimeScaleOverride>();

    app.add_systems(
        Update,
        (set_time_scale, set_time_scale_override)
            .in_set(AppSystems::Update)
            .run_if(in_state(Screen::Gameplay)),
    );
}

#[derive(Default, Clone, Copy, Reflect)]
pub enum TimeScaleKind {
    #[default]
    Normal,
    Slowed,
}

impl TimeScaleKind {
    fn value(&self) -> f32 {
        match *self {
            TimeScaleKind::Normal => 1.0,
            TimeScaleKind::Slowed => 0.1,
        }
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
struct TimeScale(TimeScaleKind);

#[derive(Event)]
pub struct SetTimeScale(pub TimeScaleKind);

fn set_time_scale(
    mut events: EventReader<SetTimeScale>,
    mut time_scale_resource: ResMut<TimeScale>,
    time_scale_override: Res<TimeScaleOverride>,
    mut timestep_mode: ResMut<TimestepMode>,
) {
    for event in events.read() {
        time_scale_resource.0 = event.0;

        if time_scale_override.0.is_none() {
            if let TimestepMode::Variable { time_scale, .. } = timestep_mode.as_mut() {
                *time_scale = time_scale_resource.0.value();
            }
        }
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
struct TimeScaleOverride(Option<TimeScaleKind>);

#[derive(Event)]
pub struct SetTimeScaleOverride(pub Option<TimeScaleKind>);

fn set_time_scale_override(
    mut events: EventReader<SetTimeScaleOverride>,
    time_scale_resource: Res<TimeScale>,
    mut time_scale_override: ResMut<TimeScaleOverride>,
    mut timestep_mode: ResMut<TimestepMode>,
) {
    for event in events.read() {
        time_scale_override.0 = event.0;

        if let TimestepMode::Variable { time_scale, .. } = timestep_mode.as_mut() {
            if let Some(ov) = time_scale_override.0 {
                *time_scale = ov.value();
            } else {
                *time_scale = time_scale_resource.0.value();
            }
        }
    }
}
