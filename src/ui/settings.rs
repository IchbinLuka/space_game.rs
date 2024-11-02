use bevy::{ecs::world::Command, prelude::*, window::PrimaryWindow};

use crate::model::settings::{AntialiasingSetting, Settings, VSyncSetting};

use super::{
    fonts::FontsResource,
    theme::text_button_style,
    ui_card,
    widgets::{screen_overlay, CheckBox, CheckBoxBundle, TextButtonBundle},
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
struct VSyncSettingsItem;

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

        let settings = settings.clone();

        let style = text_button_style(font_res);

        world
            .spawn((SettingsScreen, screen_overlay()))
            .with_children(|c| {
                c.spawn(NodeBundle {
                    style: Style {
                        height: Val::Px(330.),
                        padding: UiRect::all(Val::Px(15.)),
                        position_type: PositionType::Relative,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..ui_card()
                })
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

                    c.settings_item(false, |c| {
                        c.spawn(TextBundle::from_section("VSync", style.clone()));

                        let initial: String = settings.vsync.into();
                        c.spawn((
                            TextButtonBundle::from_section(initial, style.clone()),
                            RotateSetting {
                                current_index: if settings.vsync.0 { 0 } else { 1 },
                                values: vec![VSyncSetting(true), VSyncSetting(false)],
                            },
                            VSyncSettingsItem,
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
        color: Color::srgb(0.7, 0.7, 0.7),
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
                width: Val::Px(350.),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|c| {
            c.spawn(NodeBundle {
                style: Style {
                    flex_grow: 1.0,
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

fn update_vsync(
    query: Query<
        &RotateSetting<VSyncSetting>,
        (
            Changed<RotateSetting<VSyncSetting>>,
            With<VSyncSettingsItem>,
        ),
    >,
    mut settings: ResMut<Settings>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
    for rotate_setting in &query {
        settings.vsync = *rotate_setting.value();
        for mut window in &mut window {
            window.present_mode = settings.vsync.into();
        }
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
        app.add_systems(
            Update,
            (
                close_settings,
                update_shadows,
                update_lang,
                settings_button,
                update_msaa,
                update_vsync,
                rotate_settings_item::<String>,
                rotate_settings_item::<AntialiasingSetting>,
                rotate_settings_item::<VSyncSetting>,
            ),
        );
    }
}
