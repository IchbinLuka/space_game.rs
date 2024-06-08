use bevy::{ecs::system::Command, prelude::*};
use bevy_rapier3d::{
    dynamics::Velocity,
    geometry::{CollisionGroups, Group},
};
use rand::Rng;

use crate::{
    components::movement::MaxSpeed,
    entities::{
        bullet::{BulletSpawnEvent, BulletTarget, BulletType},
        explosion::ExplosionEvent,
        powerup::SpawnPowerup,
        Enemy,
    },
    states::{game_running, DespawnOnCleanup, ON_GAME_STARTED},
    ui::{
        game_hud::{ScoreEvent, SpawnEnemyIndicator},
        health_bar_3d::SpawnHealthBar,
        minimap::{MinimapAssets, ShowOnMinimap},
    },
    utils::{
        collisions::{BOT_COLLISION_GROUP, CRUISER_COLLISION_GROUP},
        misc::Comparef32,
    },
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
const POWERUP_SPAWN_PROBABILITY: f64 = 0.3;

#[derive(Component)]
pub struct EnemyTarget;

#[derive(Component)]
pub struct Bot;

#[derive(Component)]
pub struct SquadLeader;

#[derive(Component)]
pub struct SquadMember {
    pub leader: Entity,
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
                squad_leader: Some(leader),
            }
            .apply(world);
        }
    }
}

pub struct SpawnBot {
    pub pos: Vec3,
    pub squad_leader: Option<Entity>,
}

impl Default for SpawnBot {
    fn default() -> Self {
        Self {
            pos: Vec3::ZERO,
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
        Bot,
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
        ShowOnMinimap {
            sprite: minimap_assets.enemy_indicator.clone(),
            size: 0.1.into(),
        },
        Enemy,
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
            let mut rng = rand::thread_rng();
            if rng.gen_bool(POWERUP_SPAWN_PROBABILITY) {
                commands.add(SpawnPowerup::random(transform.translation, &mut rng));
            }

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
            &mut LastBulletInfo,
            &Spaceship,
        ),
        (IsBot, Without<EnemyTarget>),
    >,
    target_query: Query<(&Transform, Entity), With<EnemyTarget>>,
    time: Res<Time>,
    mut bullet_spawn_events: EventWriter<BulletSpawnEvent>,
) {
    for (velocity, transform, mut last_bullet, spaceship) in &mut bots {
        let current_pos = transform.translation;
        let Some((target_transform, _)) = target_query
            .iter()
            .min_by_key(|(t, _)| Comparef32((t.translation - current_pos).length()))
        else {
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
    }
}

fn bot_movement(
    mut bots: Query<
        (&mut Transform, &mut Velocity, &Bot, &Spaceship, Entity),
        Without<SquadMember>,
    >,
    enemy_targets: Query<(&Transform, &EnemyTarget), Without<Bot>>,
    spaceship_collisions: Query<(&Transform, &SpaceshipCollisions), Without<Bot>>,
    time: Res<Time>,
    mut exhaust_particles: EventWriter<ParticleSpawnEvent>,
) {
    const C: f32 = 5.0;

    for (mut transform, mut velocity, _bot, spaceship, entity) in &mut bots {
        // Determine target direction by potential field path-planning
        let Some(target) = enemy_targets
            .iter()
            .min_by_key(|(t, _)| Comparef32((t.translation - transform.translation).length()))
        else {
            continue;
        };
        let distance = (target.0.translation - transform.translation).length();
        let f_attract = C
            * (target.0.translation - transform.translation).normalize()
            * if distance < 20.0 { -1.0 } else { 1.0 };
        let f_repulse = spaceship_collisions
            .iter()
            .map(|(t, collisions)| {
                let delta = transform.translation - t.translation;
                let distance = f32::max(distance - collisions.bound_radius, 0.01);
                if distance > 75.0 || distance < 0.01 {
                    return Vec3::ZERO;
                }

                let direction = delta.normalize();
                let magnitude = 1.0 / distance;
                direction * magnitude
            })
            .sum::<Vec3>();

        let f = f_attract + f_repulse;

        let angle = transform.forward().angle_between(f);
        let sign = angle_between_sign(*transform.forward(), f);

        transform.rotate_y(sign * f32::clamp(angle * 3.0, 1.0, 10.0) * time.delta_seconds());

        if angle < 0.3 {
            velocity.linvel +=
                transform.forward().normalize() * time.delta_seconds() * BOT_ACCELERATION;
            spaceship.main_exhaust(entity, &mut exhaust_particles);
        }
    }
}

fn bot_squad_update(
    mut squad_bots: Query<
        (
            &mut Velocity,
            &mut Transform,
            &SquadMember,
            Entity,
            &Spaceship,
        ),
        IsBot,
    >,
    leader_query: Query<&Transform, Without<SquadMember>>,
    mut commands: Commands,
    time: Res<Time>,
    mut exhaust_particles: EventWriter<ParticleSpawnEvent>,
) {
    for (mut velocity, mut transform, squad_member, entity, spaceship) in &mut squad_bots {
        let Ok(leader_transform) = leader_query.get(squad_member.leader) else {
            commands.entity(entity).remove::<SquadMember>();
            continue;
        };

        let delta = leader_transform.translation - transform.translation;
        let distance = delta.length();

        let angle = transform.forward().angle_between(delta);

        let sign = angle_between_sign(*transform.forward(), delta);

        transform.rotate_y(sign * 5.0 * time.delta_seconds());

        if angle < 0.6 && distance > 5.0 {
            velocity.linvel +=
                transform.forward().normalize() * time.delta_seconds() * BOT_ACCELERATION;
            spaceship.main_exhaust(entity, &mut exhaust_particles);
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

        let sign = angle_between_sign(*transform.forward(), delta);

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
                bot_movement,
                // bot_avoid_collisions,
            )
                .run_if(game_running()),
        )
        .add_systems(ON_GAME_STARTED, bot_setup);
    }
}
