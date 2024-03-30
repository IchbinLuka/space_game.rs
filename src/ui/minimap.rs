use std::f32::consts::PI;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::sprite::{Anchor, Mesh2dHandle};
use bevy::window::WindowResized;
use bevy_asset_loader::loading_state::LoadingStateAppExt;
use bevy_asset_loader::prelude::AssetCollection;

use crate::entities::camera::RENDER_LAYER_2D;
use crate::entities::planet::Planet;
use crate::states::{AppState, game_running, ON_GAME_STARTED};

pub const MINIMAP_RANGE: f32 = 400.;
pub const MINIMAP_SIZE: f32 = 300.;
const MINIMAP_PADDING: f32 = 10.;

#[derive(Component)]
struct Minimap;

#[derive(Component)]
struct MinimapObject {
    entity: Entity,
}

#[derive(Resource)]
struct MinimapRes {
    player_material: Handle<ColorMaterial>,
    player_mesh: Handle<Mesh>,
    bot_material: Handle<ColorMaterial>,
    bot_mesh: Handle<Mesh>,
    space_station_material: Handle<ColorMaterial>,
    space_station_mesh: Handle<Mesh>,
}

#[derive(Component)]
pub struct ShowOnMinimap {
    pub sprite: Handle<Image>,
    pub size: Option<Vec2>,
}

fn get_minimap_pos(window_width: f32, window_height: f32) -> Vec3 {
    Vec3::new(
        window_width / 2. - MINIMAP_SIZE / 2. - MINIMAP_PADDING,
        window_height / 2. - MINIMAP_SIZE / 2. - MINIMAP_PADDING,
        0.,
    )
}

fn setup_minimap(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window_query: Query<&Window>,
) {
    let Ok(window) = window_query.get_single() else {
        warn!("Could not find a window");
        return;
    };

    commands.spawn((
        Minimap,
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::splat(MINIMAP_SIZE)),
                ..default()
            },
            transform: Transform::from_translation(get_minimap_pos(window.width(), window.height())),
            ..default()
        }
    ));

    commands.insert_resource(MinimapRes {
        // TODO: Update colors and meshes
        player_material: materials.add(Color::GREEN.into()),
        player_mesh: meshes.add(Mesh::from(shape::Circle { radius: 5., ..shape::Circle::default() })),
        bot_material: materials.add(Color::RED.into()),
        bot_mesh: meshes.add(Mesh::from(shape::Circle { radius: 5., ..shape::Circle::default() })),
        space_station_material: materials.add(Color::YELLOW.into()),
        space_station_mesh: meshes.add(Mesh::from(shape::Circle { radius: 5., ..shape::Circle::default() })),
    });
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
        let marker = commands.spawn((
            MinimapObject { entity },
            SpriteBundle {
                sprite: Sprite {
                    anchor: Anchor::Center,
                    custom_size: show_on_minimap.size,
                    ..default()
                },
                texture: show_on_minimap.sprite.clone(),
                ..default()
            },
            RenderLayers::layer(RENDER_LAYER_2D),
        )).id();

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
    mut minimap_objects: Query<(&MinimapObject, &mut Transform, Entity), Without<ShowOnMinimap>>,
    mut commands: Commands,
) {
    for (minimap_obj, mut transform, entity) in &mut minimap_objects {
        let Ok(object_transform) = show_on_minimap_query.get(minimap_obj.entity) else {
            commands.entity(entity).despawn_recursive();
            continue;
        };

        let minimap_pos = object_transform.translation / MINIMAP_RANGE * MINIMAP_SIZE;

        if minimap_pos.length() > MINIMAP_SIZE / 2. {
            continue;
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
}

pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_collection_to_loading_state::<_, MinimapAssets>(AppState::MainSceneLoading)
            .add_systems(ON_GAME_STARTED, setup_minimap)
            .add_systems(Update, (
                update_minimap,
                spawn_minimap_objects,
                window_resize, 
            ).run_if(game_running()));
    }
}