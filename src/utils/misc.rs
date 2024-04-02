use std::cmp::Ordering;

use bevy::{
    ecs::{
        query::{QueryData, QueryFilter, ROQueryItem},
        system::RunSystemOnce,
    },
    prelude::*,
};
use bevy_rapier3d::geometry::CollidingEntities;

pub trait CollidingEntitiesExtension {
    fn _fulfills_query<D: QueryData, F: QueryFilter>(&self, query: &Query<D, F>) -> bool;

    fn filter_fulfills_query<'a, D, F>(
        &self,
        query: &'a Query<D, F>,
    ) -> impl Iterator<Item = ROQueryItem<'a, D>>
    where
        D: QueryData, 
        F: QueryFilter;
}

impl CollidingEntitiesExtension for CollidingEntities {
    #[inline]
    fn _fulfills_query<D: QueryData, F: QueryFilter>(&self, query: &Query<D, F>) -> bool {
        for entity in self.iter() {
            if query.get(entity).is_ok() {
                return true;
            }
        }
        false
    }

    #[inline]
    fn filter_fulfills_query<'a, D, F>(
        &self,
        query: &'a Query<D, F>,
    ) -> impl Iterator<Item = ROQueryItem<'a, D>>
    where
        D: QueryData,
        F: QueryFilter,
    {
        self.iter().filter_map(|e| query.get(e).ok())
    }
}

pub trait AsCommand<In, Out, Marker> {
    fn to_command(self, input: In) -> impl FnOnce(&mut World) -> Out;
}

impl<In, Out, Marker, T: IntoSystem<In, Out, Marker>> AsCommand<In, Out, Marker> for T {
    fn to_command(self, input: In) -> impl FnOnce(&mut World) -> Out {
        |world: &mut World| world.run_system_once_with(input, self)
    }
}

/// A wrapper type for `f32` that implements `PartialOrd` and `Ord` traits.
///
/// Note: since we are dealing with floating point numbers, this may not always work as expected.
#[derive(Deref, DerefMut, PartialEq)]
pub struct Comparef32(pub f32);

impl Eq for Comparef32 {}

impl PartialOrd for Comparef32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Comparef32 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap_or(Ordering::Equal)
    }
}

impl From<f32> for Comparef32 {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_f32() {
        assert!(Comparef32(10.0) > Comparef32(0.0));
        assert!(Comparef32(0.0) < Comparef32(10.0));
        assert!(Comparef32(0.0) == Comparef32(0.0));
        assert!(Comparef32(-10.0) < Comparef32(0.0));
        assert!(Comparef32(0.0) > Comparef32(-10.0));
    }
}
