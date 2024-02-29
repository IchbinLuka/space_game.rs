use bevy::prelude::*;
use bevy_asset_loader::asset_collection::{AssetCollection, AssetCollectionApp};

#[derive(Resource, AssetCollection)]
pub struct FontsResource {
    #[asset(path = "fonts/MouseMemoirs-Regular.ttf")]
    pub mouse_memoirs: Handle<Font>,
}

pub struct FontsPlugin;

impl Plugin for FontsPlugin {
    fn build(&self, app: &mut App) {
        // Do not add collection to loading state as we need it in the loading screen
        app.init_collection::<FontsResource>();
    }
}
