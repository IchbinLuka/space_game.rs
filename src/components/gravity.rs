use bevy::prelude::*;
use bevy_rapier3d::dynamics::Velocity;

#[derive(Component)]
pub struct GravitySource {
    pub mass: f32,
    pub radius: Option<f32>,
}

impl Default for GravitySource {
    fn default() -> Self {
        Self {
            mass: 0.0,
            radius: None,
        }
    }
}

#[derive(Component)]
pub struct GravityAffected;

fn gravity_system(
    time: Res<Time>,
    sources: Query<(&Transform, &GravitySource), Without<GravityAffected>>, 
    mut affected: Query<(&mut Velocity, &mut Transform), (With<GravityAffected>, Without<GravitySource>)>,
) {
    for (source_transform, source) in &sources {
        for (mut velocity, mut transform) in &mut affected {
            let distance = source_transform.translation.distance(transform.translation);
            if distance < 0.01 { continue; }
            if let Some(radius) = source.radius {
                if distance < radius { continue; }
            }
            let acc = source.mass / (distance * distance);
            let new_vel = velocity.linvel + (source_transform.translation - transform.translation).normalize() * acc * time.delta_seconds();
            let delta_angle = Vec3::X.angle_between(new_vel) - Vec3::X.angle_between(velocity.linvel);
            transform.rotate_y(delta_angle);
            velocity.linvel = new_vel;
        }
    }
}


pub struct GravityPlugin;

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, gravity_system);
    }
}