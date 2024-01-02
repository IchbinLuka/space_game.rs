use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::{seq::SliceRandom, Rng};

use crate::particles::fire_particles::FireParticleRes;


#[derive(Component)]
pub struct ExplosionParticle {
    spawn_time: f32,
}

#[derive(Event)]
pub struct ExplosionEvent {
    pub position: Vec3,
    // pub radius: f32,
}

fn spawn_explosion(
    mut events: EventReader<ExplosionEvent>, 
    mut commands: Commands,
    time: Res<Time>,
    fire_res: Res<FireParticleRes>,
) {
    const PARTICLE_COUNT: usize = 20;
    for event in events.read() {
        let mut rng = rand::thread_rng();
        for _ in 0..PARTICLE_COUNT {
            let scale = Vec3::splat(rng.gen_range(0.7..1.4));

            commands.spawn((
                ExplosionParticle {
                    spawn_time: time.elapsed_seconds(),
                },
                PbrBundle {
                    mesh: fire_res.mesh.clone(),
                    material: fire_res.materials.choose(&mut rng).unwrap().clone(),
                    transform: Transform {
                        translation: event.position,
                        scale,
                        rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                    },
                    ..default()
                }, 
                RigidBody::KinematicVelocityBased, 
                Velocity {
                    linvel: Vec3::new(
                        rng.gen_range(-1.0..1.0), 
                        rng.gen_range(-1.0..1.0), 
                        rng.gen_range(-1.0..1.0)
                    ).normalize() * rng.gen_range(1.0..4.0),
                    ..default()
                }
            ));
        }
    }
}

fn explosion_particle_update(
    time: Res<Time>, 
    mut particles: Query<(&mut Transform, &ExplosionParticle, Entity)>, 
    mut commands: Commands
) {
    const START_PHASE_LENGTH: f32 = 0.2;
    const END_PHASE_LENGTH: f32 = 0.5;
    for (mut transform, particle, entity) in &mut particles {
        let lifetime = time.elapsed_seconds() - particle.spawn_time;
        if lifetime < START_PHASE_LENGTH {
            transform.scale = Vec3::splat(lifetime / START_PHASE_LENGTH) * 3.0;
        } else if lifetime < START_PHASE_LENGTH + END_PHASE_LENGTH {
            transform.scale = Vec3::splat(1.0 - (lifetime - START_PHASE_LENGTH) / END_PHASE_LENGTH) * 3.0;
        } else {
            commands.entity(entity).despawn();
        }
    }
}



pub struct ExplosionPlugin;

impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, spawn_explosion)
            .add_systems(Update, explosion_particle_update)
            .add_event::<ExplosionEvent>();
    }
}