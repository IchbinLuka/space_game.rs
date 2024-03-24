pub mod loading_screen;
pub mod pause;

use bevy::app::{App, Plugin};
use bevy::ecs::schedule::common_conditions::in_state;
use bevy::ecs::schedule::States;
use bevy::prelude::{OnExit, Res, State};
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States, Copy)]
pub enum AppState {
    #[default]
    StartScreenLoading, 
    StartScreen, 
    
    MainSceneLoading,
    MainScene,
    MainPaused,
    ParticleTestScene,
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
pub fn game_running() -> impl FnMut(Res<State<AppState>>) -> bool + Clone {
    in_state(AppState::MainScene)
}

#[inline]
pub fn game_paused() -> impl FnMut(Res<State<AppState>>) -> bool + Clone {
    in_state(AppState::MainPaused)
}

#[inline]
pub fn in_start_menu() -> impl FnMut(Res<State<AppState>>) -> bool + Clone {
    in_state(AppState::StartScreen)
}

pub const ON_GAME_STARTED: OnExit<AppState> = OnExit(AppState::MainSceneLoading);

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

        app.add_state::<AppState>()
            .add_plugins((pause::PausePlugin, loading_screen::LoadingScreenPlugin));
    }
}
