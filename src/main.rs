#![allow(clippy::type_complexity)] // Query types can be really complex
#![feature(let_chains)]


use bevy::{prelude::*, log::LogPlugin, window::PresentMode};
use bevy_asset_loader::loading_state::{LoadingStateAppExt, LoadingState};
use bevy_mod_outline::{OutlinePlugin, AutoGenerateOutlineNormalsPlugin};
use bevy_obj::ObjPlugin;
use bevy_rapier3d::prelude::*;
use bevy_round_ui::prelude::RoundUiPlugin;
use bevy_screen_diagnostics::{ScreenDiagnosticsPlugin, ScreenFrameDiagnosticsPlugin};
use bevy_toon_shader::{ToonShaderPlugin, ToonShaderSun};
use entities::EntitiesPlugin;
use components::ComponentsPlugin;
use particles::ParticlesPlugin;
use ui::UIPlugin;
use utils::scene_outline::SceneOutlinePlugin;

mod entities;
mod utils;
mod components;
mod particles;
mod ui;


#[derive(Component)]
pub struct Movement {
    /// Velocity
    pub vel: Vec3,
    /// Acceleration
    pub acc: Vec3,
    /// Maximum speed
    pub max_speed: Option<f32>,
    /// Friction must be between 0 and 1
    /// 0 means no friction, 1 that the object will stop immediately
    pub friction: f32,
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            vel: Vec3::ZERO,
            acc: Vec3::ZERO,
            max_speed: None,
            friction: 0.0,
        }
    }
}

fn movement_system(
    time: Res<Time>, 
    mut query: Query<(&mut Transform, &mut Movement)>
) {
    for (mut transform, mut movement) in &mut query {
        let acc = movement.acc;
        movement.vel += acc * time.delta_seconds();
        transform.translation += movement.vel * time.delta_seconds();

        if let Some(max_speed) = movement.max_speed {
            if movement.vel.length() > max_speed {
                movement.vel = movement.vel.normalize() * max_speed;
            }
        }
        let friction = movement.friction;
        movement.vel *= 1.0 - friction * time.delta_seconds();
    }
}


pub struct ScenePlugin3D;

impl Plugin for ScenePlugin3D {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, scene_setup_3d)
            .add_systems(Update, movement_system);
    }
}

fn setup_physics(
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    rapier_config.gravity = Vec3::ZERO;
}

fn scene_setup_3d(
    mut commands: Commands,
) {

    commands.insert_resource(AmbientLight {
        color: Color::WHITE, 
        brightness: 0.5,
    });

    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 10000.0, 
                color: Color::hex("fcd4b5").unwrap(), 
                ..default()
            }, 
            transform: Transform::from_xyz(0.0, 40.0, 0.0).with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2 + 0.1)),
            ..default()
        }, 
        ToonShaderSun
    ));
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States, Copy)]
enum AppState {
    #[default]
    MainSceneLoading, 
    MainScene,
    ParticleTestScene,
}

struct LoadingStateItem {
    loading_state: AppState,
    next_state: AppState,
}

impl AppState {
    pub const LOADING_STATES: &'static [LoadingStateItem] = &[
        LoadingStateItem {
            loading_state: AppState::MainSceneLoading,
            next_state: AppState::MainScene,
        },
    ];
}

fn main() {
    let mut app = App::new();

    // Add loading states
    for LoadingStateItem { loading_state, next_state } in AppState::LOADING_STATES {
        app.add_loading_state(
            LoadingState::new(*loading_state)
                .continue_to_state(*next_state)
        );
    }

    app
        .add_plugins(DefaultPlugins
            .set(LogPlugin {
                level: bevy::log::Level::INFO,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Space Game".into(),
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                }), 
                ..default()
            })
        )
        .add_plugins((
            OutlinePlugin, 
            AutoGenerateOutlineNormalsPlugin, 
            RapierPhysicsPlugin::<NoUserData>::default(), 
            ToonShaderPlugin, 
            ObjPlugin, 
            ScreenDiagnosticsPlugin::default(), 
            ScreenFrameDiagnosticsPlugin, 
            RoundUiPlugin, 
        ))
        .add_state::<AppState>()
        .add_systems(Startup, setup_physics)
        .add_plugins((
            ScenePlugin3D, 
            EntitiesPlugin, 
            ComponentsPlugin, 
            ParticlesPlugin, 
            SceneOutlinePlugin, 
            UIPlugin, 
        ));
    
    app.run();
}
