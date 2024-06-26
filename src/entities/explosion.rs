use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::{seq::SliceRandom, Rng};

use crate::{
    particles::fire_particles::FireParticleRes, states::DespawnOnCleanup, utils::sets::Set,
};

#[derive(Component)]
pub struct ExplosionParticle {
    spawn_time: f32,
    size: f32,
}

#[derive(Event)]
pub struct ExplosionEvent {
    pub position: Vec3,
    pub parent: Option<Entity>,
    pub radius: f32,
}

impl Default for ExplosionEvent {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            parent: None,
            radius: 3.0,
        }
    }
}

fn spawn_explosion(
    mut events: EventReader<ExplosionEvent>,
    mut commands: Commands,
    time: Res<Time>,
    fire_res: Res<FireParticleRes>,
    velocity_query: Query<(&Velocity, &Transform)>,
) {
    const PARTICLE_COUNT: usize = 10;
    for event in events.read() {
        let mut rng = rand::thread_rng();

        let (parent_velocity, parent_pos) = if let Some(parent) = event.parent
            && let Ok((velocity, transform)) = velocity_query.get(parent)
        {
            (velocity.linvel, transform.translation)
        } else {
            (Vec3::ZERO, Vec3::ZERO)
        };

        for _ in 0..(PARTICLE_COUNT * event.radius as usize) {
            let scale = rng.gen_range(0.3..1.0);

            commands.spawn((
                DespawnOnCleanup,
                ExplosionParticle {
                    spawn_time: time.elapsed_seconds(),
                    size: scale * event.radius,
                },
                MaterialMeshBundle {
                    mesh: fire_res.mesh.clone(),
                    material: fire_res.materials.choose(&mut rng).unwrap().clone(),
                    transform: Transform {
                        translation: event.position + parent_pos,
                        rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                        ..default()
                    },
                    inherited_visibility: InheritedVisibility::VISIBLE,
                    ..default()
                },
                RigidBody::KinematicVelocityBased,
                Velocity {
                    linvel: parent_velocity
                        + Vec3::new(
                            rng.gen_range(-1.0..1.0),
                            rng.gen_range(-1.0..1.0),
                            rng.gen_range(-1.0..1.0),
                        )
                        .normalize()
                            * rng.gen_range(0.3..1.3)
                            * event.radius,
                    ..default()
                },
            ));
        }
    }
}

fn explosion_particle_update(
    time: Res<Time>,
    mut particles: Query<(&mut Transform, &ExplosionParticle, Entity)>,
    mut commands: Commands,
) {
    const START_PHASE_LENGTH: f32 = 0.2;
    const END_PHASE_LENGTH: f32 = 0.5;
    for (mut transform, particle, entity) in &mut particles {
        let lifetime = time.elapsed_seconds() - particle.spawn_time;
        if lifetime < START_PHASE_LENGTH {
            transform.scale = Vec3::splat(lifetime / START_PHASE_LENGTH) * particle.size;
        } else if lifetime < START_PHASE_LENGTH + END_PHASE_LENGTH {
            transform.scale = Vec3::splat(1.0 - (lifetime - START_PHASE_LENGTH) / END_PHASE_LENGTH)
                * particle.size;
        } else {
            commands.entity(entity).despawn();
        }
    }
}

pub struct ExplosionPlugin;

impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_explosion.after(Set::ExplosionEvents))
            .add_systems(Update, explosion_particle_update)
            .add_event::<ExplosionEvent>();
    }
}
