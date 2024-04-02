use bevy::{app::App, ecs::system::Resource};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{
        config::{ConfigureLoadingState, LoadingStateConfig},
        LoadingStateAppExt,
    },
};

use crate::states::AppState;

pub trait AppExtension {
    fn add_collection_to_loading_states<T>(&mut self, states: &[AppState]) -> &mut Self
    where
        T: AssetCollection + Resource;
}

impl AppExtension for App {
    fn add_collection_to_loading_states<T>(&mut self, states: &[AppState]) -> &mut Self
    where
        T: AssetCollection + Resource,
    {
        for state in states {
            self.configure_loading_state(LoadingStateConfig::new(*state).load_collection::<T>());
        }
        self
    }
}
