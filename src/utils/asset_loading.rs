use bevy::{app::App, ecs::system::Resource};
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};

use crate::states::AppState;

trait _AppExtension {
    fn add_collection_to_loading_states<T>(&mut self, states: &[AppState]) -> &mut Self
    where
        T: AssetCollection + Resource;
}

impl _AppExtension for App {
    fn add_collection_to_loading_states<T>(&mut self, states: &[AppState]) -> &mut Self
    where
        T: AssetCollection + Resource,
    {
        for state in states {
            self.add_collection_to_loading_state::<_, T>(*state);
        }
        self
    }
}
