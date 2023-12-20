use bevy::prelude::*;
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::LoadingStateAppExt};

use crate::{Movement, AppState};

#[derive(Component)]
pub struct Player;

fn player_input(
    timer: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Movement, &mut Transform, With<Player>)>,
) {
    for (mut movement, mut transform, _) in &mut query {
        for key in keyboard_input.get_pressed() {
            match key {
                KeyCode::Up | KeyCode::W => movement.vel += transform.forward().normalize(),
                KeyCode::Down | KeyCode::S => movement.vel -= transform.forward().normalize(),
                KeyCode::Left | KeyCode::A => transform.rotate_y(3.0 * timer.delta_seconds()),
                KeyCode::Right | KeyCode::D => transform.rotate_y(-3.0 * timer.delta_seconds()),
                _ => (),
            }
        }
        if movement.vel.length() > 10.0 {
            movement.vel = movement.vel.normalize() * 10.0;
        }
    }
}

fn player_setup(
    mut commands: Commands,
    assets: Res<PlayerAssets>,
) {

    commands.spawn((
        SceneBundle {
            scene: assets.spaceship.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(0.2)), 
            ..default()
        }, 
        Player, 
        Movement {
            max_speed: Some(10.0),
            friction: 0.3,
            ..default()
        },
    ));
}

#[derive(AssetCollection, Resource)]
struct PlayerAssets {
    #[asset(path = "spaceship.glb#Scene0")]
    spaceship: Handle<Scene>
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_collection_to_loading_state::<_, PlayerAssets>(AppState::Loading)
            .add_systems(OnEnter(AppState::Running), player_setup)
            .add_systems(Update, player_input.run_if(in_state(AppState::Running)));
    }
}