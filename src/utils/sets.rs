use bevy::ecs::schedule::SystemSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub enum Set {
    BulletEvents, 
    ExplosionEvents, 
}