use bevy::prelude::*;

use crate::utils::sets::Set;


#[derive(Component)]
pub struct Sprite3DObject {
    pub parent: Entity,
}


fn node_3d_renderer_update(
    mut node_query: Query<(&Sprite3DObject, &mut Transform, Entity)>,
    transform_query: Query<&GlobalTransform, Without<Camera>>,
    camera_query: Query<(&GlobalTransform, &Camera), With<Camera3d>>, 
    window_query: Query<&Window>,
    mut commands: Commands, 
) {
    let Ok((camera_transform, camera)) = camera_query.get_single() else { return; };
    let Ok(window) = window_query.get_single() else { return; };

    for (node, mut transform, entity) in &mut node_query {
        let Ok(global) = transform_query.get(node.parent) else {
            warn!("Entity of Sprite3DObject must exist and have a GlobalTransform component. Despawning entity...");
            commands.entity(entity).despawn_recursive();
            continue;
        };
        
        let Some(screen_pos) = camera.world_to_viewport(
            camera_transform, 
            global.compute_transform().translation, 
        ) else {
            warn!("Could not get viewport position for node");
            continue;
        };
        
        transform.translation = Vec3::new(
            screen_pos.x - window.width() / 2.0, 
            -screen_pos.y + window.height() / 2.0, 
            0.0
        );
        // info!("Node position: {:?}", screen_pos);
    }
}


pub struct Sprite3DRendererPlugin;

impl Plugin for Sprite3DRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            node_3d_renderer_update.after(Set::CameraMovement), 
        ));
    }
}