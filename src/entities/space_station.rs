use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::{
    components::health::Health,
    materials::toon::{ApplyToonMaterial, ToonMaterial},
    states::{game_running, AppState, ON_GAME_STARTED},
    ui::health_bar_3d::SpawnHealthBar,
};

use super::{
    bullet::{BulletTarget, BulletType},
    explosion::ExplosionEvent,
    spaceship::{bot::EnemyTarget, SpaceshipCollisions},
};

#[derive(Component)]
pub struct SpaceStation;

pub fn setup_space_station(mut commands: Commands, res: Res<SpaceStationRes>) {
    let mut rng = rand::thread_rng();
    let space_station = commands
        .spawn((
            SceneBundle {
                scene: res.model.clone(),
                transform: Transform::from_translation(
                    Vec3::new(rng.gen_range(-50.0..50.0), 0., rng.gen_range(-50.0..50.0))),
                ..default()
            },
            ApplyToonMaterial {
                base_material: ToonMaterial {
                    filter_scale: 1.,
                    ..default()
                },
            },
            SpaceStation,
            RigidBody::Fixed,
            Collider::cylinder(5., 5.25),
            CollisionGroups::new(Group::all(), Group::all()), // TODO,
            BulletTarget {
                target_type: BulletType::Bot,
                bullet_damage: Some(10.0),
            },
            SpaceshipCollisions {
                collision_damage: 5.,
                bound_radius: 5.,
            },
            EnemyTarget,
            ActiveCollisionTypes::DYNAMIC_STATIC | ActiveCollisionTypes::KINEMATIC_STATIC,
            Health::new(200.),
        ))
        .id();

    commands.add(SpawnHealthBar {
        entity: space_station,
        shield_entity: None,
        scale: 0.5,
        offset: Vec2::new(0., 20.),
    });
}

fn space_station_death(
    space_stations: Query<(&Health, &Transform, Entity), (With<SpaceStation>, Changed<Health>)>,
    mut commands: Commands,
    mut explosion_events: EventWriter<ExplosionEvent>,
) {
    for (health, transform, entity) in &space_stations {
        if health.is_dead() {
            explosion_events.send(ExplosionEvent {
                position: transform.translation,
                parent: None,
                radius: 10.,
            });
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[derive(AssetCollection, Resource, Debug)]
pub struct SpaceStationRes {
    #[asset(path = "space_station.glb#Scene0")]
    model: Handle<Scene>,
}

pub struct SpaceStationPlugin;

impl Plugin for SpaceStationPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, SpaceStationRes>(AppState::MainSceneLoading)
            .add_systems(ON_GAME_STARTED, (setup_space_station,))
            .add_systems(Update, (space_station_death,).run_if(game_running()));
    }
}
