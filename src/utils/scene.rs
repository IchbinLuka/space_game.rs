use std::{marker::PhantomData, sync::Arc};

use bevy::{prelude::*, scene::SceneInstance, utils::HashMap};
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
    scene_query: Query<(&SceneInstance, Entity), Added<SceneInstance>>,
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

#[derive(Component)]
pub struct MaterialsApplied<M> {
    phantom: PhantomData<M>,
}

impl<M> Default for MaterialsApplied<M> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

fn replace_materials<Id: Component, M: Material>(
    mut commands: Commands,
    scene_query: Query<
        (Entity, &SceneInstance),
        (With<Id>, Added<SceneInstance>, Without<MaterialsApplied<M>>),
    >,
    scene_manager: Res<SceneSpawner>,
    name_query: Query<(&Name, &Handle<StandardMaterial>)>,
    standard_materials: Res<Assets<StandardMaterial>>,
    mut materials: ResMut<Assets<M>>,
    mut library: ResMut<MaterialLibrary<Id, M>>,
) {
    for (entity, scene) in &scene_query {
        if !scene_manager.instance_is_ready(**scene) {
            continue;
        }
        for entity in scene_manager.iter_instance_entities(**scene) {
            let Ok((name, handle)) = name_query.get(entity) else {
                continue;
            };
            let Some(material) = standard_materials.get(handle) else {
                continue;
            };

            let material = if let Some(handle) = library.materials.get(name.as_str()) {
                // If the material has already been created, we can resuse it for better performance
                handle.clone()
            } else if let Some(m) = (library.builder)(name, material) {
                let handle = materials.add(m);
                library
                    .materials
                    .insert(name.as_str().to_string(), handle.clone());
                handle
            } else {
                continue;
            };

            commands
                .entity(entity)
                .remove::<Handle<StandardMaterial>>()
                .insert(material);
        }
        commands
            .entity(entity)
            .insert(MaterialsApplied::<M>::default());
    }
}

pub struct ReplaceMaterialPlugin<Id, M>
where
    Id: Component,
    M: Material,
{
    phantom: PhantomData<Id>,
    material_builder: Arc<MaterialBuilder<M>>,
}

pub type MaterialBuilder<M> = dyn Fn(&Name, &StandardMaterial) -> Option<M> + Send + Sync;

impl<Id, M> ReplaceMaterialPlugin<Id, M>
where
    Id: Component,
    M: Material,
{
    pub fn new(builder: Box<MaterialBuilder<M>>) -> Self {
        Self {
            phantom: PhantomData,
            material_builder: builder.into(),
        }
    }
}

impl<Id, M> Plugin for ReplaceMaterialPlugin<Id, M>
where
    Id: Component,
    M: Material,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(MaterialLibrary::<Id, M> {
            materials: HashMap::default(),
            builder: self.material_builder.clone(),
            phantom: PhantomData,
        })
        .add_systems(
            PostUpdate,
            replace_materials::<Id, M>.run_if(any_with_component::<Id>),
        );
    }
}

#[derive(Resource)]
struct MaterialLibrary<Id: Component, M: Material> {
    materials: HashMap<String, Handle<M>>,
    builder: Arc<MaterialBuilder<M>>,
    phantom: PhantomData<Id>,
}

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, (setup_scene_once_loaded, setup_animation_root));
    }
}
