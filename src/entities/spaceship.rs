use std::time::Duration;

use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_mod_outline::{OutlineBundle, OutlineVolume};
use bevy_rapier3d::prelude::*;
use rand::{seq::SliceRandom, Rng};

use crate::states::{AppState, DespawnOnCleanup};
use crate::{
    components::{colliders::VelocityColliderBundle, despawn_after::DespawnTimer, health::Health},
    particles::fire_particles::FireParticleRes,
    states::game_running,
    utils::{collisions::BULLET_COLLISION_GROUP, misc::CollidingEntitiesExtension, sets::Set},
};

use self::{bot::Bot, player::Player};

use super::bullet::{BulletSpawnEvent, BulletType};
use super::explosion::ExplosionEvent;

pub mod bot;
pub mod player;

pub type IsPlayer = (With<Player>, Without<Bot>);
pub type IsBot = (With<Bot>, Without<Player>);

const BULLET_COOLDOWN: f32 = 0.2;

#[derive(Component)]
pub struct SpaceshipCollisions {
    pub collision_damage: f32,
    pub bound_radius: f32,
}

impl Default for SpaceshipCollisions {
    fn default() -> Self {
        Self {
            collision_damage: 10.,
            bound_radius: 0.,
        }
    }
}

#[derive(Resource, Component)]
struct LastBulletInfo {
    side: BulletSide,
    timer: Timer,
}

impl LastBulletInfo {
    fn with_cooldown(seconds: f32) -> Self {
        Self {
            timer: Timer::from_seconds(seconds, TimerMode::Repeating),
            ..default()
        }
    }
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
pub struct Spaceship {
    pub auxiliary_drive: bool,
}

impl Spaceship {
    fn shoot(
        &self,
        last_bullet: &mut LastBulletInfo,
        bullet_spawn_events: &mut EventWriter<BulletSpawnEvent>,
        transform: &Transform,
        velocity: Velocity,
        bullet_type: BulletType,
    ) {
        let side = last_bullet.side;
        let pos = transform.translation + transform.rotation.mul_vec3(side.into());
        let mut bullet_transform = Transform::from_translation(pos);

        bullet_transform.rotate(transform.rotation);
        debug!("Spawning bullet");
        bullet_spawn_events.send(BulletSpawnEvent {
            position: bullet_transform,
            entity_velocity: velocity,
            direction: transform.forward(),
            bullet_type,
        });

        last_bullet.side = side.other();
    }
}

#[derive(Bundle)]
pub struct SpaceshipBundle {
    pub velocity_collider_bundle: VelocityColliderBundle,
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
                collider: Collider::ball(1.2),
                velocity: Velocity {
                    linvel: Vec3::X,
                    ..default()
                },
                ..default()
            },
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
            spaceship: Spaceship {
                auxiliary_drive: true,
            },
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
    planet_query: Query<(&GlobalTransform, &SpaceshipCollisions), Without<Spaceship>>,
    mut explosions: EventWriter<ExplosionEvent>,
) {
    for (mut velocity, mut transform, colliding_entities, entity, mut health) in &mut spaceship {
        for (global_transform, collisions) in
            colliding_entities.filter_fulfills_query(&planet_query)
        {
            explosions.send(ExplosionEvent {
                parent: Some(entity),
                ..default()
            });
            let colliding_transform = global_transform.compute_transform();
            let delta = transform.translation - colliding_transform.translation;
            let distance = delta.length();
            let normal = delta.normalize();
            velocity.linvel = -velocity.linvel.normalize()
                * f32::max(velocity.linvel.length() * 0.5, 20.)
                * normal;
            transform.translation = colliding_transform.translation + normal * (distance + 2.0);

            if let Some(ref mut health) = health {
                health.take_damage(collisions.collision_damage);
            }
        }
    }
}

fn auxiliary_drive(
    mut spaceships: Query<(&Transform, &mut Velocity, &Spaceship, Entity)>,
    time: Res<Time>,
    mut particle_events: EventWriter<ParticleSpawnEvent>,
) {
    let mut rng = rand::thread_rng();

    for (transform, mut velocity, spaceship, entity) in &mut spaceships {
        if !spaceship.auxiliary_drive {
            continue;
        }
        let forward = transform.forward();
        let vel = velocity.linvel;
        let delta = forward * vel.length() - vel;
        velocity.linvel += delta * f32::min(time.delta_seconds() * 4., 1.);

        if rng.gen_bool(f64::min(
            delta.length() as f64 * time.delta_seconds_f64() * 2.,
            1.,
        )) {
            particle_events.send(ParticleSpawnEvent {
                entity,
                direction: Some(-delta.normalize()),
            });
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
    direction: Option<Vec3>,
}

impl ParticleSpawnEvent {
    fn main_exhaust(entity: Entity) -> Self {
        Self {
            entity,
            direction: None,
        }
    }
}

fn spawn_exhaust_particle(
    mut events: EventReader<ParticleSpawnEvent>,
    mut commands: Commands,
    res: Res<FireParticleRes>,
    space_ship_query: Query<(&Transform, &Velocity), With<Spaceship>>,
) {
    let mut rng = rand::thread_rng();
    const RANDOM_VEL_RANGE: std::ops::Range<f32> = -4.0..4.0;
    const LIFE_TIME_RANGE: std::ops::Range<u64> = 200..300;

    for event in events.read() {
        let Ok((transform, velocity)) = space_ship_query.get(event.entity) else {
            continue;
        };
        let scale = Vec3::splat(rng.gen_range(0.7..1.4));
        let lifetime = rng.gen_range(LIFE_TIME_RANGE);

        let direction = event.direction.unwrap_or(-transform.forward());

        let linvel = velocity.linvel +
            direction * 10.0 + // Speed relative to spaceship
            direction.cross(Vec3::Y).normalize() * rng.gen_range(RANDOM_VEL_RANGE); // Random sideways velocity

        commands.spawn((
            MaterialMeshBundle {
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
            DespawnOnCleanup,
            Velocity {
                linvel,
                ..default()
            },
            RigidBody::KinematicVelocityBased,
            DespawnTimer::new(Duration::from_millis(lifetime)),
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
                    auxiliary_drive,
                )
                    .run_if(game_running()),
            );
    }
}
