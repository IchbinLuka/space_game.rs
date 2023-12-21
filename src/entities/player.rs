use std::time::Duration;

use bevy::{prelude::*, scene::SceneInstance};
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_mod_outline::{OutlineBundle, OutlineVolume, InheritOutlineBundle};
use rand::{seq::SliceRandom, Rng};

use crate::{Movement, AppState, components::despawn_after::DespawnAfter};

#[derive(Component)]
pub struct Player;

fn player_input(
    timer: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Movement, &mut Transform, With<Player>)>,
    mut particle_spawn: EventWriter<ParticleSpawnEvent>
) {
    let mut rng = rand::thread_rng();
    for (mut movement, mut transform, _) in &mut query {
        for key in keyboard_input.get_pressed() {
            
            match key {
                KeyCode::Up | KeyCode::W => {
                    let particle_offset: Vec3 = Vec3::new(rng.gen_range(-0.2..0.2), 0.0, 0.8);
                    movement.vel += transform.forward().normalize();
                    particle_spawn.send(ParticleSpawnEvent {
                        pos: transform.translation + transform.rotation.mul_vec3(particle_offset),
                        vel: -transform.forward().normalize(),
                    });
                },
                KeyCode::Left | KeyCode::A => transform.rotate_y(3.0 * timer.delta_seconds()),
                KeyCode::Right | KeyCode::D => transform.rotate_y(-3.0 * timer.delta_seconds()),
                _ => (),
            }
        }
        if movement.vel.length() > 10.0 {
            movement.vel = movement.vel.normalize() * 10.0;
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
        Movement {
            max_speed: Some(10.0),
            friction: 0.3,
            ..default()
        },
        OutlineBundle {
            outline: OutlineVolume {
                visible: true,
                colour: Color::BLACK, 
                width: 3.0,
                ..default()
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
            Movement {
                vel: event.vel + Vec3::new(
                    rng.gen_range(RANDOM_VEL_RANGE.clone()), 
                    rng.gen_range(RANDOM_VEL_RANGE.clone()), 
                    rng.gen_range(RANDOM_VEL_RANGE.clone())
                ),
                ..default()
            }, 
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
        Color::hex("f79b22").unwrap(), 
        Color::hex("f78442").unwrap(), 
        Color::hex("cc4122").unwrap(), 
        Color::hex("e5a029").unwrap(), 
    ];

    let materials = colors.iter().map(|color| {
        materials.add(StandardMaterial {
            emissive: *color, 
            ..default()
        })
    }).collect::<Vec<_>>().try_into().unwrap();

    commands.insert_resource(
        ExhausParticleRes {
            mesh: mesh, 
            materials: materials
        }
    )
}


pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_collection_to_loading_state::<_, PlayerAssets>(AppState::Loading)
            .add_event::<ParticleSpawnEvent>()
            .add_systems(OnEnter(AppState::Running), (
                player_setup, 
                setup_exhaust_particles
            ))
            .add_systems(Update, (
                player_input, 
                setup_scene_once_loaded, 
                spawn_exhaust_particle
            ).run_if(in_state(AppState::Running)));
    }
}