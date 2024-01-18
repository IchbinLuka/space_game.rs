use bevy::prelude::*;
use bevy_rapier3d::dynamics::Velocity;
use rand::Rng;

use crate::{components::movement::MaxSpeed, entities::{bullet::BulletSpawnEvent, explosion::ExplosionEvent}, AppState, ui::score::ScoreEvent};

use super::{
    IsBot, IsPlayer, LastBulletInfo, ParticleSpawnEvent, SpaceshipAssets, SpaceshipBundle, Health,
};

#[derive(Component)]
pub struct Bot {
    pub state: BotState,
}

#[derive(Clone, Copy)]
pub enum BotState {
    Chasing,
    Fleeing,
}

#[derive(Event)]
pub struct BotSpawnEvent {
    pub pos: Vec3,
    pub initial_state: BotState,
}

fn spawn_bot(
    mut spawn_events: EventReader<BotSpawnEvent>,
    mut commands: Commands,
    assets: Res<SpaceshipAssets>,
) {
    for event in spawn_events.read() {
        commands.spawn((
            Bot {
                state: event.initial_state,
            },
            LastBulletInfo::default(),
            SpaceshipBundle::new(assets.enemy_ship.clone(), Vec3::new(0.0, 0.0, 10.0)),
            MaxSpeed { max_speed: 30.0 },
            Health(20.0), 
        ));
    }
}

fn bot_death(
    mut commands: Commands, 
    mut explosions: EventWriter<ExplosionEvent>,
    mut scores: EventWriter<ScoreEvent>,
    bots: Query<(Entity, &Transform, &Health), IsBot>,
) {
    for (entity, transform, health) in &bots {
        if health.0 <= 0.0 {
            explosions.send(ExplosionEvent {
                parent: Some(entity),
                position: transform.translation,
            });
            scores.send(ScoreEvent {
                score: 300, 
                world_pos: transform.translation,
            });
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn bot_update(
    mut bots: Query<
        (
            &mut Velocity,
            &mut Transform,
            &mut Bot,
            Entity,
            &mut LastBulletInfo,
        ),
        IsBot,
    >,
    player_query: Query<&Transform, IsPlayer>,
    time: Res<Time>,
    mut exhaust_particles: EventWriter<ParticleSpawnEvent>,
    mut bullet_spawn_events: EventWriter<BulletSpawnEvent>,
) {
    let mut rng = rand::thread_rng();

    for (mut velocity, mut transform, mut bot, entity, mut last_bullet) in &mut bots {
        if !last_bullet.timer.finished() {
            last_bullet.timer.tick(time.delta());
        }
        let Ok(player_transform) = player_query.get_single() else {
            continue;
        };

        let delta = player_transform.translation - transform.translation;
        let distance = delta.length();

        match bot.state {
            BotState::Chasing => {
                let angle = transform.forward().angle_between(delta);

                let cross = transform.forward().cross(delta);
                let mut sign = cross.y.signum();

                if distance < 20.0 {
                    sign *= -1.0;
                }

                transform.rotate_y(sign * 5.0 * time.delta_seconds());

                if angle < 0.1 || distance < 20.0 {
                    velocity.linvel += transform.forward().normalize();
                    exhaust_particles.send(ParticleSpawnEvent { entity });
                }

                if last_bullet.timer.finished() &&
                       angle < 0.1 &&  // Angle should be small
                       distance < 50.0
                // Enemy should only shoot when close
                {
                    // TODO: duplicate code
                    let side = last_bullet.side;
                    let pos = transform.translation + transform.rotation.mul_vec3(side.into());
                    let mut bullet_transform = Transform::from_translation(pos);

                    bullet_transform.rotate(transform.rotation);
                    debug!("Spawning bullet");
                    bullet_spawn_events.send(BulletSpawnEvent {
                        position: bullet_transform,
                        entity_velocity: *velocity,
                        direction: transform.forward(),
                        entity,
                    });

                    last_bullet.side = side.other();
                    last_bullet.timer.tick(time.delta());
                }

                if rng.gen_bool(0.001) {
                    bot.state = BotState::Fleeing;
                }
            }
            BotState::Fleeing => {
                let angle = delta.angle_between(-transform.forward());
                if angle > 0.1 {
                    let cross = transform.forward().cross(delta);
                    let sign = if cross.y > 0.0 { 1.0 } else { -1.0 };
                    transform.rotate_y(sign * 5.0 * time.delta_seconds());
                } else {
                    velocity.linvel += transform.forward().normalize();
                }

                if rng.gen_bool(0.002) {
                    bot.state = BotState::Chasing;
                }
            }
        }
    }
}


fn bot_setup(mut bot_events: EventWriter<BotSpawnEvent>) {
    bot_events.send(BotSpawnEvent {
        pos: Vec3::new(0.0, 0.0, 100.0),
        initial_state: BotState::Chasing,
    });
}

pub struct BotPlugin;

impl Plugin for BotPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<BotSpawnEvent>()
            .add_systems(Update,(
                spawn_bot, 
                bot_update, 
                bot_death,
            ).run_if(in_state(AppState::MainScene)),)
            .add_systems(OnEnter(AppState::MainScene), (
                bot_setup,
            ));
    }
}
