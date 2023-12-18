use bevy::prelude::*;

use crate::Movement;

use super::player::Player;

#[derive(Component)]
pub struct CameraComponent;

fn camera_follow_system(
    timer: Res<Time>,
    mut camera_query: Query<(&mut Movement, &Transform, With<CameraComponent>, Without<Player>)>,
    cube_query: Query<(&Transform, &Movement, With<Player>)>,
) {
    for (mut camera_movement, camera_transform, _, _) in &mut camera_query {
        let cube_tranform = cube_query.iter().next();
        if let Some((transform, cube_movement, _)) = cube_tranform {
            let delta_vel = (transform.translation.xz() - camera_transform.translation.xz()) * 0.1 * timer.delta_seconds();
            camera_movement.vel = cube_movement.vel * 0.9 + Vec3::new(delta_vel.x, 0.0, delta_vel.y);
        } else {
            println!("No cube transform found");
        }
    }
}

fn camera_setup(
    mut commands: Commands,
) {
    let mut camera_tranform = Transform::from_xyz(0.0, 15.0, 0.0);
    camera_tranform.rotate(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));

    commands.spawn((
        Camera3dBundle {
            transform: camera_tranform,
            ..default()
        }, 
        CameraComponent, 
        Movement::default(),
    ));
}

pub struct CameraComponentPlugin;

impl Plugin for CameraComponentPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, camera_setup)
            .add_systems(Update, camera_follow_system);
    }
}