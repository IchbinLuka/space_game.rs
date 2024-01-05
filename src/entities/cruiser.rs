use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_rapier3d::dynamics::Velocity;

use crate::{AppState, components::colliders::VelocityColliderBundle, utils::materials::default_outline};

#[derive(Component)]
pub struct Cruiser;


#[derive(AssetCollection, Resource)]
struct CruiserAssets {
    #[asset(path = "cruiser.glb#Scene0")]
    pub cruiser_model: Handle<Scene>,
}

fn cruiser_setup(
    mut commands: Commands,
    assets: Res<CruiserAssets>,
) {
    commands.spawn((
        SceneBundle {
            scene: assets.cruiser_model.clone(),
            transform: Transform::from_translation(Vec3 { z: 20.0, ..Vec3::ZERO }), 
            ..default()
        }, 
        VelocityColliderBundle {
            velocity: Velocity {
                linvel: Vec3 { z: -1.0, ..Vec3::ZERO}, 
                ..default()
            }, 
            ..default()
        },
        Cruiser,
        default_outline(), 
    ));
}




pub struct CruiserPLugin;

impl Plugin for CruiserPLugin {
    fn build(&self, app: &mut App) {
        app
            .add_collection_to_loading_state::<_, CruiserAssets>(AppState::Loading)
            .add_systems(OnEnter(AppState::Running), cruiser_setup);
    }
}