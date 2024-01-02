use std::time::Duration;

use bevy::{prelude::*, scene::SceneInstance, render::render_resource::PrimitiveTopology};
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_mod_outline::{OutlineBundle, OutlineVolume, InheritOutlineBundle};
use bevy_rapier3d::prelude::*;
use rand::{seq::SliceRandom, Rng};

use crate::{AppState, components::{despawn_after::DespawnAfter, gravity::{GravitySource, GravityAffected, gravity_step}, movement::MaxSpeed, colliders::VelocityColliderBundle}, particles::fire_particles::FireParticleRes};

use super::{planet::Planet, explosion::ExplosionEvent};

#[derive(Component)]
pub struct Player;

fn player_input(
    timer: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Transform, Entity), With<Player>>,
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

fn player_setup(
    mut commands: Commands,
    assets: Res<PlayerAssets>,
) {

    commands.spawn((
        SceneBundle {
            scene: assets.spaceship.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(0.2)), 
            ..default()
        }, 
        Player, 
        MaxSpeed {
            max_speed: 100.0,
        },
        VelocityColliderBundle {
            collider: Collider::ball(1.0),
            velocity: Velocity {
                linvel: Vec3::X, 
                ..default()
            }, 
            ..default()
        }, 
        GravityAffected, 
        OutlineBundle {
            outline: OutlineVolume {
                visible: true,
                colour: Color::BLACK, 
                width: 3.0,
            }, 
            ..default()
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

fn player_collisions(
    mut player: Query<(&mut Velocity, &mut Transform, &CollidingEntities), With<Player>>,
    planet_query: Query<(&Transform, &Planet), Without<Player>>, 
    mut explosions: EventWriter<ExplosionEvent>,
) {
    for (mut velocity, mut transform, colliding_entities) in &mut player {
        let Some((planet_transform, planet)) = colliding_entities
            .iter()
            .map(|e| planet_query.get(e))
            .find(Result::is_ok).map(Result::unwrap)
        else { continue };

        explosions.send(ExplosionEvent {
            position: transform.translation,
        });

        let normal = (transform.translation - planet_transform.translation).normalize();

        velocity.linvel = -30.0 * normal.dot(velocity.linvel.normalize()) * normal;
        transform.translation = planet_transform.translation + normal * (planet.radius + 0.5);
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
    space_ship_query: Query<(&Transform, &Velocity), With<Player>>
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
    player_query: Query<(&Transform, &Velocity), With<Player>>,
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
                setup_scene_once_loaded, 
                spawn_exhaust_particle, 
                exhaust_particle_update,
                player_line_update, 
                player_collisions, 
            ).run_if(in_state(AppState::Running)));
    }
}