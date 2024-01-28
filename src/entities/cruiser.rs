use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_mod_outline::OutlineBundle;
use bevy_rapier3d::{dynamics::Velocity, geometry::Collider};
use bevy_rapier3d::prelude::*;

use crate::components::health::{DespawnOnDeath, Health};
use crate::ui::enemy_indicator::SpawnEnemyIndicator;
use crate::ui::health_bar_3d::SpawnHealthBar;
use crate::utils::sets::Set;
use crate::{
    components::colliders::VelocityColliderBundle, 
    utils::materials::default_outline, 
    AppState,
};

use super::bullet::{BulletTarget, BulletType};
use super::explosion::ExplosionEvent;

#[derive(Component)]
pub struct Cruiser;


#[derive(Component)]
pub struct CruiserShield;

#[derive(AssetCollection, Resource)]
struct CruiserAssets {
    #[asset(path = "cruiser.glb#Scene0")]
    pub cruiser_model: Handle<Scene>,
}

const CRUISER_HITBOX_SIZE: Vec3 = Vec3::new(3.5, 3., 13.);

fn cruiser_setup(
    mut commands: Commands, 
    assets: Res<CruiserAssets>, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

) {
    let Vec3 { x, y, z } = CRUISER_HITBOX_SIZE;
    let entity = commands.spawn((
        SceneBundle {
            scene: assets.cruiser_model.clone(),
            transform: Transform::from_translation(Vec3 {
                z: 20.0,
                ..Vec3::ZERO
            }),
            ..default()
        },
        VelocityColliderBundle {
            velocity: Velocity {
                linvel: Vec3 {
                    z: -2.0,
                    ..Vec3::ZERO
                },
                ..default()
            },
            collider: Collider::cuboid(x, y, z), 
            ..default()
        },
        Cruiser,
        BulletTarget {
            target_type: BulletType::Player, 
            bullet_damage: Some(20.)
        },
        OutlineBundle {
            outline: default_outline(),
            ..default()
        }, 
        DespawnOnDeath, 
        Health::new(100.0),
    )).id();

    let child = commands.spawn((
        CruiserShield, 
        PbrBundle {
            mesh: meshes.add(shape::UVSphere {
                radius: 10., 
                ..default()
            }.into()), 
            material: materials.add(StandardMaterial {
                base_color: Color::hex("2ae0ed0f").unwrap(), 
                unlit: true, 
                alpha_mode: AlphaMode::Blend, 
                ..default()
            }),
            transform: Transform::from_scale(Vec3 {
                z: 2., 
                ..Vec3::ONE
            }), 
            ..default()
        }, 
        Collider::ball(10.), 
        RigidBody::Fixed,  
        ActiveCollisionTypes::KINEMATIC_STATIC,
        BulletTarget {
            target_type: BulletType::Player, 
            bullet_damage: Some(10.0)
        },
        Health::new(100.0),
    )).id();

    commands.entity(entity).add_child(child);

    commands.add(SpawnHealthBar { entity: child, scale: 1., offset: Vec2::ZERO });
    commands.add(SpawnEnemyIndicator { enemy: entity });
}

fn cruiser_shield_death(
    query: Query<(Entity, &Health, &Parent), With<CruiserShield>>, 
    mut commands: Commands, 
) {
    for (entity, health, parent) in &query {
        if health.is_dead() {
            commands.add(SpawnHealthBar {
                entity: parent.get(), 
                scale: 1., 
                offset: Vec2::ZERO
            });

            commands.entity(entity).despawn_recursive();
        }
    }
}

fn cruiser_death(
    query: Query<(&Health, &Transform), (With<Cruiser>, Changed<Health>)>,
    mut explosion_events: EventWriter<ExplosionEvent>,
) {
    for (health, transform) in &query {
        if health.is_dead() {
            let forward = transform.forward();
            explosion_events.send_batch([
                ExplosionEvent {
                    position: transform.translation, 
                    radius: 10., 
                    ..default()
                }, 
                ExplosionEvent {
                    position: transform.translation + forward * 7., 
                    radius: 10., 
                    ..default()
                },
                ExplosionEvent {
                    position: transform.translation - forward * 7., 
                    radius: 10., 
                    ..default()
                },
            ]);
        }
    }
}


pub struct CruiserPLugin;

impl Plugin for CruiserPLugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, CruiserAssets>(AppState::MainSceneLoading)
            .add_systems(OnEnter(AppState::MainScene), cruiser_setup,)
            .add_systems(Update, (
                cruiser_shield_death, 
                cruiser_death.in_set(Set::ExplosionEvents), 
            ).run_if(in_state(AppState::MainScene)));
    }
}
