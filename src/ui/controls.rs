use bevy::{ecs::world::Command, prelude::*};

use super::{
    fonts::FontsResource,
    theme::{text_body_style, text_button_style, text_title_style_small},
    ui_card,
    widgets::{screen_overlay, TextButtonBundle},
};

#[derive(Component)]
pub struct ControlsButton;

#[derive(Component)]
struct CloseButton;

#[derive(Component)]
pub struct ControlsScreen;

pub struct ShowControls;
impl Command for ShowControls {
    fn apply(self, world: &mut World) {
        let controls_items = [
            (t!("accelerate"), "W"),
            (t!("turn_left"), "A"),
            (t!("turn_right"), "D"),
            (t!("shoot"), "Space"),
            (t!("place_turret"), "T"),
            (t!("place_bomb"), "G"),
        ];

        let font_res = world
            .get_resource::<FontsResource>()
            .expect("FontsResource not found")
            .clone();

        world
            .spawn((screen_overlay(), ControlsScreen))
            .with_children(|c| {
                c.spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(300.),
                        padding: UiRect::all(Val::Px(15.)),
                        position_type: PositionType::Relative,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..ui_card()
                })
                .with_children(|c| {
                    c.spawn(TextBundle::from_section(
                        t!("controls"),
                        text_title_style_small(&font_res),
                    ));

                    for (title, key) in &controls_items {
                        c.spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::SpaceBetween,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|c| {
                            c.spawn(TextBundle::from_section(
                                title.clone(),
                                text_body_style(&font_res),
                            ));
                            c.spawn(TextBundle::from_section(*key, text_body_style(&font_res)));
                        });
                    }
                    c.spawn((
                        TextButtonBundle::from_section(t!("close"), text_button_style(&font_res)),
                        CloseButton,
                    ))
                    .insert(Style {
                        margin: UiRect::top(Val::Px(15.)),
                        ..default()
                    });
                });
            });
    }
}

fn close_button(
    close_button: Query<&Interaction, With<CloseButton>>,
    controls_screen: Query<Entity, With<ControlsScreen>>,
    mut commands: Commands,
) {
    let Ok(screen) = controls_screen.get_single() else {
        return;
    };
    for interaction in &close_button {
        if *interaction == Interaction::Pressed {
            commands.entity(screen).despawn_recursive();
        }
    }
}

fn show_controls(
    controls_button: Query<&Interaction, (With<ControlsButton>, Changed<Interaction>)>,
    mut commands: Commands,
) {
    for interaction in &controls_button {
        if *interaction == Interaction::Pressed {
            commands.add(ShowControls);
        }
    }
}

pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (close_button, show_controls));
    }
}
