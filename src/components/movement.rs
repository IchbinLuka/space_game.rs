use bevy::prelude::*;
use bevy_rapier3d::dynamics::Velocity;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, max_speed_system);
    }
}

#[derive(Component)]
pub struct MaxSpeed {
    pub max_speed: f32,
}

#[derive(Component)]
pub struct Friction {
    pub friction: f32,
}

fn max_speed_system(mut query: Query<(&MaxSpeed, &mut Velocity)>) {
    for (max_speed, mut velocity) in &mut query {
        if velocity.linvel.length() > max_speed.max_speed {
            velocity.linvel = velocity.linvel.normalize() * max_speed.max_speed;
        }
    }
}
