use bevy::animation::RepeatAnimation;
use bevy::prelude::*;
use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_mod_outline::{OutlineBundle, OutlineVolume};
use bevy_rapier3d::prelude::*;
use rand::Rng;

use crate::components::health::Regeneration;
use crate::entities::spaceship::player::LastHit;
use crate::materials::toon::replace_with_toon_materials;
use crate::states::DespawnOnCleanup;
use crate::ui::game_over::GameOverEvent;
use crate::ui::minimap::{MinimapAssets, ShowOnMinimap};
use crate::utils::asset_loading::AppExtension;
use crate::utils::materials::default_outline;
use crate::utils::scene::{AnimationRoot, ReplaceMaterialPlugin};
use crate::{
    components::health::Health,
    materials::toon::ToonMaterial,
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

pub fn setup_space_station(
    mut commands: Commands,
    res: Res<SpaceStationRes>,
    minimap_res: Res<MinimapAssets>,
) {
    let mut rng = rand::thread_rng();

    spawn_space_station(
        &mut commands,
        &res,
        &minimap_res,
        Vec3::new(rng.gen_range(-50.0..50.0), 0., rng.gen_range(-50.0..50.0)),
        true,
    );
}

pub fn spawn_space_station(
    commands: &mut Commands,
    res: &SpaceStationRes,
    minimap_res: &MinimapAssets,
    position: Vec3,
    with_health_bar: bool,
) {
    let space_station = commands
        .spawn((
            SceneBundle {
                scene: res.model.clone(),
                transform: Transform::from_translation(position),
                ..default()
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
            ShowOnMinimap {
                sprite: minimap_res.space_station_indicator.clone(),
                size: 0.1.into(),
            },
            EnemyTarget,
            Regeneration {
                heal_cooldown: 5.0,
                regen_speed: 10.0,
            },
            LastHit::default(),
            ActiveCollisionTypes::DYNAMIC_STATIC | ActiveCollisionTypes::KINEMATIC_STATIC,
            Health::new(200.),
            (
                DespawnOnCleanup,
                OutlineBundle {
                    outline: OutlineVolume {
                        width: 1.0,
                        ..default_outline()
                    },
                    ..default()
                },
            ),
        ))
        .id();

    if with_health_bar {
        commands.add(SpawnHealthBar {
            entity: space_station,
            shield_entity: None,
            scale: 0.5,
            offset: Vec2::new(0., 20.),
        });
    }
}

fn space_station_animation(
    space_station: Query<&AnimationRoot, (With<SpaceStation>, Added<AnimationRoot>)>,
    mut animation_players: Query<&mut AnimationPlayer>,
    space_station_res: Res<SpaceStationRes>,
) {
    for animation_root in &space_station {
        for entity in &animation_root.player_entites {
            let Ok(mut player) = animation_players.get_mut(*entity) else {
                continue;
            };
            player.play(space_station_res.animation.clone());
            player.set_repeat(RepeatAnimation::Forever);
        }
    }
}

fn space_station_death(
    space_stations: Query<(&Health, &Transform, Entity), (With<SpaceStation>, Changed<Health>)>,
    mut commands: Commands,
    mut explosion_events: EventWriter<ExplosionEvent>,
    mut game_over_events: EventWriter<GameOverEvent>,
) {
    let mut count = space_stations.iter().count();

    for (health, transform, entity) in &space_stations {
        if health.is_dead() {
            explosion_events.send(ExplosionEvent {
                position: transform.translation,
                parent: None,
                radius: 10.,
            });
            commands.entity(entity).despawn_recursive();
            count -= 1;
            if count == 0 {
                game_over_events.send(GameOverEvent);
            }
        }
    }
}

#[derive(AssetCollection, Resource, Debug)]
pub struct SpaceStationRes {
    #[asset(path = "space_station.glb#Scene0")]
    model: Handle<Scene>,
    #[asset(path = "space_station.glb#Animation0")]
    animation: Handle<AnimationClip>,
}

pub struct SpaceStationPlugin;

impl Plugin for SpaceStationPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_states::<SpaceStationRes>(&[
            AppState::MainSceneLoading,
            AppState::StartScreenLoading,
        ])
        .add_plugins(ReplaceMaterialPlugin::<SpaceStation, _>::new(
            replace_with_toon_materials(ToonMaterial {
                filter_scale: 1.0,
                ..default()
            }),
        ))
        .add_systems(ON_GAME_STARTED, (setup_space_station,))
        .add_systems(
            Update,
            (
                space_station_death.run_if(game_running()),
                space_station_animation
                    .run_if(game_running().or_else(in_state(AppState::StartScreen))),
            ),
        );
    }
}
