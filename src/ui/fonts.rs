use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};

use crate::AppState;

#[derive(Resource, AssetCollection)]
pub struct FontsResource {
    #[asset(path = "fonts/MouseMemoirs-Regular.ttf")]
    pub mouse_memoirs: Handle<Font>,
}

pub struct FontsPlugin;

impl Plugin for FontsPlugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, FontsResource>(AppState::MainSceneLoading);
    }
}