pub mod exhaust_test;
pub mod loading_screen;
pub mod main_scene;
pub mod pause;
pub mod start_screen;

use bevy::app::{App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::system::ReadOnlySystem;
use bevy::prelude::*;
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};
use bevy_rapier3d::plugin::{RapierConfiguration, TimestepMode};
use iyes_progress::ProgressPlugin;

use crate::utils::misc::cleanup_system;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States, Copy)]
pub enum AppState {
    #[default]
    StartScreenLoading,
    StartScreen,
    MainSceneLoading,
    MainScene,
    GameOver,
    ParticleTestScene,
    TestSceneLoading,
    TestScene,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States, Copy)]
pub enum PausedState {
    Paused,
    #[default]
    Running,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States, Copy)]
pub enum StartScreenState {
    // TODO: Replace this with computed states once Bevy is updated
    #[default]
    Menu,
    Leaderboard,
}

pub struct LoadingStateItem {
    pub loading_state: AppState,
    pub next_state: AppState,
}

impl AppState {
    pub const LOADING_STATES: &'static [LoadingStateItem] = &[
        LoadingStateItem {
            loading_state: AppState::MainSceneLoading,
            next_state: AppState::MainScene,
        },
        LoadingStateItem {
            loading_state: AppState::StartScreenLoading,
            next_state: AppState::StartScreen,
        },
        LoadingStateItem {
            loading_state: AppState::TestSceneLoading,
            next_state: AppState::TestScene,
        },
    ];
}

#[inline]
pub fn game_running() -> impl ReadOnlySystem<In = (), Out = bool> {
    in_state(AppState::MainScene).and_then(in_state(PausedState::Running))
}

#[inline]
pub fn game_paused() -> impl ReadOnlySystem<In = (), Out = bool> {
    in_state(AppState::MainScene).and_then(in_state(PausedState::Paused))
}

#[inline]
pub fn in_start_menu() -> impl FnMut(Option<Res<State<AppState>>>) -> bool + Clone {
    in_state(AppState::StartScreen)
}

#[inline]
pub fn game_over() -> impl FnMut(Option<Res<State<AppState>>>) -> bool + Clone {
    in_state(AppState::GameOver)
}

#[inline]
pub fn pause_physics(rapier_config: &mut RapierConfiguration) {
    rapier_config.physics_pipeline_active = false;
    rapier_config.query_pipeline_active = false;
}

#[inline]
pub fn resume_physics(rapier_config: &mut RapierConfiguration) {
    rapier_config.physics_pipeline_active = true;
    rapier_config.query_pipeline_active = true;
}

#[inline]
pub fn slow_down_physics(rapier_config: &mut RapierConfiguration) {
    rapier_config.timestep_mode = TimestepMode::Variable {
        time_scale: 0.025,
        substeps: 1,
        max_dt: 1.0 / 60.0,
    };
}

#[inline]
pub fn reset_physics_speed(rapier_config: &mut RapierConfiguration) {
    rapier_config.timestep_mode = TimestepMode::Variable {
        time_scale: 1.0,
        substeps: 1,
        max_dt: 1.0 / 60.0,
    };
}

pub const ON_GAME_STARTED: OnEnter<AppState> = OnEnter(AppState::MainScene);

#[derive(Component, Default)]
pub struct DespawnOnCleanup;

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        // Add loading states
        for LoadingStateItem {
            loading_state,
            next_state,
        } in AppState::LOADING_STATES
        {
            // app.add_loading_state(LoadingState::new(*loading_state).continue_to_state(*next_state));
            app.add_loading_state(LoadingState::new(*loading_state))
                .add_plugins(ProgressPlugin::new(*loading_state).continue_to(*next_state))
                .add_systems(OnEnter(*loading_state), cleanup_system::<DespawnOnCleanup>);
        }

        app.init_state::<AppState>()
            .init_state::<PausedState>()
            .init_state::<StartScreenState>()
            .add_plugins((
                pause::PausePlugin,
                loading_screen::LoadingScreenPlugin,
                start_screen::StartScreenPlugin,
                main_scene::MainScenePlugin,
                // exhaust_test::ExhaustTestPlugin,
            ));
    }
}
