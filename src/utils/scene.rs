use bevy::{prelude::*, scene::SceneInstance};
use bevy_mod_outline::{InheritOutlineBundle, OutlineStencil};

#[derive(Component)]
pub struct OutlineApplied;

fn setup_scene_once_loaded(
    mut commands: Commands,
    scene_query: Query<
        (&SceneInstance, Entity),
        (
            Changed<SceneInstance>,
            With<OutlineStencil>,
            Without<OutlineApplied>,
        ),
    >,
    scene_manager: Res<SceneSpawner>,
) {
    for (scene, entity) in &scene_query {
        if scene_manager.instance_is_ready(**scene) {
            for entity in scene_manager.iter_instance_entities(**scene) {
                commands
                    .entity(entity)
                    .insert(InheritOutlineBundle::default());
            }
            commands.entity(entity).insert(OutlineApplied);
        }
    }
}

#[derive(Component, Default)]
pub struct AnimationRoot {
    pub player_entites: Vec<Entity>,
}

fn setup_animation_root(
    mut commands: Commands, 
    scene_query: Query<
        (&SceneInstance, Entity),
        Added<SceneInstance>,
    >,
    animation_players: Query<Entity, With<AnimationPlayer>>,
    scene_manager: Res<SceneSpawner>,
) {
    for (scene, entity) in &scene_query {
        let mut animation_root = AnimationRoot::default();
        if scene_manager.instance_is_ready(**scene) {
            for entity in scene_manager.iter_instance_entities(**scene) {
                let Ok(entity) = animation_players.get(entity) else {
                    continue;
                };
                animation_root.player_entites.push(entity);
            }
        }
        if !animation_root.player_entites.is_empty() {
            commands.entity(entity).insert(animation_root);
        }
    }
}


pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            setup_scene_once_loaded, 
            setup_animation_root, 
        ));
    }
}