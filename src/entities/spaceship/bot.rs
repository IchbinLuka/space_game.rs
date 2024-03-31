use std::cmp::Ordering;

use bevy::{ecs::system::Command, prelude::*};
use bevy_rapier3d::{
    dynamics::Velocity,
    geometry::{CollisionGroups, Group},
};
use rand::Rng;

use crate::{states::DespawnOnCleanup, ui::minimap::{MinimapAssets, ShowOnMinimap}};
use crate::{
    components::movement::MaxSpeed,
    entities::{
        bullet::{BulletSpawnEvent, BulletTarget, BulletType},
        explosion::ExplosionEvent,
    },
    states::game_running,
    ui::{enemy_indicator::SpawnEnemyIndicator, health_bar_3d::SpawnHealthBar, score::ScoreEvent},
    utils::{
        collisions::{BOT_COLLISION_GROUP, CRUISER_COLLISION_GROUP},
        math,
    },
};
use crate::{
    materials::toon::{ApplyToonMaterial, ToonMaterial},
    states::ON_GAME_STARTED,
};

use super::{
    Health, IsBot, LastBulletInfo, ParticleSpawnEvent, Spaceship, SpaceshipAssets, SpaceshipBundle,
    SpaceshipCollisions,
};

const BOT_ACCELERATION: f32 = 20.0;
const COLLISION_GROUPS: CollisionGroups = CollisionGroups::new(
    BOT_COLLISION_GROUP,
    Group::ALL.difference(CRUISER_COLLISION_GROUP),
);

#[derive(Component)]
pub struct EnemyTarget;

#[derive(Component)]
pub struct Bot {
    pub state: BotState,
    // pub current_target: Option<Entity>
}

#[derive(Component)]
pub struct SquadLeader;

#[derive(Component)]
pub struct SquadMember {
    pub leader: Entity,
}

#[derive(Clone, Copy)]
pub enum BotState {
    Chasing,
    Fleeing,
}

pub struct SpawnSquad {
    pub squad_size: u16,
    pub leader_pos: Vec3,
}

impl Command for SpawnSquad {
    fn apply(self, world: &mut World) {
        let Ok(leader) = spawn_bot_from_world(
            world,
            SpawnBot {
                pos: self.leader_pos,
                initial_state: BotState::Chasing,
                ..default()
            },
        ) else {
            error!("Failed to spawn squad leader");
            return;
        };
        world.entity_mut(leader).insert(SquadLeader);

        for _ in 1..self.squad_size {
            let pos = Vec3::new(
                self.leader_pos.x + rand::thread_rng().gen_range(-5.0..5.0),
                self.leader_pos.y,
                self.leader_pos.z + rand::thread_rng().gen_range(-5.0..5.0),
            );

            SpawnBot {
                pos,
                initial_state: BotState::Fleeing,
                squad_leader: Some(leader),
            }
            .apply(world);
        }
    }
}

pub struct SpawnBot {
    pub pos: Vec3,
    pub initial_state: BotState,
    pub squad_leader: Option<Entity>,
}

impl Default for SpawnBot {
    fn default() -> Self {
        Self {
            pos: Vec3::ZERO,
            initial_state: BotState::Chasing,
            squad_leader: None,
        }
    }
}

fn spawn_bot_from_world(world: &mut World, spawn_bot: SpawnBot) -> Result<Entity, ()> {
    let Some(assets) = world.get_resource::<SpaceshipAssets>() else {
        return Err(());
    };

    let Some(minimap_assets) = world.get_resource::<MinimapAssets>() else {
        return Err(());
    };

    let mut entity_commands = world.spawn((
        Bot {
            state: spawn_bot.initial_state,
            // current_target: None,
        },
        LastBulletInfo::with_cooldown(0.5),
        SpaceshipBundle {
            collision_groups: COLLISION_GROUPS,
            ..SpaceshipBundle::new(assets.enemy_ship.clone(), spawn_bot.pos)
        },
        MaxSpeed { max_speed: 30.0 },
        Health::new(20.0),
        BulletTarget {
            target_type: BulletType::Player,
            bullet_damage: Some(10.0),
        },
        ApplyToonMaterial {
            base_material: ToonMaterial {
                filter_scale: 0.0,
                ..default()
            },
        },
        ShowOnMinimap {
            sprite: minimap_assets.enemy_indicator.clone(),
            size: Some(Vec2::splat(10.)),
        },
        DespawnOnCleanup, 
    ));

    if let Some(leader) = spawn_bot.squad_leader {
        entity_commands.insert(SquadMember { leader });
    }

    let entity = entity_commands.id();

    SpawnHealthBar {
        entity,
        scale: 0.2,
        offset: Vec2::new(0., -20.),
        shield_entity: None,
    }
    .apply(world);

    SpawnEnemyIndicator { enemy: entity }.apply(world);
    Ok(entity)
}

impl Command for SpawnBot {
    fn apply(self, world: &mut World) {
        if let Err(()) = spawn_bot_from_world(world, self) {
            error!("Failed to spawn bot");
        }
    }
}

fn bot_death(
    mut commands: Commands,
    mut explosions: EventWriter<ExplosionEvent>,
    mut scores: EventWriter<ScoreEvent>,
    bots: Query<(Entity, &GlobalTransform, &Health), (IsBot, Changed<Health>)>,
) {
    for (entity, global_transform, health) in &bots {
        if health.is_dead() {
            let transform = global_transform.compute_transform();
            explosions.send(ExplosionEvent {
                parent: None,
                position: transform.translation,
                radius: 5.0,
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
            &Health,
            Entity,
            &mut LastBulletInfo,
            &Spaceship,
            Option<&SquadMember>,
        ),
        (IsBot, Without<EnemyTarget>),
    >,
    target_query: Query<(&Transform, Entity), With<EnemyTarget>>,

    time: Res<Time>,
    mut exhaust_particles: EventWriter<ParticleSpawnEvent>,
    mut bullet_spawn_events: EventWriter<BulletSpawnEvent>,
) {
    let mut rng = rand::thread_rng();

    for (
        mut velocity,
        mut transform,
        mut bot,
        health,
        entity,
        mut last_bullet,
        spaceship,
        squad_member,
    ) in &mut bots
    {
        let current_pos = transform.translation;
        let Some((target_transform, _)) = target_query.iter().min_by(|(t1, _), (t2, _)| {
            let diff =
                (t1.translation - current_pos).length() - (t2.translation - current_pos).length();
            if diff < 0.0 {
                Ordering::Less
            } else if diff > 0.0 {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }) else {
            continue;
        };
        if !last_bullet.timer.finished() {
            last_bullet.timer.tick(time.delta());
        }

        let delta = target_transform.translation - transform.translation;
        let distance = delta.length();
        let angle = transform.forward().angle_between(delta);

        if last_bullet.timer.finished() &&
                       angle < 0.1 &&  // Angle should be small
                       distance < 50.0
        // Enemy should only shoot when close
        {
            spaceship.shoot(
                &mut last_bullet,
                &mut bullet_spawn_events,
                &transform,
                *velocity,
                BulletType::Bot,
            );
            last_bullet.timer.tick(time.delta());
        }

        if squad_member.is_some() {
            // Squad members are handled seperately in bot_squad_update
            continue;
        }

        match bot.state {
            BotState::Chasing => {
                let mut sign = angle_between_sign(transform.forward(), delta);

                if distance < 30.0 {
                    sign *= -1.0;
                }

                transform.rotate_y(sign * 3.0 * time.delta_seconds());

                if angle < 0.3 || distance < 20.0 {
                    velocity.linvel +=
                        transform.forward().normalize() * time.delta_seconds() * BOT_ACCELERATION;
                    exhaust_particles.send(ParticleSpawnEvent::main_exhaust(entity));
                }

                if rng.gen_bool(0.01 * (1. - health.health / health.max_health) as f64) {
                    bot.state = BotState::Fleeing;
                }
            }
            BotState::Fleeing => {
                let angle = delta.angle_between(-transform.forward());
                if angle > 0.1 {
                    let cross = (-transform.forward()).cross(delta);
                    let sign = cross.y.signum();
                    transform.rotate_y(sign * 3.0 * time.delta_seconds());
                }

                if angle < 0.3 {
                    velocity.linvel +=
                        transform.forward().normalize() * time.delta_seconds() * BOT_ACCELERATION;
                    exhaust_particles.send(ParticleSpawnEvent::main_exhaust(entity));
                }

                if rng.gen_bool(0.01) || distance > 100.0 {
                    bot.state = BotState::Chasing;
                }
            }
        }
    }
}

fn bot_avoid_collisions(
    spaceship_collisions: Query<(&Transform, &SpaceshipCollisions), Without<Bot>>,
    mut bots: Query<(&mut Transform, &mut Velocity, Entity), IsBot>,
    time: Res<Time>,
    mut exhaust_particles: EventWriter<ParticleSpawnEvent>,
) {
    let colliders = spaceship_collisions.iter().collect::<Vec<_>>();
    for (mut transform, mut velocity, entity) in &mut bots {
        for (other_transform, other_collisions) in &colliders {
            let delta = other_transform.translation - transform.translation;
            let distance = delta.length();

            if distance > other_collisions.bound_radius + 20.0 {
                continue;
            }

            if math::sphere_intersection(
                other_transform.translation,
                other_collisions.bound_radius,
                transform.translation,
                velocity.linvel.normalize(),
            )
            .is_none()
            {
                velocity.linvel +=
                    transform.forward().normalize() * time.delta_seconds() * BOT_ACCELERATION;
                exhaust_particles.send(ParticleSpawnEvent::main_exhaust(entity));
                continue;
            }

            let sign = angle_between_sign(transform.forward(), delta);
            transform.rotate_y(-sign * 4.0 * time.delta_seconds());
        }
    }
}

fn bot_squad_update(
    mut squad_bots: Query<(&mut Velocity, &mut Transform, &SquadMember, Entity), IsBot>,
    leader_query: Query<&Transform, Without<SquadMember>>,
    mut commands: Commands,
    time: Res<Time>,
    mut exhaust_particles: EventWriter<ParticleSpawnEvent>,
) {
    for (mut velocity, mut transform, squad_member, entity) in &mut squad_bots {
        let Ok(leader_transform) = leader_query.get(squad_member.leader) else {
            commands.entity(entity).remove::<SquadMember>();
            continue;
        };

        let delta = leader_transform.translation - transform.translation;
        let distance = delta.length();

        let angle = transform.forward().angle_between(delta);

        let sign = angle_between_sign(transform.forward(), delta);

        transform.rotate_y(sign * 5.0 * time.delta_seconds());

        if angle < 0.6 && distance > 5.0 {
            velocity.linvel +=
                transform.forward().normalize() * time.delta_seconds() * BOT_ACCELERATION;
            exhaust_particles.send(ParticleSpawnEvent::main_exhaust(entity));
        }
    }
}

#[inline(always)]
fn angle_between_sign(a: Vec3, b: Vec3) -> f32 {
    let cross = a.cross(b);
    cross.y.signum()
}

fn bot_repulsion(
    mut bots: Query<(&mut Transform, &Bot), (IsBot, Without<SquadLeader>)>,
    time: Res<Time>,
) {
    let mut combinations = bots.iter_combinations_mut();
    while let Some([(mut transform, _bot), (transform_2, _bot_2)]) = combinations.fetch_next() {
        let delta = transform_2.translation - transform.translation;
        let distance = delta.length();

        if distance > 10.0 {
            continue;
        }

        let sign = angle_between_sign(transform.forward(), delta);

        transform.rotate_y(-sign * 4.0 * time.delta_seconds());
    }
}

fn bot_setup(mut commands: Commands) {
    commands.add(SpawnSquad {
        squad_size: 3,
        leader_pos: Vec3::new(0.0, 0.0, 50.0),
    });
}

pub struct BotPlugin;

impl Plugin for BotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                bot_update,
                bot_death,
                bot_repulsion,
                bot_squad_update,
                bot_avoid_collisions,
            )
                .run_if(game_running()),
        )
        .add_systems(ON_GAME_STARTED, bot_setup);
    }
}
