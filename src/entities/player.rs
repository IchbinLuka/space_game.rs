use std::time::Duration;

use bevy::{prelude::*, scene::SceneInstance, render::render_resource::PrimitiveTopology};
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_mod_outline::{OutlineBundle, OutlineVolume, InheritOutlineBundle};
use bevy_rapier3d::prelude::*;
use rand::{seq::SliceRandom, Rng};

use crate::{AppState, components::{despawn_after::DespawnAfter, gravity::{GravitySource, GravityAffected}, movement::MaxSpeed}};

#[derive(Component)]
pub struct Player;

fn player_input(
    timer: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Transform, With<Player>)>,
    mut particle_spawn: EventWriter<ParticleSpawnEvent>
) {
    let mut rng = rand::thread_rng();
    for (mut velocity, mut transform, _) in &mut query {
        for key in keyboard_input.get_pressed() {
            
            match key {
                KeyCode::Up | KeyCode::W => {
                    let particle_offset: Vec3 = Vec3::new(rng.gen_range(-0.2..0.2), 0.0, 0.8);
                    velocity.linvel += transform.forward().normalize();
                    particle_spawn.send(ParticleSpawnEvent {
                        pos: transform.translation + transform.rotation.mul_vec3(particle_offset),
                        vel: -transform.forward().normalize(),
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
        Velocity {
            linvel: Vec3::X, 
            ..default()
        }, 
        RigidBody::KinematicVelocityBased, 
        Collider::ball(1.0),
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



#[derive(AssetCollection, Resource)]
struct PlayerAssets {
    #[asset(path = "spaceship.glb#Scene0")]
    spaceship: Handle<Scene>
}


#[derive(Component)]
struct SpaceshipExhaustParticle;

#[derive(Event)]
struct ParticleSpawnEvent {
    pos: Vec3, 
    vel: Vec3,
}

#[derive(Resource)]
struct ExhausParticleRes {
    mesh: Handle<Mesh>,
    materials: [Handle<StandardMaterial>; 4],
}

fn spawn_exhaust_particle(
    mut events: EventReader<ParticleSpawnEvent>, 
    mut commands: Commands, 
    res: Res<ExhausParticleRes>, 
    time: Res<Time>
) {
    let mut rng = rand::thread_rng();
    const RANDOM_VEL_RANGE: std::ops::Range<f32> = -0.7..0.7;
    const RANDOM_ANG_RANGE: std::ops::Range<f32> = -0.7..0.7;
    for event in events.read() {
        commands.spawn((
            PbrBundle {
                material: res.materials.choose(&mut rng).unwrap().clone(), 
                mesh: res.mesh.clone(),
                transform: Transform::from_translation(event.pos).with_scale(
                    Vec3::splat(rng.gen_range(0.7..1.4))
                ),
                ..default()
            }, 
            SpaceshipExhaustParticle,
            Velocity {
                linvel: event.vel + Vec3::new(
                    rng.gen_range(RANDOM_VEL_RANGE.clone()), 
                    rng.gen_range(RANDOM_VEL_RANGE.clone()), 
                    rng.gen_range(RANDOM_VEL_RANGE.clone())
                ), 
                angvel: Vec3::new(
                    rng.gen_range(RANDOM_ANG_RANGE.clone()), 
                    rng.gen_range(RANDOM_ANG_RANGE.clone()), 
                    rng.gen_range(RANDOM_ANG_RANGE.clone())
                )
            }, 
            RigidBody::KinematicVelocityBased, 
            DespawnAfter {
                time: Duration::from_millis(500), 
                spawn_time: time.elapsed()
            }
        ));
    }
}

fn setup_exhaust_particles(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(shape::Cube::new(0.2).into());
    
    let colors = [
        Color::hex("ef8904").unwrap(), 
        Color::hex("f2600c").unwrap(), 
        Color::hex("cc2804").unwrap(), 
        Color::hex("e89404").unwrap(), 
    ];

    let materials = colors.iter().map(|color| {
        materials.add(StandardMaterial {
            emissive: *color, 
            base_color: *color,
            ..default()
        })
    }).collect::<Vec<_>>().try_into().unwrap();

    commands.insert_resource(
        ExhausParticleRes {
            mesh, 
            materials
        }
    )
}

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

    let mesh = meshes.add(Mesh::new(PrimitiveTopology::TriangleStrip));


    commands.spawn((
        PbrBundle {
            mesh, 
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
    const PREDICTION_LENGTH: usize = 20;
    const LINE_THICKNESS: f32 = 0.1;
    const NORMALS: [Vec3; PREDICTION_LENGTH * 2] = [Vec3::Y; PREDICTION_LENGTH * 2];

    for (mesh_handle, mut transform) in &mut line_query {
        let Some(mesh) = assets.get_mut(mesh_handle.id()) else { continue };
        for (player_transform, player_velocity) in &player_query {
            // let mut position_attribute = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();
            transform.translation = player_transform.translation;

            let mut positions: Vec<Vec3> = Vec::with_capacity(PREDICTION_LENGTH * 2);

            let mut current_pos = Vec3::ZERO;
            let mut current_vel = player_velocity.linvel;

            for i in 0..PREDICTION_LENGTH {
                let perpendicular = current_vel.cross(Vec3::Y).normalize();
                let thickness = (1.0 - (i as f32 / PREDICTION_LENGTH as f32).powf(2.0)) * LINE_THICKNESS;
                positions.push(current_pos - perpendicular * thickness);
                positions.push(current_pos + perpendicular * thickness);

                current_pos += current_vel * 0.1;
                // TODO: Duplicate code
                current_vel += gravity_sources.iter().map(|(gravity_transform, gravity_source)| {
                    let gravity_dir = (gravity_transform.translation - current_pos).normalize();
                    let gravity_strength = gravity_source.mass / (gravity_transform.translation - current_pos).length_squared();
                    gravity_dir * gravity_strength * 0.1
                }).sum::<Vec3>();
            }
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, NORMALS.to_vec());
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
                setup_exhaust_particles, 
                player_line_setup
            ))
            .add_systems(Update, (
                player_input, 
                setup_scene_once_loaded, 
                spawn_exhaust_particle, 
                player_line_update
            ).run_if(in_state(AppState::Running)));
    }
}