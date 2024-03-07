use bevy::{
    core_pipeline::{
        clear_color::ClearColorConfig,
        prepass::{DepthPrepass, NormalPrepass},
        Skybox,
    },
    prelude::*,
    render::{
        render_resource::{TextureViewDescriptor, TextureViewDimension},
        view::RenderLayers,
    },
};
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};

use crate::states::AppState;
use crate::{
    states::{game_running, ON_GAME_STARTED},
    utils::sets::Set,
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

fn camera_setup(
    mut commands: Commands,
    camera_assets: Res<CameraAssets>,
    mut images: ResMut<Assets<Image>>,
) {
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

    let mut camera_transform = Transform::from_xyz(0.0, 100.0, 0.0);
    camera_transform.rotate(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));

    commands.spawn((
        Camera3dBundle {
            transform: camera_transform,
            projection: Projection::Perspective(PerspectiveProjection {
                far: 10000.0,
                ..default()
            }),
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::Custom(Color::MIDNIGHT_BLUE),
                ..default()
            },
            ..default()
        },
        Skybox(camera_assets.skybox.clone()),
        CameraComponent,
        DepthPrepass,
        NormalPrepass,
        Movement::default(),
    ));

    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None,
            },
            camera: Camera {
                order: 1,
                ..default()
            },
            ..default()
        },
        RenderLayers::layer(RENDER_LAYER_2D),
    ));
}

#[derive(AssetCollection, Resource)]
struct CameraAssets {
    #[asset(path = "skybox.png")]
    skybox: Handle<Image>,
}

pub struct CameraComponentPlugin;

impl Plugin for CameraComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, CameraAssets>(AppState::MainSceneLoading)
            .add_systems(ON_GAME_STARTED, camera_setup)
            .add_systems(
                Update,
                camera_follow_system
                    .in_set(Set::CameraMovement)
                    .run_if(game_running()),
            );
    }
}
