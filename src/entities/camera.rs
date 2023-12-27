use bevy::{prelude::*, core_pipeline::clear_color::ClearColorConfig};
use bevy_toon_shader::ToonShaderMainCamera;

use crate::{Movement, AppState};

use super::player::Player;

#[derive(Component)]
pub struct CameraComponent;

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
) {
    let mut camera_tranform = Transform::from_xyz(0.0, 70.0, 0.0);
    camera_tranform.rotate(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));

    commands.spawn((
        Camera3dBundle {
            transform: camera_tranform,
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
        CameraComponent, 
        ToonShaderMainCamera, 
        Movement::default(),
    ));
}

pub struct CameraComponentPlugin;

impl Plugin for CameraComponentPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Running), camera_setup)
            .add_systems(Update, camera_follow_system.run_if(in_state(AppState::Running)));
    }
}