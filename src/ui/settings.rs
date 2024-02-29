use bevy::{ecs::system::Command, prelude::*, ui::FocusPolicy};

use crate::model::settings::Settings;

use super::{
    button::{CheckBox, CheckBoxBundle, TextButtonBundle},
    fonts::FontsResource,
    theme::text_button_style,
};

#[derive(Component)]
pub struct SettingsScreen;

#[derive(Component)]
struct CloseButton;

#[derive(Component)]
struct ShadowSetting;

#[derive(Component)]
struct LanguageSetting {
    lang: String,
    available_langs: Vec<String>,
}

pub struct OpenSettings;

impl Command for OpenSettings {
    fn apply(self, world: &mut World) {
        let Some(font_res) = world.get_resource::<FontsResource>() else {
            return;
        };
        let Some(settings) = world.get_resource::<Settings>() else {
            return;
        };

        let settings = settings.clone();

        let style = text_button_style(font_res);

        world
            .spawn((
                SettingsScreen,
                NodeBundle {
                    focus_policy: FocusPolicy::Block,
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::rgba(0., 0., 0., 0.5).into(),
                    ..default()
                },
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
                        LanguageSetting {
                            lang: settings.lang.clone(),
                            available_langs: rust_i18n::available_locales!()
                                .iter()
                                .map(|s| s.to_string())
                                .collect(),
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
    }
}

fn restart_required_text_style() -> TextStyle {
    TextStyle {
        font_size: 30.,
        color: Color::rgb(0.7, 0.7, 0.7),
        ..default()
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

fn update_lang(
    mut query: Query<(&Interaction, &mut LanguageSetting, &mut Text), Changed<Interaction>>,
    mut settings: ResMut<Settings>,
) {
    for (interaction, mut lang_settings, mut text) in &mut query {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let next_lang = lang_settings
            .available_langs
            .iter()
            .cycle()
            .skip_while(|s| *s != &lang_settings.lang)
            .nth(1)
            .unwrap()
            .clone();
        lang_settings.lang = next_lang.clone();
        text.sections[0].value = next_lang.clone();
        settings.lang = next_lang;
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (close_settings, update_shadows, update_lang));
    }
}
