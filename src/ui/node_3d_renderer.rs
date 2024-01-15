use bevy::prelude::*;


#[derive(Component)]
pub struct Node3DObject {
    pub world_pos: Vec3,
}


fn node_3d_renderer_update(
    mut node_query: Query<(&mut Style, &Node3DObject), (With<Node>, )>,
    camera_query: Query<(&GlobalTransform, &Camera)>
) {
    let Ok((transform, camera)) = camera_query.get_single() else { return; };

    for (mut style, node) in &mut node_query {
        let Some(screen_pos) = camera.world_to_viewport(
            transform, 
            node.world_pos, 
        ) else {
            warn!("Could not get viewport position for node");
            continue;
        };
        
        style.left = Val::Px(screen_pos.x);
        style.top = Val::Px(screen_pos.y);

        info!("Node position: {:?}", screen_pos);
    }
}


pub struct Node3DRendererPlugin;

impl Plugin for Node3DRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            node_3d_renderer_update, 
        ));
    }
}