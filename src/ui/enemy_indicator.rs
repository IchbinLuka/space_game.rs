use bevy::{prelude::*, render::{render_resource::PrimitiveTopology, view::RenderLayers}, sprite::MaterialMesh2dBundle};

use crate::{AppState, entities::{spaceship::{IsBot, IsPlayer, bot::{BotSpawnEvent, Bot}, player::Player}, camera::RENDER_LAYER_2D}};


#[derive(Component)]
pub struct EnemyIndicator;

fn update_enemy_indicator(
    enemies: Query<&Transform, IsBot>, 
    player: Query<&Transform, IsPlayer>,
    mut indicators: Query<
        &mut Transform, 
        (With<EnemyIndicator>, Without<Player>, Without<Bot>)
    >,
) {
    let Ok(transform) = enemies.get_single() else { return; };
    let Ok(player_transform) = player.get_single() else { return; };
    let Ok(mut indicator) = indicators.get_single_mut() else { return; };

    let dir = player_transform.translation.xz() - transform.translation.xz();

    indicator.translation = (dir.normalize() * 200.0).extend(0.); // TODO: Adapt to screen size
    indicator.rotation = Quat::from_rotation_z(dir.y.atan2(dir.x));
}

fn spawn_enemy_indicator(
    mut commands: Commands,
    res: Res<EnemyIndicatorRes>,
    mut bot_events: EventReader<BotSpawnEvent>,
) {
    for _ in bot_events.read() {
        commands.spawn((
            EnemyIndicator,
            MaterialMesh2dBundle {
                mesh: res.mesh.clone().into(),
                material: res.material.clone(),
                transform: Transform::from_scale(Vec3::splat(20.0)), 
                ..default()
            },
            RenderLayers::layer(RENDER_LAYER_2D)
        )); 
    }
}
    

fn setup_enemy_indicator(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands, 
) {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![
        [1., 0., 0.], 
        [-1., 0., 0.], 
        [0., 1., 0.]
    ]);

    let mesh = meshes.add(mesh);

    let material = materials.add(Color::RED.into());

    commands.insert_resource(EnemyIndicatorRes {
        mesh, 
        material,
    });
}

#[derive(Resource)]
struct EnemyIndicatorRes {
    mesh: Handle<Mesh>, 
    material: Handle<ColorMaterial>,
}


pub struct EnemyIndicatorPlugin;

impl Plugin for EnemyIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_enemy_indicator)
            .add_systems(Update, (
                spawn_enemy_indicator, 
                update_enemy_indicator,
            ).run_if(in_state(AppState::MainScene)));
    }
}