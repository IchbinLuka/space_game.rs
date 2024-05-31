use bevy::{
    ecs::system::Command,
    prelude::*,
    render::{mesh::PrimitiveTopology, render_asset::RenderAssetUsages, view::RenderLayers},
    sprite::{Anchor, MaterialMesh2dBundle},
};
use bevy_asset_loader::{asset_collection::AssetCollection, loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt}};
use bevy_round_ui::{
    autosize::RoundUiAutosizeMaterial,
    prelude::{RoundUiBorder, RoundUiMaterial},
};

use crate::{
    components::health::Health,
    entities::{
        camera::RENDER_LAYER_2D,
        spaceship::{
            bot::Bot,
            player::{Player, PlayerInventory, PlayerRespawnTimer},
            IsPlayer, Spaceship,
        },
    },
    states::{game_running, AppState, DespawnOnCleanup, ON_GAME_STARTED},
    utils::{misc::cleanup_system, sets::Set},
};

use super::{fonts::FontsResource, theme::text_body_style};

#[derive(Component)]
pub struct HudRootNode;

#[derive(Component)]
pub struct BombCounter;

#[derive(Component)]
struct HealthBarContent;

fn main_hud_setup(
    mut commands: Commands,
    font_resource: Res<FontsResource>,
    mut materials: ResMut<Assets<RoundUiMaterial>>,
) {
    let root = commands
        .spawn((
            HudRootNode,
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    padding: UiRect::all(Val::Px(10.)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
            DespawnOnCleanup,
        ))
        .id();

    commands.insert_resource(Score(0));

    let score = commands
        .spawn((
            TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: t!("score", score = 0).to_string(),
                        style: TextStyle {
                            font_size: 60.0,
                            font: font_resource.mouse_memoirs.clone(),
                            ..default()
                        },
                    }],
                    ..default()
                },
                ..default()
            },
            ScoreCounter,
        ))
        .id();

    let bottom_section = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::End,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.),
                ..default()
            },
            ..default()
        })
        .id();

    commands
        .entity(root)
        .add_child(score)
        .add_child(bottom_section);

    let auxiliary_drive_status = commands
        .spawn((
            TextBundle {
                text: Text::from_section(
                    t!("auxiliary_drive", state = t!("state_off")),
                    TextStyle {
                        font_size: 60.0,
                        font: font_resource.mouse_memoirs.clone(),
                        ..default()
                    },
                ),
                ..default()
            },
            RoundUiAutosizeMaterial,
            AuxiliaryDriveUI,
        ))
        .id();

    const PANEL_WIDTH: f32 = 400.;
    const PANEL_HEIGHT: f32 = 40.;
    const PADDING: f32 = 5.;
    
    let bomb_counter = commands.spawn((
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                padding: UiRect::all(Val::Px(PADDING)),
                ..default()
            }, 
            ..default()
        }, 
        BombCounter, 
    )).id();

    let health_bar = commands
        .spawn((MaterialNodeBundle {
            style: Style {
                width: Val::Px(PANEL_WIDTH),
                height: Val::Px(PANEL_HEIGHT),
                padding: UiRect::all(Val::Px(PADDING)),
                ..default()
            },
            material: materials.add(RoundUiMaterial {
                background_color: Color::BLACK,
                border_radius: RoundUiBorder::all(PANEL_HEIGHT / 2.).into(),
                size: Vec2::new(PANEL_WIDTH, PANEL_HEIGHT),
                ..default()
            }),
            ..default()
        },))
        .with_children(|p| {
            p.spawn((
                MaterialNodeBundle {
                    material: materials.add(RoundUiMaterial {
                        background_color: Color::hex("#ef4d34").unwrap(),
                        border_radius: RoundUiBorder::all((PANEL_HEIGHT - PADDING * 2.) / 2.)
                            .into(),
                        ..default()
                    }),
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        ..default()
                    },
                    ..default()
                },
                RoundUiAutosizeMaterial,
                HealthBarContent,
            ));
        })
        .id();

    let bottom_left = commands.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            ..default()
        }, 
        ..default()
    }).id();

    commands.entity(bottom_left)
        .add_child(bomb_counter)
        .add_child(health_bar);

    commands
        .entity(bottom_section)
        .add_child(bottom_left)
        .add_child(auxiliary_drive_status);
}

fn health_bar_update(
    player_query: Query<&Health, (IsPlayer, Changed<Health>)>,
    mut health_bar_query: Query<&mut Style, With<HealthBarContent>>,
) {
    let Ok(player_health) = player_query.get_single() else {
        return;
    };
    for mut style in &mut health_bar_query {
        style.width = Val::Percent(player_health.health / player_health.max_health * 100.);
    }
}

fn bomb_counter_update(
    bomb_counter: Query<Entity, With<BombCounter>>,
    player_inventory: Res<PlayerInventory>,
    mut commands: Commands, 
    ui_assets: Res<UiAssets>,
) {
    for entity in &bomb_counter {
        commands.entity(entity)
            .despawn_descendants();
        for _ in 0..player_inventory.bombs {
            let child = commands.spawn((
                NodeBundle {
                    background_color: Color::WHITE.into(),
                    style: Style {
                        width: Val::Px(40.),
                        height: Val::Px(40.),
                        ..default()
                    }, 
                    ..default()
                }, 
                UiImage::new(ui_assets.bomb_icon.clone()), 
            )).id();
            commands.entity(entity).add_child(child);
        }
    }
}

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
            DespawnOnCleanup,
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
        (Without<Player>, Without<Bot>),
    >,
    mut commands: Commands,
) {
    const MAX_SCALE: f32 = 20.0;

    let Ok(player_transform) = player.get_single() else {
        return;
    };
    for (mut indicator_transform, indicator, entity) in &mut indicators {
        let Ok(transform) = transform_query.get(indicator.enemy) else {
            commands.entity(entity).despawn_recursive();
            continue;
        };

        let mut dir = player_transform.translation.xz() - transform.translation.xz();
        dir.x *= -1.;

        indicator_transform.translation = (dir.normalize() * 200.0).extend(0.);
        indicator_transform.rotation = Quat::from_rotation_z(dir.y.atan2(dir.x));
        indicator_transform.scale = Vec3::splat((MAX_SCALE - dir.length() * 0.1).max(0.));
    }
}

fn setup_enemy_indicator(
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[0., 0.5, 0.], [0., -0.5, 0.], [1., 0., 0.]],
    );

    let mesh = meshes.add(mesh);

    let material = materials.add(Color::RED);

    commands.insert_resource(EnemyIndicatorRes { mesh, material });
}

#[derive(Resource)]
pub struct EnemyIndicatorRes {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

#[derive(Component)]
pub struct AuxiliaryDriveUI;

fn auxiliary_drive_update(
    mut query: Query<&mut Text, (Without<Player>, With<AuxiliaryDriveUI>)>,
    player_query: Query<&Spaceship, (Changed<Spaceship>, IsPlayer)>,
) {
    for mut text in query.iter_mut() {
        for player in player_query.iter() {
            text.sections[0].value = t!(
                "auxiliary_drive",
                state = if player.auxiliary_drive {
                    t!("state_on")
                } else {
                    t!("state_off")
                }
            )
            .to_string();
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct Score(pub u32);

#[derive(Event)]
pub struct ScoreEvent {
    pub score: u32,
    pub world_pos: Vec3,
}

#[derive(Component)]
pub struct ScoreElement {
    pub score: u32,
}

#[derive(Component)]
pub struct ScoreCounter;

fn score_events(
    mut score_events: EventReader<ScoreEvent>,
    mut commands: Commands,
    camera_query: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    window: Query<&Window>,
    font_resource: Res<FontsResource>,
) {
    let Ok((transform, camera)) = camera_query.get_single() else {
        return;
    };
    let Ok(window) = window.get_single() else {
        return;
    };

    let screen_size = Vec2::new(window.width(), window.height());

    for event in score_events.read() {
        let Some(screen_pos) = camera.world_to_viewport(transform, event.world_pos) else {
            warn!("Could not get viewport position for node");
            continue;
        };

        let pos = Vec2::new(
            screen_pos.x - screen_size.x / 2.0,
            -screen_pos.y + screen_size.y / 2.0,
        );

        commands.spawn((
            DespawnOnCleanup,
            Text2dBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: format!("+{}", event.score),
                        style: TextStyle {
                            font_size: 40.0,
                            color: Color::WHITE,
                            font: font_resource.mouse_memoirs.clone(),
                        },
                    }],
                    ..default()
                },
                text_anchor: Anchor::Center,
                transform: Transform::from_translation(pos.extend(0.)),
                ..default()
            },
            ScoreElement { score: event.score },
            RenderLayers::layer(RENDER_LAYER_2D),
        ));
    }
}

fn score_element_update(
    mut score_query: Query<(&mut Transform, &ScoreElement, Entity)>,
    time: Res<Time>,
    window: Query<&Window>,
    mut commands: Commands,
    mut score: ResMut<Score>,
) {
    const UI_ELEMENT_SPEED: f32 = 500.0;

    let Ok(window) = window.get_single() else {
        return;
    };

    let counter_location = Vec2::new(0.0, window.height() / 2.0);

    for (mut transform, score_element, entity) in &mut score_query {
        let delta = counter_location - transform.translation.xy();

        if delta.length() < 20.0 {
            score.0 += score_element.score;
            commands.entity(entity).despawn_recursive();
            continue;
        }

        let speed = delta.normalize() * UI_ELEMENT_SPEED;

        transform.translation += Vec3 {
            x: speed.x,
            y: speed.y,
            z: 0.0,
        } * time.delta_seconds();
    }
}

fn score_update(mut score_query: Query<&mut Text, With<ScoreCounter>>, score: Res<Score>) {
    if !score.is_changed() {
        return;
    }
    for mut text in &mut score_query {
        text.sections[0].value = t!("score", score = score.0).to_string();
    }
}

#[derive(Component)]
struct RespawnTimerUIParent;

#[derive(Component)]
struct RespawnTimerUI;

fn respawn_ui_setup(mut commands: Commands, font_res: Res<FontsResource>) {
    commands
        .spawn((
            RespawnTimerUIParent,
            DespawnOnCleanup,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|c| {
            c.spawn((
                RespawnTimerUI,
                TextBundle::from_section(
                    t!("respawning_in", time = 0),
                    TextStyle {
                        font_size: 70.,
                        ..text_body_style(&font_res)
                    },
                ),
            ));
        });
}

fn respawn_ui_update(
    mut respawn_ui: Query<&mut Text, With<RespawnTimerUI>>,
    timer: Res<PlayerRespawnTimer>,
) {
    for mut text in &mut respawn_ui {
        text.sections[0].value = t!(
            "respawning_in",
            time = timer.0.remaining_secs().ceil() as u32
        )
        .to_string();
    }
}

#[derive(AssetCollection, Resource)]
pub struct UiAssets {
    #[asset(path = "textures/bomb_icon.png")]
    bomb_icon: Handle<Image>,
}


pub struct GameHudPlugin;

impl Plugin for GameHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ScoreEvent>()
            .add_systems(
                OnExit(AppState::MainScene),
                (
                    cleanup_system::<RespawnTimerUIParent>,
                    cleanup_system::<ScoreElement>,
                ),
            )
            .add_systems(Startup, setup_enemy_indicator)
            .add_systems(ON_GAME_STARTED, (main_hud_setup,))
            .add_loading_state(LoadingState::new(AppState::MainSceneLoading).load_collection::<UiAssets>())
            .add_systems(
                Update,
                (
                    health_bar_update,
                    update_enemy_indicator,
                    auxiliary_drive_update,
                    score_events.in_set(Set::ScoreEvents),
                    score_element_update,
                    score_update,
                    bomb_counter_update.run_if(resource_changed::<PlayerInventory>),
                    respawn_ui_setup.run_if(resource_added::<PlayerRespawnTimer>),
                    cleanup_system::<RespawnTimerUIParent>
                        .run_if(resource_removed::<PlayerRespawnTimer>()),
                    respawn_ui_update.run_if(resource_exists::<PlayerRespawnTimer>),
                )
                    .run_if(game_running()),
            );
    }
}
