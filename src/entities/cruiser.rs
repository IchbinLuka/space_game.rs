use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_mod_outline::OutlineBundle;
use bevy_rapier3d::{dynamics::Velocity, geometry::Collider};
use bevy_rapier3d::prelude::*;

use crate::ui::health_bar_3d::HealthBarSpawnEvent;
use crate::utils::sets::Set;
use crate::{
    components::colliders::VelocityColliderBundle, 
    utils::materials::default_outline, 
    AppState, 
    ui::enemy_indicator::{EnemyIndicatorBundle, EnemyIndicatorRes},
};

use super::{bullet::{BulletTarget, BulletType}, spaceship::Health};

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
    indicator_res: Res<EnemyIndicatorRes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut health_bar_spawn_events: EventWriter<HealthBarSpawnEvent>,
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
            target_type: BulletType::Bot, 
            bullet_damage: None
        },
        OutlineBundle {
            outline: default_outline(),
            ..default()
        }, 
        Health::new(100.0),
    )).with_children(|c| {
        let entity = c.spawn((
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

        health_bar_spawn_events.send(HealthBarSpawnEvent { entity });
    }).id();

    commands.spawn(
        EnemyIndicatorBundle::new(&indicator_res, entity),
    );
}

fn cruiser_shield_death(
    query: Query<(Entity, &Health), With<CruiserShield>>, 
    mut commands: Commands, 
) {
    for (entity, health) in &query {
        if health.is_dead() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub struct CruiserPLugin;

impl Plugin for CruiserPLugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, CruiserAssets>(AppState::MainSceneLoading)
            .add_systems(
                OnEnter(AppState::MainScene), 
                cruiser_setup.in_set(Set::HealthBarSpawn),
            )
            .add_systems(Update, (
                cruiser_shield_death
            ).run_if(in_state(AppState::MainScene)));
    }
}
