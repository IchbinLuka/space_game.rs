use bevy::prelude::*;
use bevy_mod_outline::{OutlineBundle, OutlineVolume};
use bevy_rapier3d::prelude::*;

use crate::components::colliders::VelocityColliderBundle;

#[derive(Component)]
pub struct Asteroid;


#[derive(Event)]
pub struct AsteroidSpawnEvent {
    pub position: Vec3,
    pub velocity: Vec3,
    pub size: f32,
}


#[derive(Resource)]
struct AsteroidRes {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

fn asteroid_setup(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(shape::UVSphere {
        radius: 1.0,
        sectors: 10,
        ..default()
    }.into());

    let material = materials.add(StandardMaterial {
        base_color: Color::DARK_GRAY, 
        ..default()
    });

    commands.insert_resource(AsteroidRes {
        mesh,
        material,
    });
}

fn asteroid_collisions(
    mut commands: Commands, 
    query: Query<(Entity, &CollidingEntities), With<Asteroid>>
) {
    for (entity, colliding) in &query {
        if !colliding.is_empty() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn asteroid_spawn(
    mut commands: Commands,
    mut spawn_events: EventReader<AsteroidSpawnEvent>, 
    asteroid_res: Res<AsteroidRes>,
) {
    for event in spawn_events.read() {
        commands.spawn((
            PbrBundle {
                mesh: asteroid_res.mesh.clone(),
                material: asteroid_res.material.clone(),
                transform: Transform::from_translation(event.position),
                ..default()
            }, 
            Asteroid,
            VelocityColliderBundle {
                velocity: Velocity {
                    linvel: event.velocity,
                    ..default()
                },
                collider: Collider::ball(1.0), 
                ..default()
            }, 
            OutlineBundle {
                outline: OutlineVolume {
                    visible: true, 
                    width: 1.0,
                    colour: Color::BLACK, 
                    ..default()
                }, 
                ..default()
            }
        ));
    }
}



pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, asteroid_setup)
            .add_systems(Update, (asteroid_spawn, asteroid_collisions))
            .add_event::<AsteroidSpawnEvent>();
    }
}
