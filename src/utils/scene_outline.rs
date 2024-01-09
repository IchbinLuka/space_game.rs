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

pub struct SceneOutlinePlugin;

impl Plugin for SceneOutlinePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, setup_scene_once_loaded);
    }
}
