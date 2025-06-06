use std::time::Duration;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{AppSystems, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    app.add_event::<InvincibleRemoved>();
    app.add_systems(
        Update,
        tick_invincibility
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

#[derive(Component, Serialize, Deserialize)]
pub struct Invincible(Timer);

impl Invincible {
    pub fn new(duration: Duration) -> Self {
        Self(Timer::new(duration, TimerMode::Once))
    }
}

#[derive(Event)]
pub struct InvincibleRemoved(pub Entity);

fn tick_invincibility(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Invincible)>,
    mut events: EventWriter<InvincibleRemoved>,
    mut commands: Commands,
) {
    for (entity, mut invincible) in query.iter_mut() {
        invincible.0.tick(time.delta());

        if invincible.0.just_finished() {
            commands.entity(entity).remove::<Invincible>();
            events.write(InvincibleRemoved(entity));
        }
    }
}
