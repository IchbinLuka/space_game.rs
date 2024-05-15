use bevy::{ecs::system::Command, prelude::*, ui::FocusPolicy};
use bevy_round_ui::{
    autosize::{RoundUiAutosizeMaterial, RoundUiAutosizeNodePadding},
    prelude::RoundUiMaterial,
};

use crate::model::settings::{AntialiasingSetting, Settings};

use super::{
    button::{CheckBox, CheckBoxBundle, TextButtonBundle},
    fonts::FontsResource,
    theme::{text_button_style, SURFACE_COLOR},
};

#[derive(Component)]
pub struct SettingsButton;

#[derive(Component)]
pub struct SettingsScreen;

#[derive(Component)]
struct CloseButton;

#[derive(Component)]
struct ShadowSetting;

#[derive(Component)]
struct RotateSetting<T> {
    current_index: usize,
    values: Vec<T>,
}

impl<T> RotateSetting<T> {
    fn next(&mut self) -> &T {
        self.current_index = (self.current_index + 1) % self.values.len();
        &self.values[self.current_index]
    }

    fn value(&self) -> &T {
        &self.values[self.current_index]
    }
}

#[derive(Component)]
struct LanguageSetting;

#[derive(Component)]
struct AntialiasSetting;

#[derive(Resource)]
struct SettingsRes {
    background_material: Handle<RoundUiMaterial>,
}

fn settings_setup(mut commands: Commands, mut materials: ResMut<Assets<RoundUiMaterial>>) {
    commands.insert_resource(SettingsRes {
        background_material: materials.add(RoundUiMaterial {
            background_color: SURFACE_COLOR,
            border_radius: Vec4::splat(30.),
            ..default()
        }),
    })
}

pub struct OpenSettings;

impl Command for OpenSettings {
    fn apply(self, world: &mut World) {
        let Some(font_res) = world.get_resource::<FontsResource>() else {
            error!("Fonts resource not found.");
            return;
        };
        let Some(settings) = world.get_resource::<Settings>() else {
            error!("Settings resource not found.");
            return;
        };

        let Some(settings_res) = world.get_resource::<SettingsRes>() else {
            error!("SettingsRes not found.");
            return;
        };

        let settings = settings.clone();

        let background_material = settings_res.background_material.clone();

        let style = text_button_style(font_res);

        world
            .spawn((
                SettingsScreen,
                NodeBundle {
                    z_index: ZIndex::Global(10),
                    focus_policy: FocusPolicy::Block,
                    style: Style {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),

                        display: Display::Flex,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
            ))
            .with_children(|c| {
                c.spawn((
                    MaterialNodeBundle {
                        material: background_material,
                        style: Style {
                            padding: UiRect::all(Val::Px(10.)),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    },
                    RoundUiAutosizeNodePadding,
                    RoundUiAutosizeMaterial,
                ))
                .with_children(|c| {
                    c.settings_item(false, |c| {
                        c.spawn(TextBundle::from_section(t!("shadows"), style.clone()));

                        c.spawn((CheckBoxBundle::new(settings.shadows_enabled), ShadowSetting));
                    });

                    c.settings_item(true, |c| {
                        c.spawn(TextBundle::from_section(t!("language"), style.clone()));

                        c.spawn((
                            TextButtonBundle::from_section(settings.lang.clone(), style.clone()),
                            LanguageSetting,
                            RotateSetting {
                                current_index: rust_i18n::available_locales!()
                                    .iter()
                                    .position(|s| s == &settings.lang)
                                    .unwrap_or(0),
                                values: rust_i18n::available_locales!()
                                    .iter()
                                    .map(|s| s.to_string())
                                    .collect(),
                            },
                        ));
                    });

                    c.settings_item(false, |c| {
                        c.spawn(TextBundle::from_section(t!("antialiasing"), style.clone()));

                        let initial: String = settings.antialiasing.into();
                        c.spawn((
                            TextButtonBundle::from_section(initial, style.clone()),
                            AntialiasSetting,
                            RotateSetting {
                                current_index: AntialiasingSetting::values()
                                    .iter()
                                    .position(|s| s == &settings.antialiasing)
                                    .unwrap_or(0),
                                values: AntialiasingSetting::values(),
                            },
                        ));
                    });

                    c.spawn(TextBundle {
                        style: Style {
                            margin: UiRect::top(Val::Percent(10.)),
                            ..default()
                        },
                        ..TextBundle::from_section(
                            format!("* {}", t!("restart_required")),
                            TextStyle {
                                font: style.font.clone(),
                                ..restart_required_text_style()
                            },
                        )
                    });

                    c.spawn((
                        TextButtonBundle::from_section(t!("close"), style),
                        CloseButton,
                    ));
                });
            });
    }
}

fn restart_required_text_style() -> TextStyle {
    TextStyle {
        font_size: 30.,
        color: Color::rgb(0.7, 0.7, 0.7),
        ..default()
    }
}

fn rotate_settings_item<T>(
    mut query: Query<(&Interaction, &mut RotateSetting<T>, &mut Text), Changed<Interaction>>,
) where
    T: Clone + Into<String> + 'static + Send + Sync,
{
    for (interaction, mut rotate_setting, mut text) in &mut query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let new_text: String = rotate_setting.next().clone().into();

        text.sections[0].value.clone_from(&new_text);
    }
}

fn settings_button(
    mut commands: Commands,
    query: Query<&Interaction, (With<SettingsButton>, Changed<Interaction>)>,
) {
    for interaction in &query {
        if *interaction == Interaction::Pressed {
            commands.add(OpenSettings);
            break;
        }
    }
}

trait WorldChildBuilderExtension {
    fn settings_item(
        &mut self,
        requires_restart: bool,
        child_builder: impl FnOnce(&mut WorldChildBuilder),
    );
}

impl<'w> WorldChildBuilderExtension for WorldChildBuilder<'w> {
    fn settings_item(
        &mut self,
        requires_restart: bool,
        child_builder: impl FnOnce(&mut WorldChildBuilder),
    ) {
        self.spawn(NodeBundle {
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|c| {
            c.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(300.),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|c| {
                child_builder(c);
            });
            if requires_restart {
                c.spawn(TextBundle::from_section("*", restart_required_text_style()));
            }
        });
    }
}

fn close_settings(
    mut commands: Commands,
    screen: Query<Entity, With<SettingsScreen>>,
    close_button: Query<&Interaction, With<CloseButton>>,
) {
    for interaction in &close_button {
        if *interaction == Interaction::Pressed {
            for entity in &screen {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn update_shadows(
    query: Query<&CheckBox, (With<ShadowSetting>, Changed<CheckBox>)>,
    mut lights: Query<&mut DirectionalLight>,
    mut settings: ResMut<Settings>,
) {
    for check_box in &query {
        for mut light in &mut lights {
            light.shadows_enabled = check_box.state;
        }
        debug!("Shadows enabled: {}", check_box.state);
        settings.shadows_enabled = check_box.state;
    }
}

fn update_msaa(
    query: Query<
        &RotateSetting<AntialiasingSetting>,
        (
            Changed<RotateSetting<AntialiasingSetting>>,
            With<AntialiasSetting>,
        ),
    >,
    mut settings: ResMut<Settings>,
    mut commands: Commands,
) {
    for rotate_setting in &query {
        settings.antialiasing = *rotate_setting.value();
        let res: Msaa = (*rotate_setting.value()).into();
        commands.insert_resource(res);
        debug!("Antialiasing set to: {:?}", settings.antialiasing);
    }
}
fn update_lang(
    query: Query<&RotateSetting<String>, (Changed<RotateSetting<String>>, With<LanguageSetting>)>,
    mut settings: ResMut<Settings>,
) {
    for rotate_setting in &query {
        settings.lang.clone_from(rotate_setting.value());
        debug!("Language set to: {}", settings.lang);
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, settings_setup).add_systems(
            Update,
            (
                close_settings,
                update_shadows,
                update_lang,
                settings_button,
                update_msaa,
                rotate_settings_item::<String>,
                rotate_settings_item::<AntialiasingSetting>,
            ),
        );
    }
}
