use bevy::ecs::bundle::Bundle;
use bevy_rapier3d::prelude::*;

#[derive(Bundle)]
pub struct VelocityColliderBundle {
    pub velocity: Velocity,
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub active_events: ActiveEvents,
    pub active_collision_types: ActiveCollisionTypes,
    pub colliding_entities: CollidingEntities,
}

impl Default for VelocityColliderBundle {
    fn default() -> Self {
        Self {
            velocity: Velocity::default(),
            collider: Collider::default(),
            rigid_body: RigidBody::KinematicVelocityBased,
            active_events: ActiveEvents::COLLISION_EVENTS,
            active_collision_types: ActiveCollisionTypes::all(),
            colliding_entities: CollidingEntities::default(),
        }
    }
}
