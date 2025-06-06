use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.configure_sets(PostUpdate, CollisionHandlers.after(PhysicsSet::Writeback));
}

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct CollisionHandlers;

/// Traverses the hierarchy from the given entity until the first ancestor with a rigid body.
pub fn find_rigidbody_ancestor(
    mut entity: Entity,
    query: &Query<(Option<&RigidBody>, &ChildOf)>,
) -> Option<Entity> {
    loop {
        let Ok((rigid_body, parent)) = query.get(entity) else {
            return None;
        };

        if rigid_body.is_some() {
            return Some(entity);
        }

        entity = parent.0;
    }
}
