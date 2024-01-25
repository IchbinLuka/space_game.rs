

use bevy::{ecs::system::Command, prelude::*, render::{render_resource::PrimitiveTopology, view::RenderLayers}, sprite::MaterialMesh2dBundle};

use crate::{AppState, entities::{spaceship::{IsPlayer, bot::Bot, player::Player}, camera::RENDER_LAYER_2D}};


#[derive(Component)]
pub struct EnemyIndicator {
    enemy: Entity,
}

pub struct SpawnEnemyIndicator {
    pub enemy: Entity,
}

impl Command for SpawnEnemyIndicator {
    fn apply(self, world: &mut World) {
        let Some(res) = world.get_resource::<EnemyIndicatorRes>() else {
            error!("Enemy indicator resources not loaded");
            return;
        };

        world.spawn((
            EnemyIndicator { enemy: self.enemy },
            MaterialMesh2dBundle {
                mesh: res.mesh.clone().into(),
                material: res.material.clone(),
                transform: Transform::from_scale(Vec3::splat(20.0)), 
                ..default()
            },
            RenderLayers::layer(RENDER_LAYER_2D),
        ));
    }
}

#[derive(Bundle)]
pub struct EnemyIndicatorBundle {
    enemy_indicator: EnemyIndicator,
    material_mesh: MaterialMesh2dBundle<ColorMaterial>,
    render_layer: RenderLayers, 
}

fn update_enemy_indicator(
    transform_query: Query<&Transform, (Without<Player>, Without<EnemyIndicator>)>, 
    player: Query<&Transform, IsPlayer>,
    mut indicators: Query<
        (&mut Transform, &EnemyIndicator, Entity), 
        (Without<Player>, Without<Bot>)
    >,
    mut commands: Commands, 
) {
    const MAX_SCALE: f32 = 20.0;
    
    let Ok(player_transform) = player.get_single() else { return; };
    for (mut indicator_transform, indicator, entity) in &mut indicators {
        let Ok(transform) = transform_query.get(indicator.enemy) else { 
            commands.entity(entity).despawn_recursive();
            continue;
        };
        
        let mut dir = player_transform.translation.xz() - transform.translation.xz();
        dir.x *= -1.;

        indicator_transform.translation = (dir.normalize() * 200.0).extend(0.); // TODO: Adapt to screen size
        indicator_transform.rotation = Quat::from_rotation_z(dir.y.atan2(dir.x));
        indicator_transform.scale = Vec3::splat((MAX_SCALE - dir.length() * 0.1).max(0.));
    }
}
    

fn setup_enemy_indicator(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands, 
) {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![
        [0., 0.5, 0.], 
        [0., -0.5, 0.], 
        [1., 0., 0.]
    ]);

    let mesh = meshes.add(mesh);

    let material = materials.add(Color::RED.into());

    commands.insert_resource(EnemyIndicatorRes {
        mesh, 
        material,
    });
}

#[derive(Resource)]
pub struct EnemyIndicatorRes {
    mesh: Handle<Mesh>, 
    material: Handle<ColorMaterial>,
}


pub struct EnemyIndicatorPlugin;

impl Plugin for EnemyIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_enemy_indicator)
            .add_systems(Update, (
                update_enemy_indicator,
            ).run_if(in_state(AppState::MainScene)));
    }
}