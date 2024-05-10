use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::sprite::Anchor;
use bevy::window::WindowResized;
use bevy_asset_loader::prelude::AssetCollection;
use std::f32::consts::PI;

use crate::entities::camera::RENDER_LAYER_2D;
use crate::states::{game_running, AppState, DespawnOnCleanup, ON_GAME_STARTED};
use crate::utils::asset_loading::AppExtension;

pub const MINIMAP_RANGE: f32 = 400.;
pub const MINIMAP_SIZE: f32 = 300.;
const MINIMAP_PADDING: f32 = 10.;

#[derive(Component)]
struct Minimap;

#[derive(Component)]
struct MinimapObject {
    entity: Entity,
}

#[derive(Component, Default)]
pub struct ShowOnMinimap {
    pub sprite: Handle<Image>,
    pub size: MinimapSize,
}

pub enum MinimapSize {
    Scale(f32),
    Custom(Vec2),
}

impl Default for MinimapSize {
    fn default() -> Self {
        MinimapSize::Scale(1.)
    }
}

impl From<f32> for MinimapSize {
    fn from(scale: f32) -> Self {
        MinimapSize::Scale(scale)
    }
}

#[inline]
fn get_minimap_pos(window_width: f32, window_height: f32) -> Vec3 {
    Vec3::new(
        window_width / 2. - MINIMAP_SIZE / 2. - MINIMAP_PADDING,
        window_height / 2. - MINIMAP_SIZE / 2. - MINIMAP_PADDING,
        0.,
    )
}

fn setup_minimap(mut commands: Commands, window_query: Query<&Window>) {
    let Ok(window) = window_query.get_single() else {
        warn!("Could not find a window");
        return;
    };

    commands.spawn((
        Minimap,
        DespawnOnCleanup,
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::splat(MINIMAP_SIZE)),
                ..default()
            },
            transform: Transform::from_translation(get_minimap_pos(
                window.width(),
                window.height(),
            )),
            ..default()
        },
    ));
}

fn spawn_minimap_objects(
    new_objects: Query<(Entity, &ShowOnMinimap), Added<ShowOnMinimap>>,
    mut commands: Commands,
    minimaps: Query<Entity, With<Minimap>>,
) {
    let Ok(minimap) = minimaps.get_single() else {
        return;
    };

    for (entity, show_on_minimap) in &new_objects {
        let scale = if let MinimapSize::Scale(scale) = show_on_minimap.size {
            scale
        } else {
            1.0
        };

        let marker = commands
            .spawn((
                MinimapObject { entity },
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: if let MinimapSize::Custom(size) = show_on_minimap.size {
                            Some(size)
                        } else {
                            None
                        },
                        anchor: Anchor::Center,
                        ..default()
                    },
                    transform: Transform::from_scale(Vec3::new(scale, scale, 1.)),
                    texture: show_on_minimap.sprite.clone(),
                    visibility: Visibility::Hidden,
                    ..default()
                },
                RenderLayers::layer(RENDER_LAYER_2D),
            ))
            .id();

        commands.entity(minimap).add_child(marker);
    }
}

fn window_resize(
    mut resize_reader: EventReader<WindowResized>,
    mut minimap_query: Query<&mut Transform, With<Minimap>>,
) {
    for event in resize_reader.read() {
        for mut transform in &mut minimap_query {
            transform.translation = get_minimap_pos(event.width, event.height);
        }
    }
}

fn update_minimap(
    show_on_minimap_query: Query<&Transform, With<ShowOnMinimap>>,
    mut minimap_objects: Query<
        (&MinimapObject, &mut Transform, &mut Visibility, Entity),
        Without<ShowOnMinimap>,
    >,
    mut commands: Commands,
) {
    for (minimap_obj, mut transform, mut visibility, entity) in &mut minimap_objects {
        let Ok(object_transform) = show_on_minimap_query.get(minimap_obj.entity) else {
            commands.entity(entity).despawn_recursive();
            continue;
        };

        let minimap_pos = object_transform.translation / MINIMAP_RANGE * MINIMAP_SIZE;

        if minimap_pos.length() > MINIMAP_SIZE / 2. {
            *visibility = Visibility::Hidden;
            continue;
        } else if *visibility == Visibility::Hidden {
            *visibility = Visibility::Visible;
        }

        transform.translation = Vec3::new(minimap_pos.x, -minimap_pos.z, 0.);
        let forward = object_transform.forward();
        transform.rotation = Quat::from_rotation_z(
            -forward.angle_between(Vec3::Z) * forward.cross(Vec3::Z).y.signum() + PI,
        );
    }
}

#[derive(Resource, AssetCollection)]
pub struct MinimapAssets {
    #[asset(path = "textures/minimap/player_indicator.png")]
    pub player_indicator: Handle<Image>,
    #[asset(path = "textures/minimap/enemy.png")]
    pub enemy_indicator: Handle<Image>,
    #[asset(path = "textures/minimap/space_station.png")]
    pub space_station_indicator: Handle<Image>,
    #[asset(path = "textures/minimap/planet.png")]
    pub planet_indicator: Handle<Image>,
    #[asset(path = "textures/minimap/cruiser.png")]
    pub cruiser_indicator: Handle<Image>,
}

pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_states::<MinimapAssets>(&[
            AppState::MainSceneLoading,
            AppState::StartScreenLoading,
        ])
        .add_systems(ON_GAME_STARTED, setup_minimap)
        .add_systems(
            Update,
            (update_minimap, spawn_minimap_objects, window_resize).run_if(game_running()),
        );
    }
}
