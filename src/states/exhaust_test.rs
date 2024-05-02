use std::f32::consts::FRAC_PI_2;

use bevy::{prelude::*, render::view::RenderLayers};

use crate::{
    entities::camera::RENDER_LAYER_2D,
    materials::{exhaust::{ExhaustMaterial, ExhaustRes}, shield::ShieldMaterial},
};

use super::{AppState, DespawnOnCleanup};

#[derive(Component)]
struct MainCamera;

fn setup_exhaust(
    mut commands: Commands,
    mut materials: ResMut<Assets<ExhaustMaterial>>,
    res: Res<ExhaustRes>,
) {
    let material = materials.add(ExhaustMaterial::default());
    commands.spawn(MaterialMeshBundle {
        mesh: res.mesh.clone(),
        transform: Transform::from_rotation(Quat::from_rotation_x(FRAC_PI_2)),
        material,
        ..default()
    });
}

fn setup_shield(
    mut commands: Commands, 
    mut materials: ResMut<Assets<ShieldMaterial>>, 
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let material = materials.add(ShieldMaterial {
        color: Color::hex("2ae0ed0f").unwrap(),
    });
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Sphere { radius: 2. }),
        material,
        ..default()
    });
}

fn init_camera(mut commands: Commands) {
    let mut camera_transform = Transform::from_xyz(0.0, 10.0, 0.0);
    camera_transform.look_at(Vec3::ZERO, Vec3::X);

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

    commands.spawn((
        Camera3dBundle {
            transform: camera_transform,
            projection: Projection::Perspective(PerspectiveProjection {
                far: 10000.0,
                ..default()
            }),
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::MIDNIGHT_BLUE),
                ..default()
            },
            ..default()
        },
        MainCamera,
    ));
}

fn camera_update(mut camera: Query<&mut Transform, With<MainCamera>>, time: Res<Time>) {
    const RADIUS: f32 = 10.0;
    let time = time.elapsed_seconds();
    for mut transform in camera.iter_mut() {
        transform.translation.x = f32::sin(time) * RADIUS;
        transform.translation.z = f32::cos(time) * RADIUS;
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

pub struct ExhaustTestPlugin;
impl Plugin for ExhaustTestPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::TestScene),
            (setup_shield, init_camera),
        )
        .add_systems(
            Update,
            camera_update.run_if(in_state(AppState::TestScene)),
        );
    }
}
