use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_mod_outline::{OutlineBundle, OutlineVolume};
use bevy_rapier3d::prelude::*;
use rand::seq::SliceRandom;

use crate::{components::colliders::VelocityColliderBundle, AppState};

#[derive(Component)]
pub struct Asteroid;


#[derive(Event)]
pub struct AsteroidSpawnEvent {
    pub position: Transform,
    pub velocity: Velocity,
    pub size: f32,
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
    asteroid_res: Res<AsteroidAssets>,
) {
    if spawn_events.len() == 0 { return; }
    let mut rng = rand::thread_rng();
    let asteroids = [asteroid_res.asteroid_1.clone(), asteroid_res.asteroid_2.clone()];
    for event in spawn_events.read() {
        let scene = asteroids.choose(&mut rng).unwrap();
        commands.spawn((
            SceneBundle {
                scene: scene.clone(), 
                transform: event.position,
                ..default()
            }, 
            Asteroid,
            VelocityColliderBundle {
                velocity: event.velocity,
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

#[derive(AssetCollection, Resource)]
struct AsteroidAssets {
    #[asset(path = "asteroid1.glb#Scene0")]
    asteroid_1: Handle<Scene>, 
    #[asset(path = "asteroid2.glb#Scene0")]
    asteroid_2: Handle<Scene>,
}

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_collection_to_loading_state::<_, AsteroidAssets>(AppState::Loading)
            .add_systems(Update, (asteroid_spawn, asteroid_collisions).run_if(in_state(AppState::Running)))
            .add_event::<AsteroidSpawnEvent>();
    }
}
