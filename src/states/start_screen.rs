use std::f32::consts::FRAC_PI_4;

use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_round_ui::{
    autosize::{RoundUiAutosizeMaterial, RoundUiAutosizeNode},
    prelude::RoundUiMaterial,
};

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
        minimap::MinimapAssets,
        settings::SettingsButton,
        theme::{text_button_style, text_title_style, SURFACE_COLOR, SURFACE_COLOR_FOCUSED},
    },
    utils::{misc::AsCommand, sets::Set},
};

use super::DespawnOnCleanup;

#[derive(Component)]
struct StartScreen;

#[derive(Component)]
struct StartButton;

const SPACE_STATION_POS: Vec3 = Vec3::new(0., 0., 150.);

fn setup_start_screen(
    font_res: Res<FontsResource>,
    mut commands: Commands,
    camera_assets: Res<CameraAssets>,
    settings: Res<Settings>,
    space_station_res: Res<SpaceStationRes>,
    minimap_res: Res<MinimapAssets>,
    mut materials: ResMut<Assets<RoundUiMaterial>>,
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
        .with_children(|c| {
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

fn start_screen_cleanup(mut commands: Commands, query: Query<Entity, With<StartScreen>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
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

pub struct StartScreenPlugin;

impl Plugin for StartScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::StartScreen),
            (
                setup_start_screen.after(Set::CameraSkyboxInit),
                // space_station_animation.after(setup_start_screen),
            ),
        )
        .add_systems(OnExit(AppState::StartScreen), start_screen_cleanup)
        .add_systems(
            Update,
            (start_game, menu_item_hover_effect).run_if(in_start_menu()),
        );
    }
}
