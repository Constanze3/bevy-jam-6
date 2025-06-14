use bevy::{ecs::system::QueryLens, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::{
    PausableSystems,
    demo::player::Player,
    physics::{CollisionHandlerSystems, find_rigidbody_ancestor},
    screens::Screen,
};

use super::time_scale::{SetTimeScale, SetTimeScaleOverride, TimeScaleKind};

pub(super) fn plugin(app: &mut App) {
    app.add_event::<KillEvent>();

    app.add_systems(
        PostUpdate,
        (
            killer_collision_handler
                .in_set(CollisionHandlerSystems)
                .in_set(PausableSystems)
                .run_if(in_state(Screen::Gameplay)),
            kill.after(CollisionHandlerSystems)
                .run_if(in_state(Screen::Gameplay)),
        ),
    );
}

#[derive(Component)]
pub struct Killer;

#[derive(Event)]
pub struct KillEvent {
    pub player: Entity,
}

fn killer_collision_handler(
    mut collision_events: EventReader<CollisionEvent>,
    mut query: Query<(
        Option<&Killer>,
        Option<&Player>,
        Option<&RigidBody>,
        &ChildOf,
    )>,
    mut events: EventWriter<KillEvent>,
) {
    for event in collision_events.read() {
        let CollisionEvent::Started(e1, e2, _) = *event else {
            return;
        };

        let mut helper_lens: QueryLens<(Option<&RigidBody>, &ChildOf)> = query.transmute_lens();
        let helper_query = helper_lens.query();
        let Some(e1) = find_rigidbody_ancestor(e1, &helper_query) else {
            return;
        };
        let Some(e2) = find_rigidbody_ancestor(e2, &helper_query) else {
            return;
        };

        let (e1_killer, e1_player, _, _) = query.get(e1).unwrap();
        let (e2_killer, e2_player, _, _) = query.get(e2).unwrap();

        if e1_killer.is_some() && e2_player.is_some() {
            events.write(KillEvent { player: e2 });
            return;
        }

        if e2_killer.is_some() && e1_player.is_some() {
            events.write(KillEvent { player: e1 });
            return;
        }
    }
}

fn kill(
    mut events: EventReader<KillEvent>,
    mut time_events: EventWriter<SetTimeScale>,
    mut time_override_events: EventWriter<SetTimeScaleOverride>,
    mut commands: Commands,
) {
    for event in events.read() {
        commands.entity(event.player).despawn();

        time_override_events.write(SetTimeScaleOverride(None));
        time_events.write(SetTimeScale(TimeScaleKind::Normal));
    }
}
