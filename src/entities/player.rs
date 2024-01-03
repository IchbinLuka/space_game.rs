use std::time::Duration;

use bevy::{prelude::*, scene::SceneInstance, render::render_resource::PrimitiveTopology};
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_mod_outline::{OutlineBundle, OutlineVolume, InheritOutlineBundle};
use bevy_rapier3d::prelude::*;
use rand::{seq::SliceRandom, Rng};

use crate::{AppState, components::{despawn_after::DespawnAfter, gravity::{GravitySource, GravityAffected, gravity_step}, movement::MaxSpeed, colliders::VelocityColliderBundle}, particles::fire_particles::FireParticleRes, utils::sets::Set};

use super::{planet::Planet, explosion::ExplosionEvent, bullet::{Bullet, BULLET_COLLISION_GROUP, BulletSpawnEvent}};

type IsPlayer = (With<Player>, Without<Bot>);
type IsBot = (With<Bot>, Without<Player>);

#[derive(Component)]
pub struct Player;


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
pub struct Bot {
    state: BotState,
}

enum BotState {
    Chasing, 
    Fleeing,
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
    const COLLISION_GROUPS: CollisionGroups = CollisionGroups::new(BULLET_COLLISION_GROUP, Group::ALL);

    fn new(assets: &PlayerAssets, pos: Vec3) -> Self {
        Self {
            velocity_collider_bundle: VelocityColliderBundle {
                collider: Collider::ball(3.0),
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
                scene: assets.spaceship.clone(),
                transform: Transform::from_translation(pos).with_scale(Vec3::splat(0.2)), 
                inherited_visibility: InheritedVisibility::VISIBLE,
                ..default()
            }, 
            spaceship: Spaceship,
            collision_groups: Self::COLLISION_GROUPS,
        }
    }
}


fn player_input(
    timer: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Transform, Entity), IsPlayer>,
    mut particle_spawn: EventWriter<ParticleSpawnEvent>
) {
    for (mut velocity, mut transform, entity) in &mut query {
        for key in keyboard_input.get_pressed() {
            
            match key {
                KeyCode::Up | KeyCode::W => {
                    velocity.linvel += transform.forward().normalize();
                    particle_spawn.send(ParticleSpawnEvent {
                        entity
                    });
                },
                KeyCode::Left | KeyCode::A => transform.rotate_y(5.0 * timer.delta_seconds()),
                KeyCode::Right | KeyCode::D => transform.rotate_y(-5.0 * timer.delta_seconds()),
                _ => (),
            }
        }
    }
}

fn player_shoot(
    keyboard_input: Res<Input<KeyCode>>, 
    time: Res<Time>,
    query: Query<(&Transform, &Velocity, With<Player>)>, 
    mut bullet_spawn_events: EventWriter<BulletSpawnEvent>,
    mut last_bullet_info: Local<LastBulletInfo>,
) {
    if !last_bullet_info.timer.finished() {
        last_bullet_info.timer.tick(time.delta());
        return;
    }
    
    for (transform, velocity, _) in &query {
        if keyboard_input.pressed(KeyCode::Space) {
            // If finished, the timer should wait for the player to shoot before ticking again 
            last_bullet_info.timer.tick(time.delta());
            let side = last_bullet_info.side;

            let pos = transform.translation + transform.rotation.mul_vec3(side.into());
            let mut bullet_transform = Transform::from_translation(pos);
            
            bullet_transform.rotate(transform.rotation);
            debug!("Spawning bullet");
            bullet_spawn_events.send(BulletSpawnEvent {
                position: bullet_transform, 
                entity_velocity: *velocity, 
                direction: transform.forward(),
            });
            
            last_bullet_info.side = side.other();
        }
    }
}

fn bot_update(
    mut bots: Query<(&mut Velocity, &mut Transform, &mut Bot, Entity, &mut LastBulletInfo), IsBot>, 
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
        let Ok(player_transform) = player_query.get_single() else { continue };
        
        let delta = player_transform.translation - transform.translation;
        let distance = delta.length();

        
        match bot.state {
            BotState::Chasing => {
                let angle = transform.forward().angle_between(delta);

                let cross = transform.forward().cross(delta);
                let mut sign = if cross.y > 0.0 { 1.0 } else { -1.0 };

                if distance < 20.0 {
                    sign *= -1.0;
                }

                // velocity.angvel = Vec3::Y * sign * 5.0;
                transform.rotate_y(sign * 5.0 * time.delta_seconds());

                if angle < 0.1 || delta.length() < 20.0 {
                    velocity.linvel += transform.forward().normalize();
                    exhaust_particles.send(ParticleSpawnEvent {
                        entity
                    });
                }

                if last_bullet.timer.finished() && 
                       angle < 0.1 &&  // Angle should be small
                       distance < 50.0 // Enemy should only shoot when close
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
                    });
                    
                    last_bullet.side = side.other();
                    last_bullet.timer.tick(time.delta());
                }

                if rng.gen_bool(0.001) {
                    bot.state = BotState::Fleeing;
                }
            }, 
            BotState::Fleeing => {
                let angle = delta.angle_between(-transform.forward());
                if angle > 0.1 {
                    let cross = transform.forward().cross(delta);
                    let sign = if cross.y > 0.0 { 1.0 } else { -1.0 };
                    // velocity.angvel = Vec3::Y * sign * 5.0;
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

fn player_setup(
    mut commands: Commands,
    assets: Res<PlayerAssets>,
) {

    commands.spawn((
        Player, 
        SpaceshipBundle::new(&assets, Vec3::ZERO), 
        MaxSpeed {
            max_speed: 60.0,
        }
    ));

    commands.spawn((
        Bot {
            state: BotState::Chasing,
        }, 
        LastBulletInfo::default(),
        SpaceshipBundle::new(&assets, Vec3::new(0.0, 0.0, 10.0)), 
        MaxSpeed {
            max_speed: 30.0,
        }
    ));
}


fn setup_scene_once_loaded(
    mut commands: Commands,
    scene_query: Query<&SceneInstance>,
    scene_manager: Res<SceneSpawner>,
    mut done: Local<bool>,
) {
    if !*done {
        if let Ok(scene) = scene_query.get_single() {
            if scene_manager.instance_is_ready(**scene) {
                for entity in scene_manager.iter_instance_entities(**scene) {
                    commands
                        .entity(entity)
                        .insert(InheritOutlineBundle::default());
                }
                *done = true;
            }
        }
    }
}

fn spaceship_collisions(
    mut spaceship: Query<(&mut Velocity, &mut Transform, &CollidingEntities, Entity), With<Spaceship>>,
    planet_query: Query<(&Transform, &Planet), Without<Spaceship>>, 
    bullet_query: Query<(), (Without<Spaceship>, With<Bullet>)>,
    mut explosions: EventWriter<ExplosionEvent>,
) {
    for (mut velocity, mut transform, colliding_entities, entity) in &mut spaceship {
        if let Some((planet_transform, planet)) = colliding_entities
            .iter()
            .map(|e| planet_query.get(e))
            .find(Result::is_ok).map(Result::unwrap) 
        {

            explosions.send(ExplosionEvent {
                parent: Some(entity),
                ..default()
            });
    
            let normal = (transform.translation - planet_transform.translation).normalize();
    
            velocity.linvel = -30.0 * normal.dot(velocity.linvel.normalize()) * normal;
            transform.translation = planet_transform.translation + normal * (planet.radius + 0.5);
        }

        if colliding_entities
            .iter()
            .map(|e| bullet_query.get(e))
            .find(Result::is_ok).map(Result::unwrap).is_some() 
        {
            explosions.send(ExplosionEvent {
                parent: Some(entity),
                ..default()
            });
            // TODO: Add bullet damage
        }
    }

}



#[derive(AssetCollection, Resource)]
struct PlayerAssets {
    #[asset(path = "spaceship.glb#Scene0")]
    spaceship: Handle<Scene>
}


#[derive(Component)]
struct SpaceshipExhaustParticle;

#[derive(Event)]
struct ParticleSpawnEvent {
    entity: Entity
}



fn spawn_exhaust_particle(
    mut events: EventReader<ParticleSpawnEvent>, 
    mut commands: Commands, 
    res: Res<FireParticleRes>, 
    time: Res<Time>, 
    space_ship_query: Query<(&Transform, &Velocity), With<Spaceship>>
) {
    let mut rng = rand::thread_rng();
    const RANDOM_VEL_RANGE: std::ops::Range<f32> = -4.0..4.0;
    const LIFE_TIME_RANGE: std::ops::Range<u64> = 300..500;
    
    for event in events.read() {
        let Ok((transform, velocity)) = space_ship_query.get(event.entity) else { continue };
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
                spawn_time: time.elapsed()
            }
        ));
    }
}

fn exhaust_particle_update(
    time: Res<Time>, 
    mut particles: Query<&mut Transform, With<SpaceshipExhaustParticle>>
) {
    for mut transform in &mut particles {
        transform.scale += Vec3::splat(1.0) * time.delta_seconds();
    }
}

const PREDICTION_LENGTH: usize = 100;
const LINE_THICKNESS: f32 = 0.1;


#[derive(Component)]
struct PlayerLine;

fn player_line_setup(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>, 
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: Color::WHITE,
        double_sided: true,
        ..default()
    });

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleStrip);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, [Vec3::ZERO; PREDICTION_LENGTH * 2].to_vec());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, [Vec3::Y; PREDICTION_LENGTH * 2].to_vec());
    let mesh_handle = meshes.add(mesh);


    commands.spawn((
        PbrBundle {
            mesh: mesh_handle, 
            material, 
            ..default()
        }, 
        PlayerLine
    ));
}

fn player_line_update(
    mut line_query: Query<(&mut Handle<Mesh>, &mut Transform), (With<PlayerLine>, Without<Player>)>, 
    player_query: Query<(&Transform, &Velocity), IsPlayer>,
    gravity_sources: Query<(&Transform, &GravitySource), (Without<Player>, Without<PlayerLine>)>,
    mut assets: ResMut<Assets<Mesh>>,
) {

    for (mesh_handle, mut transform) in &mut line_query {
        let Some(mesh) = assets.get_mut(mesh_handle.id()) else { continue };
        for (player_transform, player_velocity) in &player_query {
            // let mut position_attribute = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();
            transform.translation = player_transform.translation - Vec3::Y * 0.1;

            let mut positions: Vec<Vec3> = Vec::with_capacity(PREDICTION_LENGTH * 2);
            let player_pos = player_transform.translation;
            let mut current_pos = Vec3::ZERO;
            let mut current_vel = player_velocity.linvel;

            for i in 0..PREDICTION_LENGTH {
                let perpendicular = current_vel.cross(Vec3::Y).normalize();
                let thickness = (1.0 - (i as f32 / PREDICTION_LENGTH as f32).powf(2.0)) * LINE_THICKNESS;

                current_pos += current_vel * 0.02;

                current_vel += gravity_sources.iter().map(|(gravity_transform, gravity_source)| {
                    gravity_step(gravity_transform, gravity_source, 0.02, current_pos + player_pos, current_vel)
                }).sum::<Vec3>();

                positions.push(current_pos - perpendicular * thickness);
                positions.push(current_pos + perpendicular * thickness);
            }
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        }
    }
}


pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_collection_to_loading_state::<_, PlayerAssets>(AppState::Loading)
            .add_event::<ParticleSpawnEvent>()
            .add_systems(OnEnter(AppState::Running), (
                player_setup,  
                player_line_setup
            ))
            .add_systems(Update, (
                player_input, 
                player_shoot.in_set(Set::BulletEvents),
                setup_scene_once_loaded, 
                spawn_exhaust_particle, 
                exhaust_particle_update,
                player_line_update, 
                spaceship_collisions.in_set(Set::ExplosionEvents), 
                bot_update, 
            ).run_if(in_state(AppState::Running)));
    }
}