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
    for (mut velocity, transform) in &mut affected {
        let current_vel = velocity.linvel;
        velocity.linvel += sources.iter().map(|(source_transform, source)| {
            gravity_step(source_transform, source, time.delta_seconds(), transform.translation, current_vel)
        }).sum::<Vec3>();
    }
}

#[inline(always)]
pub fn gravity_step(
    source_transform: &Transform,
    source: &GravitySource,
    delta_time: f32, 
    pos: Vec3,
    vel: Vec3,
) -> Vec3 {
    let distance = source_transform.translation.distance(pos);

    if distance < 0.01 { return vel; }
    if let Some(radius) = source.radius {
        if distance < radius { return vel; }
    }
    let acc = source.mass / (distance * distance);
    (source_transform.translation - pos).normalize() * acc * delta_time
}


pub struct GravityPlugin;

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, gravity_system);
    }
}