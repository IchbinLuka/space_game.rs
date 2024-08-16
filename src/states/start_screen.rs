use std::f32::consts::FRAC_PI_4;

use bevy::{
    ecs::system::{Command, EntityCommands, RunSystemOnce},
    prelude::*,
};
use bevy_round_ui::{
    autosize::{RoundUiAutosizeMaterial, RoundUiAutosizeNode},
    prelude::RoundUiMaterial,
};
use bevy_simple_text_input::{TextInputBundle, TextInputInactive, TextInputValue};

use crate::{
    entities::{
        camera::{spawn_camera, CameraAssets},
        planet::{spawn_planet, PlanetSpawnConfig},
        space_station::{spawn_space_station, SpaceStationRes},
    },
    model::settings::Settings,
    states::{in_start_menu, AppState},
    ui::{
        fonts::FontsResource,
        leaderboard::{AddLeaderboardExtension, FetchLeaderboardRequest},
        minimap::MinimapAssets,
        settings::SettingsButton,
        theme::{
            text_body_style, text_button_small_style, text_button_style, text_title_style,
            SURFACE_COLOR, SURFACE_COLOR_FOCUSED,
        },
        widgets::{CardBundle, FocusTextInputOnInteraction, TextButtonBundle, TextInputDisabled},
        UiRes,
    },
    utils::{
        api::{ApiManager, Token},
        clipboard::Clipboard,
        misc::AsCommand,
        sets::Set,
        tasks::StartJob,
    },
};

use super::{DespawnOnCleanup, StartScreenState};

#[derive(Component)]
struct StartScreen;

#[derive(Component)]
struct StartButton;

#[derive(Component)]
pub struct LeaderboardButton;

const SPACE_STATION_POS: Vec3 = Vec3::new(0., 0., 150.);

fn setup_start_screen(
    mut commands: Commands,
    camera_assets: Res<CameraAssets>,
    settings: Res<Settings>,
    space_station_res: Res<SpaceStationRes>,
    minimap_res: Res<MinimapAssets>,
) {
    let camera_transform = Transform {
        // rotation: Quat::from_rotation_x(FRAC_P),
        translation: Vec3::new(0.0, 20.0, 200.0),
        ..default()
    }
    .looking_at(SPACE_STATION_POS, Vec3::Y);
    spawn_camera(&mut commands, camera_transform, &camera_assets);

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.5,
    });

    let mut transform = Transform::from_xyz(0.0, 40.0, 0.0);
    // transform.rotate_x(-FRAC_PI_2);
    transform.rotate_y(FRAC_PI_4 * 0.7);

    commands.spawn((
        DespawnOnCleanup,
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 10000.0,
                color: Color::hex("ffffff").unwrap(),
                shadows_enabled: settings.shadows_enabled,
                ..default()
            },
            transform,
            ..default()
        },
    ));

    for config in [
        PlanetSpawnConfig {
            color: Color::hex("ffffff").unwrap(),
            size: 30.0,
            pos: Vec3::new(-60.0, -10.0, 100.0),
        },
        PlanetSpawnConfig {
            color: Color::hex("11ff22").unwrap(),
            size: 25.0,
            pos: Vec3::new(90.0, -20.0, 30.0),
        },
        PlanetSpawnConfig {
            color: Color::hex("3300ff").unwrap(),
            size: 50.0,
            pos: Vec3::new(0.0, -20.0, 0.0),
        },
    ] {
        commands.add(spawn_planet.to_command(config));
    }

    spawn_space_station(
        &mut commands,
        &space_station_res,
        &minimap_res,
        SPACE_STATION_POS,
        false,
    );
}

fn setup_startscreen_ui(
    mut materials: ResMut<Assets<RoundUiMaterial>>,
    font_res: Res<FontsResource>,
    root: Query<Entity, With<StartScreen>>,
    mut commands: Commands,
) {
    const MENU_ITEM_SIZE: Vec2 = Vec2::new(200., 50.);

    let menu_resource = MenuItemResource {
        hover_material: materials.add(RoundUiMaterial {
            background_color: SURFACE_COLOR_FOCUSED,
            border_radius: Vec4::splat(30.),
            size: MENU_ITEM_SIZE,
            ..default()
        }),
        normal_material: materials.add(RoundUiMaterial {
            background_color: SURFACE_COLOR,
            border_radius: Vec4::splat(30.),
            size: MENU_ITEM_SIZE,
            ..default()
        }),
    };

    let root = if let Ok(root) = root.get_single() {
        root
    } else {
        commands
            .spawn((
                DespawnOnCleanup,
                StartScreen,
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceAround,
                        ..default()
                    },
                    ..default()
                },
            ))
            .id()
    };

    commands.entity(root).with_children(|c| {
        c.spawn((
            MaterialNodeBundle {
                style: Style {
                    padding: UiRect::all(Val::Px(20.)),
                    ..default()
                },
                material: materials.add(RoundUiMaterial {
                    background_color: SURFACE_COLOR,
                    border_radius: Vec4::splat(30.),
                    ..default()
                }),
                ..default()
            },
            RoundUiAutosizeMaterial,
        ))
        .with_children(|c| {
            c.spawn(TextBundle::from_section(
                "Space Game",
                text_title_style(&font_res),
            ));
        });

        c.menu_item(&menu_resource)
            .with_children(|c| {
                c.spawn(TextBundle::from_section(
                    t!("leaderboard"),
                    text_button_style(&font_res),
                ));
            })
            .insert(LeaderboardButton);

        c.menu_item(&menu_resource)
            .with_children(|c| {
                c.spawn(TextBundle::from_section(
                    t!("settings"),
                    text_button_style(&font_res),
                ));
            })
            .insert(SettingsButton);

        c.menu_item(&menu_resource)
            .with_children(|c| {
                c.spawn(TextBundle::from_section(
                    t!("start_game"),
                    text_button_style(&font_res),
                ));
            })
            .insert(StartButton);
    });
    commands.insert_resource(menu_resource);
}

#[derive(Component)]
struct MenuItem;

trait ChildBuilderExtension {
    fn menu_item<'w>(&'w mut self, materials: &MenuItemResource) -> EntityCommands<'w>;
}

impl ChildBuilderExtension for ChildBuilder<'_> {
    fn menu_item<'w>(&'w mut self, materials: &MenuItemResource) -> EntityCommands<'w> {
        self.spawn((
            MenuItem,
            MaterialNodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceAround,
                    ..default()
                },
                material: materials.normal_material.clone(),
                ..default()
            },
            RoundUiAutosizeNode,
            Interaction::default(),
        ))
    }
}

fn menu_item_hover_effect(
    mut query: Query<
        (&Interaction, &mut Handle<RoundUiMaterial>),
        (With<MenuItem>, Changed<Interaction>),
    >,
    menu_item_res: Res<MenuItemResource>,
) {
    for (interaction, mut material) in &mut query {
        match *interaction {
            Interaction::Hovered => {
                *material = menu_item_res.hover_material.clone();
            }
            Interaction::None => {
                *material = menu_item_res.normal_material.clone();
            }
            _ => {}
        }
    }
}

#[derive(Resource)]
struct MenuItemResource {
    hover_material: Handle<RoundUiMaterial>,
    normal_material: Handle<RoundUiMaterial>,
}

fn start_game(
    start_button: Query<&Interaction, (With<StartButton>, Changed<Interaction>)>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for interaction in &start_button {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::MainSceneLoading);
        }
    }
}

#[derive(Component)]
struct CopyTokenButton;

#[derive(Component)]
struct BackButton;

#[derive(Component)]
struct EnterTokenInput;

#[derive(Component)]
struct SaveTokenButton;

#[derive(Component)]
struct ResetTokenButton;

fn setup_leaderboard_screen(
    mut commands: Commands,
    api_manager: Res<ApiManager>,
    font_res: Res<FontsResource>,
    settings: Res<Settings>,
    root_node: Query<Entity, With<StartScreen>>,
    menu_res: Res<MenuItemResource>,
    ui_res: Res<UiRes>,
) {
    let Ok(root_node) = root_node.get_single() else {
        return;
    };
    commands.entity(root_node).with_children(|c| {
        c.spawn(CardBundle::new(&ui_res).with_style(Style {
            flex_direction: FlexDirection::Row,
            width: Val::Px(600.),
            padding: UiRect::all(Val::Px(20.)),
            align_items: AlignItems::Center,
            ..default()
        }))
        .with_children(|c| {
            let body_style = text_body_style(&font_res);
            if let Some(profile) = &settings.profile {
                c.spawn(TextBundle::from_section(
                    profile.token.0.clone(),
                    body_style.clone(),
                ));
                c.spawn(NodeBundle {
                    style: Style {
                        flex_grow: 1.,
                        ..default()
                    },
                    ..default()
                });
                c.spawn((
                    TextButtonBundle::from_section(t!("copy"), text_button_small_style(&font_res)),
                    CopyTokenButton,
                ))
                .insert(Style {
                    margin: UiRect::right(Val::Px(10.)),
                    ..default()
                });
                c.spawn((
                    TextButtonBundle::from_section(t!("reset"), text_button_small_style(&font_res)),
                    ResetTokenButton,
                ));
            } else {
                c.spawn((
                    NodeBundle {
                        style: Style {
                            flex_grow: 1.,
                            height: Val::Px(body_style.font_size),
                            ..default()
                        },
                        ..default()
                    },
                    TextInputBundle::default()
                        .with_placeholder(t!("enter_token"), Some(body_style.clone()))
                        .with_text_style(body_style.clone())
                        .with_inactive(true),
                    EnterTokenInput,
                    FocusTextInputOnInteraction,
                ));
                c.spawn((
                    TextButtonBundle::from_section(t!("save"), text_button_small_style(&font_res)),
                    SaveTokenButton,
                ));
            }
        });
        let (request, num) = match &settings.profile {
            Some(profile) => (
                FetchLeaderboardRequest::NearPlayer {
                    token: profile.token.clone(),
                },
                5,
            ),
            None => (FetchLeaderboardRequest::BestPlayers, 10),
        };
        c.spawn(CardBundle::new(&ui_res).with_style(Style {
            width: Val::Px(400.),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(20.)),
            ..default()
        }))
        .with_children(|c| c.add_leaderboard(request, num, api_manager.clone(), &font_res));

        c.menu_item(&menu_res)
            .insert(BackButton)
            .with_children(|c| {
                c.spawn(TextBundle::from_section(
                    t!("back"),
                    text_button_style(&font_res),
                ));
            });
    });
}

fn clear_screen(mut commands: Commands, start_screen: Query<Entity, With<StartScreen>>) {
    let Ok(entity) = start_screen.get_single() else {
        error!("No entity with StartScreen component exists");
        return;
    };

    commands.entity(entity).despawn_descendants();
}

fn open_leaderboard(
    mut next_state: ResMut<NextState<StartScreenState>>,
    query: Query<&Interaction, (With<LeaderboardButton>, Changed<Interaction>)>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            next_state.set(StartScreenState::Leaderboard);
        }
    }
}

fn back_button(
    mut next_state: ResMut<NextState<StartScreenState>>,
    query: Query<&Interaction, (With<BackButton>, Changed<Interaction>)>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            next_state.set(StartScreenState::Menu);
        }
    }
}

fn copy_token(
    mut clipboard: ResMut<Clipboard>,
    settings: Res<Settings>,
    query: Query<&Interaction, (With<CopyTokenButton>, Changed<Interaction>)>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            if let Some(profile) = &settings.profile {
                clipboard.set_contents(profile.token.0.clone());
            }
        }
    }
}

fn save_token(
    mut text_fields: Query<
        (&TextInputValue, &mut TextInputInactive, Entity),
        With<EnterTokenInput>,
    >,
    save_button: Query<&Interaction, (With<SaveTokenButton>, Changed<Interaction>)>,
    mut commands: Commands,
    api_manager: Res<ApiManager>,
) {
    for interaction in &save_button {
        if *interaction == Interaction::Pressed {
            let Ok((value, mut inactive, entity)) = text_fields.get_single_mut() else {
                return;
            };
            let token = Token(value.0.clone());

            // settings.api_token = Some(Token(value.0.clone()));

            let api_manager = api_manager.clone();

            commands.add(StartJob {
                job: Box::pin(async move { api_manager.get_profile(&token).await }),
                on_complete: |result, world: &mut World| {
                    let Ok(profile) = result else {
                        error!("Could not fetch profile");
                        return;
                    };
                    let mut settings = world
                        .get_resource_mut::<Settings>()
                        .expect("Settings resource does not exist");

                    settings.profile = Some(profile);

                    RebuildScreen.apply(world);
                },
            });

            inactive.0 = true;
            commands.entity(entity).insert(TextInputDisabled);
        }
    }
}

struct RebuildScreen;
impl Command for RebuildScreen {
    fn apply(self, world: &mut World) {
        world.run_system_once(clear_screen);
        world.run_system_once(setup_leaderboard_screen);
    }
}

fn reset_token(
    mut settings: ResMut<Settings>,
    reset_button: Query<&Interaction, (With<ResetTokenButton>, Changed<Interaction>)>,
    mut commands: Commands,
) {
    for interaction in &reset_button {
        if *interaction == Interaction::Pressed {
            settings.profile = None;
            commands.add(RebuildScreen);
        }
    }
}

pub struct StartScreenPlugin;

impl Plugin for StartScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::StartScreen),
            (
                setup_start_screen.after(Set::CameraSkyboxInit),
                setup_startscreen_ui.after(clear_screen),
            ),
        )
        .add_systems(
            OnExit(StartScreenState::Leaderboard),
            (clear_screen, setup_startscreen_ui).chain(),
        )
        .add_systems(
            OnEnter(StartScreenState::Leaderboard),
            (clear_screen, setup_leaderboard_screen).chain(),
        )
        .add_systems(
            Update,
            (
                start_game,
                menu_item_hover_effect,
                open_leaderboard,
                back_button,
                copy_token,
                save_token,
                reset_token,
            )
                .run_if(in_start_menu()),
        );
    }
}
