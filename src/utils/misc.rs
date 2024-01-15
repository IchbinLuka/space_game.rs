use bevy::{prelude::*, ecs::query::{WorldQuery, ReadOnlyWorldQuery}};
use bevy_rapier3d::geometry::CollidingEntities;

pub trait CollidingEntitiesExtension {
    fn fulfills_query<'world, 'state, Q: WorldQuery, F: ReadOnlyWorldQuery>(&self, query: &Query<'world, 'state, Q, F>) -> bool;
}

impl CollidingEntitiesExtension for CollidingEntities {
    fn fulfills_query<'world, 'state, Q: WorldQuery, F: ReadOnlyWorldQuery>(&self, query: &Query<'world, 'state, Q, F>) -> bool {
        for entity in self.iter() {
            if query.get(entity).is_ok() {
                return true;
            }
        }
        false
    }
}