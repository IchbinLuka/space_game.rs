use bevy::{ecs::system::Command, prelude::*, ui::FocusPolicy};
use bevy_round_ui::{
    autosize::{RoundUiAutosizeMaterial, RoundUiAutosizeNodePadding},
    prelude::RoundUiMaterial,
};

use crate::model::settings::Settings;

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
struct LanguageSetting {
    lang: String,
    available_langs: Vec<String>,
}

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
            .expect("Error while cycling through available languages: No languages available.")
            .clone();
        lang_settings.lang.clone_from(&next_lang);
        text.sections[0].value.clone_from(&next_lang);
        settings.lang = next_lang;
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, settings_setup).add_systems(
            Update,
            (close_settings, update_shadows, update_lang, settings_button),
        );
    }
}
