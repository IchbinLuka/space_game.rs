use bevy::prelude::*;

use crate::utils::sets::Set;


#[derive(Component)]
pub struct Node3DObject {
    pub world_pos: Vec3,
}


fn node_3d_renderer_update(
    mut node_query: Query<(&mut Transform, &Node3DObject)>,
    camera_query: Query<(&GlobalTransform, &Camera), With<Camera3d>>, 
    window_query: Query<&Window>,
) {
    let Ok((camera_transform, camera)) = camera_query.get_single() else { return; };
    let Ok(window) = window_query.get_single() else { return; };

    for (mut transform, node) in &mut node_query {
        let Some(screen_pos) = camera.world_to_viewport(
            camera_transform, 
            node.world_pos, 
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