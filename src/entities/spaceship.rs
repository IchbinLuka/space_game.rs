use std::time::Duration;

use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_mod_outline::{OutlineBundle, OutlineVolume};
use bevy_rapier3d::prelude::*;
use rand::{seq::SliceRandom, Rng};

use crate::{
    components::{
        colliders::VelocityColliderBundle, despawn_after::DespawnAfter, gravity::GravityAffected,
    },
    particles::fire_particles::FireParticleRes,
    utils::sets::Set,
    AppState,
};

use self::{bot::Bot, player::Player};

use super::{
    bullet::{Bullet, BULLET_COLLISION_GROUP},
    explosion::ExplosionEvent,
    planet::Planet,
};

pub mod bot;
pub mod player;

type IsPlayer = (With<Player>, Without<Bot>);
type IsBot = (With<Bot>, Without<Player>);

#[derive(Component)]
pub struct Health(pub f32);

impl Health {
    pub fn take_damage(&mut self, damage: f32) {
        self.0 = (self.0 - damage).max(0.0);
    }
}


const BULLET_COOLDOWN: f32 = 0.2;

#[derive(Resource, Component)]
struct LastBulletInfo {
    side: BulletSide,
    timer: Timer,
}

impl Default for LastBulletInfo {
    fn default() -> Self {
        Self {
            side: BulletSide::default(),
            timer: Timer::from_seconds(BULLET_COOLDOWN, TimerMode::Repeating),
        }
    }
}

#[derive(Clone, Copy, Default)]
enum BulletSide {
    #[default]
    Left,
    Right,
}

impl BulletSide {
    const LEFT_POSITION: Vec3 = Vec3::new(-0.6, 0.0, -0.44);
    const RIGHT_POSITION: Vec3 = Vec3::new(0.6, 0.0, -0.44);

    fn other(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

impl From<BulletSide> for Vec3 {
    fn from(value: BulletSide) -> Self {
        match value {
            BulletSide::Left => BulletSide::LEFT_POSITION,
            BulletSide::Right => BulletSide::RIGHT_POSITION,
        }
    }
}

#[derive(Component)]
pub struct Spaceship;

#[derive(Bundle)]
pub struct SpaceshipBundle {
    pub velocity_collider_bundle: VelocityColliderBundle,
    pub gravity_affected: GravityAffected,
    pub outline_bundle: OutlineBundle,
    pub scene_bundle: SceneBundle,
    pub spaceship: Spaceship,
    pub collision_groups: CollisionGroups,
}

impl SpaceshipBundle {
    const COLLISION_GROUPS: CollisionGroups =
        CollisionGroups::new(BULLET_COLLISION_GROUP, Group::ALL);

    fn new(model: Handle<Scene>, pos: Vec3) -> Self {
        Self {
            velocity_collider_bundle: VelocityColliderBundle {
                collider: Collider::ball(1.0),
                velocity: Velocity {
                    linvel: Vec3::X,
                    ..default()
                },
                ..default()
            },
            gravity_affected: GravityAffected,
            outline_bundle: OutlineBundle {
                outline: OutlineVolume {
                    visible: true,
                    colour: Color::BLACK,
                    width: 3.0,
                },
                ..default()
            },
            scene_bundle: SceneBundle {
                scene: model,
                transform: Transform::from_translation(pos),
                inherited_visibility: InheritedVisibility::VISIBLE,
                ..default()
            },
            spaceship: Spaceship,
            collision_groups: Self::COLLISION_GROUPS,
        }
    }
}

fn spaceship_collisions(
    mut spaceship: Query<
        (
            &mut Velocity,
            &mut Transform,
            &CollidingEntities,
            Entity,
            Option<&mut Health>,
        ),
        With<Spaceship>,
    >,
    planet_query: Query<(&Transform, &Planet), Without<Spaceship>>,
    bullet_query: Query<&Bullet, Without<Spaceship>>,
    mut explosions: EventWriter<ExplosionEvent>,
) {
    for (
        mut velocity, 
        mut transform, 
        colliding_entities, 
        entity, 
        mut health
    ) in &mut spaceship {
        
        if let Some((planet_transform, planet)) = colliding_entities
            .iter()
            .map(|e| planet_query.get(e))
            .find(Result::is_ok)
            .map(Result::unwrap)
        {
            explosions.send(ExplosionEvent {
                parent: Some(entity),
                ..default()
            });

            let normal = (transform.translation - planet_transform.translation).normalize();
            velocity.linvel = -30.0 * normal.dot(velocity.linvel.normalize()) * normal;
            transform.translation = planet_transform.translation + normal * (planet.radius + 1.0);

            if let Some(ref mut health) = health {
                health.take_damage(5.0);
            }
        }

        if let Some(bullet) = colliding_entities
            .iter()
            .map(|e| bullet_query.get(e))
            .find(Result::is_ok)
            .map(Result::unwrap)
        {
            if bullet.origin == entity {
                continue;
            }

            if let Some(ref mut health) = health {
                health.take_damage(10.0);
            }

            explosions.send(ExplosionEvent {
                parent: Some(entity),
                ..default()
            });
            // TODO: Add bullet damage
        }
    }
}

#[derive(AssetCollection, Resource)]
struct SpaceshipAssets {
    #[asset(path = "spaceship.glb#Scene0")]
    player_ship: Handle<Scene>,
    #[asset(path = "enemy.glb#Scene0")]
    enemy_ship: Handle<Scene>,
}

#[derive(Component)]
struct SpaceshipExhaustParticle;

#[derive(Event)]
struct ParticleSpawnEvent {
    entity: Entity,
}

fn spawn_exhaust_particle(
    mut events: EventReader<ParticleSpawnEvent>,
    mut commands: Commands,
    res: Res<FireParticleRes>,
    time: Res<Time>,
    space_ship_query: Query<(&Transform, &Velocity), With<Spaceship>>,
) {
    let mut rng = rand::thread_rng();
    const RANDOM_VEL_RANGE: std::ops::Range<f32> = -4.0..4.0;
    const LIFE_TIME_RANGE: std::ops::Range<u64> = 300..500;

    for event in events.read() {
        let Ok((transform, velocity)) = space_ship_query.get(event.entity) else {
            continue;
        };
        let scale = Vec3::splat(rng.gen_range(0.7..1.4));
        let lifetime = rng.gen_range(LIFE_TIME_RANGE);
        let linvel = velocity.linvel -
            transform.forward() * 10.0 + // Speed relative to spaceship
            transform.forward().cross(Vec3::Y).normalize() * rng.gen_range(RANDOM_VEL_RANGE); // Random sideways velocity

        commands.spawn((
            PbrBundle {
                material: res.materials.choose(&mut rng).unwrap().clone(),
                mesh: res.mesh.clone(),
                transform: Transform {
                    translation: transform.translation - transform.forward() * 0.4,
                    scale,
                    rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                },
                ..default()
            },
            SpaceshipExhaustParticle,
            Velocity {
                linvel,
                ..default()
            },
            RigidBody::KinematicVelocityBased,
            DespawnAfter {
                time: Duration::from_millis(lifetime),
                spawn_time: time.elapsed(),
            },
        ));
    }
}

fn exhaust_particle_update(
    time: Res<Time>,
    mut particles: Query<&mut Transform, With<SpaceshipExhaustParticle>>,
) {
    for mut transform in &mut particles {
        transform.scale += Vec3::splat(1.0) * time.delta_seconds();
    }
}

pub struct SpaceshipPlugin;

impl Plugin for SpaceshipPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, SpaceshipAssets>(AppState::MainSceneLoading)
            .add_plugins((bot::BotPlugin, player::PlayerPlugin))
            .add_event::<ParticleSpawnEvent>()
            .add_systems(
                Update,
                (
                    spawn_exhaust_particle,
                    exhaust_particle_update,
                    spaceship_collisions.in_set(Set::ExplosionEvents),
                )
                    .run_if(in_state(AppState::MainScene)),
            );
    }
}
