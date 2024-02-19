use bevy::{ecs::system::Command, prelude::*, render::view::RenderLayers, sprite::Anchor};

use crate::{
    components::health::Health,
    entities::camera::RENDER_LAYER_2D,
    states::game_running,
};

use super::sprite_3d_renderer::Sprite3DObject;

const HEALTH_BAR_HEIGHT: f32 = 20.;
const HEALTH_BAR_WIDTH: f32 = 150.;

const HEALTH_BAR_PADDING: f32 = 2.;

const HEALTH_BAR_CONTENT_WIDTH: f32 = HEALTH_BAR_WIDTH - HEALTH_BAR_PADDING * 2.;
const HEALTH_BAR_CONTENT_TRANSFORM: Vec3 =
    Vec3::new(HEALTH_BAR_WIDTH * -0.5 + HEALTH_BAR_PADDING, 0., 1.);
const HEALTH_BAR_SHIELD_TRANSFORM: Vec3 = Vec3::new(HEALTH_BAR_WIDTH * -0.5, 0., 2.);

#[derive(Component)]
pub struct HealthBar3d {
    entity: Entity,
}

#[derive(Component)]
pub struct HealthBar3dBackground;

#[derive(Bundle)]
pub struct HealthBar3dBundle {
    pub health_bar: HealthBar3d,
    sprite: SpriteBundle,
}

pub struct SpawnHealthBar {
    pub entity: Entity,
    pub scale: f32,
    pub offset: Vec2,
    pub shield_entity: Option<Entity>,
}

impl Command for SpawnHealthBar {
    fn apply(self, world: &mut World) {
        world
            .spawn((
                HealthBar3dBackground,
                Sprite3DObject {
                    parent: self.entity,
                    offset: self.offset,
                },
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::BLACK,
                        custom_size: Some(Vec2::new(HEALTH_BAR_WIDTH, HEALTH_BAR_HEIGHT)),
                        ..default()
                    },
                    transform: Transform::from_scale(Vec3::splat(self.scale)),
                    ..default()
                },
                RenderLayers::layer(RENDER_LAYER_2D),
            ))
            .with_children(|c| {
                c.spawn((
                    HealthBar3d {
                        entity: self.entity,
                    },
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::RED,
                            custom_size: Some(Vec2::new(
                                HEALTH_BAR_CONTENT_WIDTH,
                                HEALTH_BAR_HEIGHT - HEALTH_BAR_PADDING * 2.,
                            )),
                            anchor: Anchor::CenterLeft,
                            ..default()
                        },
                        transform: Transform::from_translation(HEALTH_BAR_CONTENT_TRANSFORM),
                        ..default()
                    },
                    RenderLayers::layer(RENDER_LAYER_2D),
                ));

                if let Some(shield_entity) = self.shield_entity {
                    c.spawn((
                        HealthBar3d {
                            entity: shield_entity,
                        },
                        SpriteBundle {
                            sprite: Sprite {
                                color: Color::hex("2ae0ed80").unwrap(),
                                custom_size: Some(Vec2::new(HEALTH_BAR_WIDTH, HEALTH_BAR_HEIGHT)),
                                anchor: Anchor::CenterLeft,
                                ..default()
                            },
                            transform: Transform::from_translation(HEALTH_BAR_SHIELD_TRANSFORM),
                            ..default()
                        },
                        RenderLayers::layer(RENDER_LAYER_2D),
                    ));
                }
            });
    }
}

fn health_bar_3d_update(
    mut health_bar_query: Query<(&mut Transform, &HealthBar3d)>,
    health_query: Query<&Health, Changed<Health>>,
) {
    for (mut transform, health_bar) in &mut health_bar_query {
        if let Ok(health) = health_query.get(health_bar.entity) {
            transform.scale.x = health.health / health.max_health;
        }
    }
}

pub struct HealthBar3DPlugin;

impl Plugin for HealthBar3DPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            health_bar_3d_update.run_if(game_running()),
        );
    }
}
