use std::f32::consts::FRAC_PI_4;

use bevy::{
    ecs::{
        system::{EntityCommands, RunSystemOnce},
        world::Command,
    },
    prelude::*,
};
use bevy_simple_text_input::{TextInputBundle, TextInputInactive, TextInputValue};
use cfg_if::cfg_if;

use crate::{
    entities::{
        camera::{spawn_camera, CameraAssets},
        planet::{spawn_planet, PlanetSpawnConfig},
        space_station::{spawn_space_station, SpaceStationRes},
    },
    model::settings::{Profile, Settings},
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
        ui_card,
        widgets::{FocusTextInputOnInteraction, TextButtonBundle, TextInputDisabled},
    },
    utils::{
        api::{ApiError, ApiManager, Token},
        clipboard::Clipboard,
        misc::AsCommand,
        sets::Set,
        tasks::TaskComponent,
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
                color: Color::WHITE,
                shadows_enabled: settings.shadows_enabled,
                ..default()
            },
            transform,
            ..default()
        },
    ));

    for config in [
        PlanetSpawnConfig {
            color: Color::WHITE,
            size: 30.0,
            pos: Vec3::new(-60.0, -10.0, 100.0),
        },
        PlanetSpawnConfig {
            color: Srgba::hex("11ff22").unwrap().into(),
            size: 25.0,
            pos: Vec3::new(90.0, -20.0, 30.0),
        },
        PlanetSpawnConfig {
            color: Srgba::hex("3300ff").unwrap().into(),
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
    font_res: Res<FontsResource>,
    root: Query<Entity, With<StartScreen>>,
    mut commands: Commands,
) {
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
        c.spawn((NodeBundle {
            style: Style {
                padding: UiRect::all(Val::Px(20.)),
                ..default()
            },
            background_color: SURFACE_COLOR.into(),
            border_radius: BorderRadius::all(Val::Px(30.)),
            ..default()
        },))
            .with_children(|c| {
                c.spawn(TextBundle::from_section(
                    "Space Game",
                    text_title_style(&font_res),
                ));
            });

        c.menu_item()
            .with_children(|c| {
                c.spawn(TextBundle::from_section(
                    t!("leaderboard"),
                    text_button_style(&font_res),
                ));
            })
            .insert(LeaderboardButton);

        c.menu_item()
            .with_children(|c| {
                c.spawn(TextBundle::from_section(
                    t!("settings"),
                    text_button_style(&font_res),
                ));
            })
            .insert(SettingsButton);

        c.menu_item()
            .with_children(|c| {
                c.spawn(TextBundle::from_section(
                    t!("start_game"),
                    text_button_style(&font_res),
                ));
            })
            .insert(StartButton);
    });
}

#[derive(Component)]
struct MenuItem;

trait ChildBuilderExtension {
    fn menu_item(&mut self) -> EntityCommands;
}

impl ChildBuilderExtension for ChildBuilder<'_> {
    fn menu_item(&mut self) -> EntityCommands {
        self.spawn((
            MenuItem,
            NodeBundle {
                style: Style {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceAround,
                    padding: UiRect::all(Val::Px(10.)),
                    ..default()
                },
                border_radius: BorderRadius::all(Val::Px(20.)),
                background_color: SURFACE_COLOR.into(),
                ..default()
            },
            Interaction::default(),
        ))
    }
}

fn menu_item_hover_effect(
    mut query: Query<(&Interaction, &mut BackgroundColor), (With<MenuItem>, Changed<Interaction>)>,
) {
    for (interaction, mut material) in &mut query {
        match *interaction {
            Interaction::Hovered => {
                *material = SURFACE_COLOR_FOCUSED.into();
            }
            Interaction::None => {
                *material = SURFACE_COLOR.into();
            }
            _ => {}
        }
    }
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

#[derive(Component)]
struct PasteTokenButton;

fn setup_leaderboard_screen(
    mut commands: Commands,
    api_manager: Res<ApiManager>,
    font_res: Res<FontsResource>,
    settings: Res<Settings>,
    root_node: Query<Entity, With<StartScreen>>,
) {
    let Ok(root_node) = root_node.get_single() else {
        return;
    };

    let body_style = text_body_style(&font_res);

    commands.entity(root_node).with_children(|c| {
        c.spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                width: Val::Px(650.),
                padding: UiRect::all(Val::Px(20.)),
                align_items: AlignItems::Start,
                ..default()
            },
            ..ui_card()
        })
        .with_children(|c| {
            if let Some(profile) = &settings.profile {
                c.spawn(TextBundle::from_section(
                    t!("logged_in_as", name = profile.name),
                    text_button_small_style(&font_res),
                ));
            }
            c.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    width: Val::Percent(100.),
                    ..default()
                },
                ..default()
            })
            .with_children(|c| {
                if let Some(profile) = &settings.profile {
                    c.spawn(TextBundle {
                        style: Style {
                            margin: UiRect::right(Val::Px(5.)), 
                            ..default()
                        }, 
                        ..TextBundle::from_section("Token: ", body_style.clone())
                    });
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
                        TextButtonBundle::from_section(
                            t!("copy"),
                            text_button_small_style(&font_res),
                        ),
                        CopyTokenButton,
                    ))
                    .insert(Style {
                        margin: UiRect::right(Val::Px(10.)),
                        ..default()
                    });
                    c.spawn((
                        TextButtonBundle::from_section(
                            t!("reset"),
                            text_button_small_style(&font_res),
                        ),
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
                        TextButtonBundle::from_section(
                            t!("save"),
                            text_button_small_style(&font_res),
                        ),
                        SaveTokenButton,
                    ));
                    c.spawn((
                        TextButtonBundle::from_section(
                            t!("paste"),
                            text_button_small_style(&font_res),
                        ),
                        PasteTokenButton,
                    ))
                    .insert(Style {
                        margin: UiRect::left(Val::Px(10.)),
                        ..default()
                    });
                }
            });
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
        c.spawn(NodeBundle {
            style: Style {
                width: Val::Px(400.),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.)),
                ..default()
            },
            ..ui_card()
        })
        .with_children(|c| c.add_leaderboard(request, num, api_manager.clone(), &font_res));

        c.menu_item().insert(BackButton).with_children(|c| {
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

// TODO: Add option to paste token from clipboard

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

            spawn_update_token_task(token, api_manager, &mut commands);

            inactive.0 = true;
            commands.entity(entity).insert(TextInputDisabled);
        }
    }
}

fn spawn_update_token_task(token: Token, api_manager: ApiManager, commands: &mut Commands) {
    commands.spawn(TaskComponent::new(
        async move { api_manager.get_profile(&token).await },
        on_profile_loaded,
    ));
}

fn on_profile_loaded(result: Result<Profile, ApiError>, world: &mut World) {
    let profile = match result {
        Ok(profile) => profile,
        Err(e) => {
            error!("Could not fetch profile: {:?}", e);
            return;
        }
    };

    let mut settings = world
        .get_resource_mut::<Settings>()
        .expect("Settings resource does not exist");

    settings.profile = Some(profile);

    RebuildScreen.apply(world);
}

fn paste_token(
    button: Query<&Interaction, (With<PasteTokenButton>, Changed<Interaction>)>,
    mut commands: Commands,
    mut clipboard: ResMut<Clipboard>,
    api_manager: Res<ApiManager>,
) {
    for interaction in &button {
        if *interaction == Interaction::Pressed {
            let api_manager = api_manager.clone();
            cfg_if! {
                if #[cfg(target_family = "wasm")] {
                    let mut clipboard = clipboard.clone();
                    commands.spawn(TaskComponent::new(
                        async move {
                            let token = clipboard.get_contents().await;

                            match token {
                                Some(token_text) => {
                                    let token = Token(token_text.clone());
                                    Some(api_manager.get_profile(&token).await)
                                }
                                None => None
                            }
                        },
                        move |result, world| {
                            if let Some(result) = result {
                                on_profile_loaded(result, world);
                            }
                        },
                    ));
                } else {
                    let Some(token_text) = clipboard.get_contents() else { continue; };
                    let token = Token(token_text.clone());
                    spawn_update_token_task(token, api_manager, &mut commands);
                }
            }
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
                paste_token,
            )
                .run_if(in_start_menu()),
        );
    }
}
