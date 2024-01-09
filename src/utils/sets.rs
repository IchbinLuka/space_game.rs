use bevy::ecs::schedule::SystemSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, SystemSet)]
pub enum Set {
    BulletEvents,
    ExplosionEvents,
}
