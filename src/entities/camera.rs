use bevy::{
    core_pipeline::{
        prepass::{DepthPrepass, NormalPrepass}, Skybox
    },
    prelude::*,
    render::{
        render_resource::{TextureViewDescriptor, TextureViewDimension},
        view::RenderLayers,
    },
};
use bevy_asset_loader::asset_collection::AssetCollection;
use cfg_if::cfg_if;

use crate::{
    states::{game_running, AppState, DespawnOnCleanup, ON_GAME_STARTED},
    utils::{asset_loading::AppExtension, sets::Set},
    Movement,
};

use super::spaceship::player::Player;

#[derive(Component)]
pub struct CameraComponent;

pub const RENDER_LAYER_2D: u8 = 1;

fn camera_follow_system(
    mut camera_query: Query<&mut Transform, (With<CameraComponent>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,
) {
    for mut camera_transform in &mut camera_query {
        let player_tranform = player_query.iter().next();
        if let Some(transform) = player_tranform {
            camera_transform.translation = Vec3::new(
                transform.translation.x,
                camera_transform.translation.y,
                transform.translation.z,
            );
        } else {
            println!("No cube transform found");
        }
    }
}

fn setup_skybox_texture(mut images: ResMut<Assets<Image>>, camera_assets: Res<CameraAssets>) {
    let image = images.get_mut(&camera_assets.skybox).unwrap();
    // NOTE: PNGs do not have any metadata that could indicate they contain a cubemap texture,
    // so they appear as one texture. The following code reconfigures the texture as necessary.
    if image.texture_descriptor.array_layer_count() == 1 {
        image.reinterpret_stacked_2d_as_array(image.height() / image.width());
        image.texture_view_descriptor = Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..default()
        });
    }
}

fn camera_setup(mut commands: Commands, camera_assets: Res<CameraAssets>) {
    let mut camera_transform = Transform::from_xyz(0.0, 75.0, 0.0);
    camera_transform.rotate(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));

    spawn_camera(&mut commands, camera_transform, &camera_assets);
}

pub fn spawn_camera(commands: &mut Commands, transform: Transform, camera_assets: &CameraAssets) {
    commands.spawn((
        DespawnOnCleanup,
        Camera2dBundle {
            camera: Camera {
                order: 1,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            ..default()
        },
        RenderLayers::layer(RENDER_LAYER_2D),
    ));

    let mut camera = commands.spawn((
        Camera3dBundle {
            transform,
            projection: Projection::Perspective(PerspectiveProjection {
                far: 10000.0,
                ..default()
            }),
            ..default()
        },
        Skybox {
            image: camera_assets.skybox.clone(),
            brightness: 1000.,
        },
        CameraComponent,
        Movement::default(),
        DespawnOnCleanup,
    ));

    cfg_if! {
        if #[cfg(not(target_family = "wasm"))] {
            camera.insert((
                DepthPrepass,
                NormalPrepass,
            ));
        }
    }
}

fn control_camera(
    input: Res<ButtonInput<KeyCode>>,
    mut camera: Query<&mut Transform, With<CameraComponent>>,
    time: Res<Time>,
) {
    let Ok(mut camera_transform) = camera.get_single_mut() else {
        return;
    };

    if input.pressed(KeyCode::ArrowUp) {
        camera_transform.rotate_local_x(time.delta_seconds() * 2.0);
    }
    if input.pressed(KeyCode::ArrowDown) {
        camera_transform.rotate_local_x(-time.delta_seconds() * 2.0);
    }
    if input.pressed(KeyCode::ArrowLeft) {
        camera_transform.rotate_local_y(time.delta_seconds() * 2.0);
    }
    if input.pressed(KeyCode::ArrowRight) {
        camera_transform.rotate_local_y(-time.delta_seconds() * 2.0);
    }
}

#[derive(AssetCollection, Resource)]
pub struct CameraAssets {
    #[asset(path = "skybox.png")]
    skybox: Handle<Image>,
}

pub struct CameraComponentPlugin;

impl Plugin for CameraComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_states::<CameraAssets>(&[
            AppState::MainSceneLoading,
            AppState::StartScreenLoading,
        ])
        .add_systems(
            OnEnter(AppState::StartScreen),
            setup_skybox_texture.in_set(Set::CameraSkyboxInit),
        )
        .add_systems(
            OnEnter(AppState::MainScene),
            setup_skybox_texture.in_set(Set::CameraSkyboxInit),
        )
        .add_systems(ON_GAME_STARTED, camera_setup)
        .add_systems(
            Update,
            (
                camera_follow_system
                    .in_set(Set::CameraMovement)
                    .run_if(game_running()),
                control_camera,
            ),
        );
    }
}
