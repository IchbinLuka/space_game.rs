use bevy::{prelude::*, ecs::query::{WorldQuery, ReadOnlyWorldQuery}};
use bevy_rapier3d::geometry::CollidingEntities;

pub trait CollidingEntitiesExtension {
    fn fulfills_query<Q: WorldQuery, F: ReadOnlyWorldQuery>(&self, query: &Query<Q, F>) -> bool;
}

impl CollidingEntitiesExtension for CollidingEntities {
    #[inline]
    fn fulfills_query<Q: WorldQuery, F: ReadOnlyWorldQuery>(&self, query: &Query<Q, F>) -> bool {
        for entity in self.iter() {
            if query.get(entity).is_ok() {
                return true;
            }
        }
        false
    }
}