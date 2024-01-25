use bevy::{prelude::*, render::view::RenderLayers, sprite::Anchor};

use crate::{components::health::Health, entities::camera::RENDER_LAYER_2D, utils::sets::Set, AppState};

use super::sprite_3d_renderer::Sprite3DObject;

const HEALTH_BAR_HEIGHT: f32 = 20.;
const HEALTH_BAR_WIDTH: f32 = 150.;

const HEALTH_BAR_PADDING: f32 = 2.;

const HEALTH_BAR_CONTENT_WIDTH: f32 = HEALTH_BAR_WIDTH - HEALTH_BAR_PADDING * 2.;
const HEALTH_BAR_CONTENT_TRANSFORM: Vec3 = Vec3::new(HEALTH_BAR_WIDTH * -0.5 + HEALTH_BAR_PADDING, 0., 1.);

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

#[derive(Event)]
pub struct HealthBarSpawnEvent {
    pub entity: Entity,
    pub scale: f32, 
    pub offset: Vec2, 
}

fn health_bar_spawn(
    mut commands: Commands,
    mut events: EventReader<HealthBarSpawnEvent>,
) {
    for event in events.read() {
        commands.spawn((
            HealthBar3dBackground, 
            Sprite3DObject { parent: event.entity, offset: event.offset }, 
            SpriteBundle {
                sprite: Sprite { 
                    color: Color::BLACK, 
                    custom_size: Some(Vec2::new(HEALTH_BAR_WIDTH, HEALTH_BAR_HEIGHT)),
                    ..default()
                 },
                 transform: Transform::from_scale(Vec3::splat(event.scale)), 
                ..default()
            }, 
            RenderLayers::layer(RENDER_LAYER_2D), 
        )).with_children(|c| {
            c.spawn((
                HealthBar3d { entity: event.entity }, 
                SpriteBundle {
                    sprite: Sprite { 
                        color: Color::RED, 
                        custom_size: Some(Vec2::new(HEALTH_BAR_CONTENT_WIDTH, HEALTH_BAR_HEIGHT - HEALTH_BAR_PADDING * 2.)),
                        anchor: Anchor::CenterLeft, 
                        ..default()
                     },
                     transform: Transform::from_translation(HEALTH_BAR_CONTENT_TRANSFORM), 
                    ..default()
                },
                RenderLayers::layer(RENDER_LAYER_2D), 
            ));
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
        app
            .add_event::<HealthBarSpawnEvent>()
            .add_systems(Update, (
                health_bar_3d_update, 
                health_bar_spawn.after(Set::HealthBarSpawn), 
            ).run_if(in_state(AppState::MainScene)));
    }
}