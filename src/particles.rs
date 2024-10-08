use std::f32::consts::FRAC_PI_2;

use bevy::{
    color::palettes::css,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
};

use crate::states::AppState;

pub mod fire_particles;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ParticleMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
}

impl ParticleMaterial {
    pub fn new(color: Color) -> Self {
        Self {
            color: color.into(),
        }
    }
}

impl Default for ParticleMaterial {
    fn default() -> Self {
        Self::new(css::BLACK.into())
    }
}

impl Material for ParticleMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/point.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

fn particle_test_scene_setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ParticleMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mesh = meshes.add(Rectangle::new(5.0, 5.0));
    let material = materials.add(ParticleMaterial {
        color: css::GREEN.into(),
    });
    commands.spawn(MaterialMeshBundle {
        material,
        mesh,
        transform: Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
        ..default()
    });
}

fn init_camera(mut commands: Commands) {
    let mut camera_transform = Transform::from_xyz(0.0, 10.0, 0.0);
    camera_transform.look_at(Vec3::ZERO, Vec3::X);

    commands.spawn(Camera3dBundle {
        transform: camera_transform,
        projection: Projection::Perspective(PerspectiveProjection {
            far: 10000.0,
            ..default()
        }),
        camera: Camera {
            clear_color: ClearColorConfig::Custom(css::MIDNIGHT_BLUE.into()),
            ..default()
        },
        ..default()
    });
}

fn camera_update(mut query: Query<&mut Transform, With<Camera3d>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.translation.x = time.elapsed_seconds().sin() * 1.0;
        transform.translation.z = time.elapsed_seconds().cos() * 1.0;
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            MaterialPlugin::<ParticleMaterial>::default(),
            fire_particles::FireParticlesPlugin,
        ))
        .add_systems(
            OnEnter(AppState::ParticleTestScene),
            (particle_test_scene_setup, init_camera),
        )
        .add_systems(
            Update,
            (camera_update,).run_if(in_state(AppState::ParticleTestScene)),
        );
    }
}
