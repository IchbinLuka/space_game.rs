pub mod loading_screen;
pub mod pause;
pub mod start_screen;
pub mod main_scene;

use bevy::app::{App, Plugin};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::schedule::common_conditions::in_state;
use bevy::ecs::schedule::{Condition, OnEnter, States};
use bevy::ecs::system::{Commands, Query, ReadOnlySystem};
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::prelude::{OnExit, Res, State};
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};
use bevy_rapier3d::plugin::RapierConfiguration;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States, Copy)]
pub enum AppState {
    #[default]
    StartScreenLoading,
    StartScreen,
    MainSceneLoading,
    MainScene,
    GameOver,
    ParticleTestScene,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States, Copy)]
pub enum PausedState {
    Paused,
    #[default]
    Running,
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

pub const ON_GAME_STARTED: OnEnter<AppState> = OnEnter(AppState::MainScene);
pub const GAME_CLEANUP: OnExit<AppState> = OnExit(AppState::GameOver);

#[derive(Component)]
pub struct DespawnOnCleanup;

fn cleanup_entities(
    cleanup_entities: Query<Entity, With<DespawnOnCleanup>>,
    mut commands: Commands,
) {
    for entity in &cleanup_entities {
        commands.entity(entity).despawn_recursive();
    }
}

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        // Add loading states
        for LoadingStateItem {
            loading_state,
            next_state,
        } in AppState::LOADING_STATES
        {
            app.add_loading_state(LoadingState::new(*loading_state).continue_to_state(*next_state));
        }

        app.init_state::<AppState>()
            .init_state::<PausedState>()
            .add_systems(GAME_CLEANUP, cleanup_entities)
            .add_systems(OnEnter(AppState::MainSceneLoading), cleanup_entities)
            .add_systems(OnEnter(AppState::StartScreenLoading), cleanup_entities)
            .add_plugins((
                pause::PausePlugin,
                loading_screen::LoadingScreenPlugin,
                start_screen::StartScreenPlugin,
                main_scene::MainScenePlugin,
            ));
    }
}
