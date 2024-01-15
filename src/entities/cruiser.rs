use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};
use bevy_mod_outline::OutlineBundle;
use bevy_rapier3d::dynamics::Velocity;

use crate::{
    components::colliders::VelocityColliderBundle, utils::materials::default_outline, AppState, ui::node_3d_renderer::Node3DObject,
};

#[derive(Component)]
pub struct Cruiser;

#[derive(AssetCollection, Resource)]
struct CruiserAssets {
    #[asset(path = "cruiser.glb#Scene0")]
    pub cruiser_model: Handle<Scene>,
}

fn cruiser_setup(mut commands: Commands, assets: Res<CruiserAssets>) {
    commands.spawn((
        SceneBundle {
            scene: assets.cruiser_model.clone(),
            transform: Transform::from_translation(Vec3 {
                z: 20.0,
                ..Vec3::ZERO
            }),
            ..default()
        },
        VelocityColliderBundle {
            velocity: Velocity {
                linvel: Vec3 {
                    z: -1.0,
                    ..Vec3::ZERO
                },
                ..default()
            },
            ..default()
        },
        Cruiser,
        OutlineBundle {
            outline: default_outline(),
            ..default()
        }, 
    ));

    commands.spawn((NodeBundle {
        transform: Transform::from_translation(Vec3::Y * 10.0), 
        ..default()
    }, Node3DObject {
        world_pos: Vec3::ZERO
    })).with_children(|c| {
        c.spawn(TextBundle {
            text: Text {
                sections: vec![
                    TextSection {
                        value: "Cruiser".to_string(),
                        style: TextStyle {
                            font_size: 20.0,
                            color: Color::WHITE,
                            ..default()
                        },
                    },
                ], 
                ..default()
            },
            ..default()
        });
    });


    // commands.entity(cruiser).push_children(&[label]);
}

pub struct CruiserPLugin;

impl Plugin for CruiserPLugin {
    fn build(&self, app: &mut App) {
        app.add_collection_to_loading_state::<_, CruiserAssets>(AppState::MainSceneLoading)
            .add_systems(OnEnter(AppState::MainScene), cruiser_setup);
    }
}
