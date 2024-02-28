use bevy::{ecs::system::Command, prelude::*, ui::FocusPolicy};

use super::{
    button::{CheckBox, CheckBoxBundle, TextButtonBundle},
    fonts::FontsResource,
    theme::text_button_style,
};

#[derive(Resource, Clone, Copy)]
pub struct Settings {
    pub shadows_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            shadows_enabled: true,
        }
    }
}

#[derive(Component)]
pub struct SettingsScreen;

#[derive(Component)]
struct CloseButton;

#[derive(Component)]
struct ShadowSetting;

pub struct OpenSettings;

impl Command for OpenSettings {
    fn apply(self, world: &mut World) {
        let Some(font_res) = world.get_resource::<FontsResource>() else {
            return;
        };
        let Some(settings) = world.get_resource::<Settings>() else {
            return;
        };

        let settings = *settings;

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
                    background_color: Color::rgba(0.1, 0.1, 0.1, 0.1).into(),
                    ..default()
                },
            ))
            .with_children(|c| {
                c.settings_item(|c| {
                    c.spawn(TextBundle {
                        text: Text::from_section(t!("shadows"), style.clone()),
                        ..default()
                    });

                    c.spawn((CheckBoxBundle::new(settings.shadows_enabled), ShadowSetting));
                });

                c.spawn((
                    TextButtonBundle::from_section(t!("close"), style),
                    CloseButton,
                ));
            });
    }
}

trait WorldChildBuilderExtension {
    fn settings_item(&mut self, child_builder: impl FnOnce(&mut WorldChildBuilder));
}

impl<'w> WorldChildBuilderExtension for WorldChildBuilder<'w> {
    fn settings_item(&mut self, child_builder: impl FnOnce(&mut WorldChildBuilder)) {
        self.spawn(NodeBundle {
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

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (close_settings, update_shadows))
            .init_resource::<Settings>();
    }
}
