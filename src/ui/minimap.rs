use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use crate::entities::camera::RENDER_LAYER_2D;

use crate::states::{game_running, ON_GAME_STARTED};

const MINIMAP_RANGE: f32 = 400.;
const MINIMAP_SIZE: f32 = 300.;
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

pub enum MinimapObjectType {
    Player,
    Bot,
    SpaceStation,
}

#[derive(Component)]
pub struct ShowOnMinimap {
    pub(crate) object_type: MinimapObjectType,
    sprite: Handle<Image>,
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
            transform: Transform::from_translation(Vec3::new(
                window.width() / 2. - MINIMAP_SIZE / 2. - MINIMAP_PADDING,
                window.height() / 2. - MINIMAP_SIZE / 2. - MINIMAP_PADDING,
                0.,
            )),
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
    minimap_res: Res<MinimapRes>,
) {
    let Ok(minimap) = minimaps.get_single() else {
        return;
    };

    for (entity, show_on_minimap) in &new_objects {
        let (material, mesh) = match show_on_minimap.object_type {
            MinimapObjectType::Player => (&minimap_res.player_material, &minimap_res.player_mesh),
            MinimapObjectType::Bot => (&minimap_res.bot_material, &minimap_res.bot_mesh),
            MinimapObjectType::SpaceStation => (&minimap_res.space_station_material, &minimap_res.space_station_mesh),
        };


        let marker = commands.spawn((
            MinimapObject { entity },
            MaterialMesh2dBundle {
                material: material.clone(),
                mesh: Mesh2dHandle(mesh.clone()),
                ..default()
            },
            RenderLayers::layer(RENDER_LAYER_2D),
        )).id();

        commands.entity(minimap).add_child(marker);
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
        transform.rotation = Quat::from_rotation_z(-object_transform.rotation.to_euler(EulerRot::XYZ).1);
    }
}

pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(ON_GAME_STARTED, setup_minimap)
            .add_systems(Update, (
                update_minimap,
                spawn_minimap_objects,
            ).run_if(game_running()));
    }
}