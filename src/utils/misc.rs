use bevy::{
    ecs::query::{ROQueryItem, ReadOnlyWorldQuery, WorldQuery},
    prelude::*,
};
use bevy_rapier3d::geometry::CollidingEntities;

pub trait CollidingEntitiesExtension {
    fn fulfills_query<Q: WorldQuery, F: ReadOnlyWorldQuery>(&self, query: &Query<Q, F>) -> bool;

    fn filter_fulfills_query<'a, Q, F>(
        &self,
        query: &'a Query<Q, F>,
    ) -> impl Iterator<Item = ROQueryItem<'a, Q>>
    where
        Q: WorldQuery,
        F: ReadOnlyWorldQuery;
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

    #[inline]
    fn filter_fulfills_query<'a, Q, F>(
        &self,
        query: &'a Query<Q, F>,
    ) -> impl Iterator<Item = ROQueryItem<'a, Q>>
    where
        Q: WorldQuery,
        F: ReadOnlyWorldQuery,
    {
        self.iter().filter_map(|e| query.get(e).ok())
    }
}
