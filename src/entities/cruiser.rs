use bevy::{prelude::*, render::view::RenderLayers};
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_mod_outline::OutlineBundle;
use bevy_rapier3d::{dynamics::Velocity, geometry::{Collider, CollidingEntities}};
use bevy_rapier3d::prelude::*;

use crate::{
    components::colliders::VelocityColliderBundle, utils::{materials::default_outline, misc::CollidingEntitiesExtension}, AppState, ui::{sprite_3d_renderer::Node3DObject, enemy_indicator::{EnemyIndicatorBundle, EnemyIndicatorRes}},
};

use super::{camera::RENDER_LAYER_2D, bullet::{BulletTarget, BulletType, Bullet}, spaceship::Health, explosion::ExplosionEvent};

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
        Health(100.0),
    )).with_children(|c| {
        c.spawn((
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
            Health(100.0),
        ));
    }).id();

    commands.spawn(
        EnemyIndicatorBundle::new(&indicator_res, entity),
    );

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(t!("cruiser"), default()), 
            ..default()
        }, 
        RenderLayers::layer(RENDER_LAYER_2D), 
        Node3DObject {
            parent: entity, 
        }
    ));
    
}


fn cruiser_collisions(
    mut cruisers: Query<(&mut Health, &CollidingEntities), With<Cruiser>>,
    bullet_query: Query<(&Bullet, &Transform)>, 
    mut explosion_events: EventWriter<ExplosionEvent>,
) {
    for (mut health, colliding) in &mut cruisers {
        for (bullet, bullet_transform) in colliding.filter_fulfills_query(&bullet_query) {
            if bullet.bullet_type != BulletType::Player {
                continue;
            }
            
            explosion_events.send(ExplosionEvent { 
                position: bullet_transform.translation, 
                ..default()
            });

            health.take_damage(10.0);
        }
    }
}

pub struct CruiserPLugin;

impl Plugin for CruiserPLugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, CruiserAssets>(AppState::MainSceneLoading)
            .add_systems(OnEnter(AppState::MainScene), cruiser_setup)
            .add_systems(Update, (
                cruiser_collisions,
            ).run_if(in_state(AppState::MainScene)));
    }
}
