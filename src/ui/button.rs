use bevy::prelude::*;

use super::{theme::default_hover_effect, NodeHoverEffect, TextHoverEffect};

#[derive(Bundle)]
pub struct TextButtonBundle {
    pub text_bundle: TextBundle,
    pub interaction: Interaction,
    pub hover_effect: TextHoverEffect,
}

impl Default for TextButtonBundle {
    fn default() -> Self {
        TextButtonBundle {
            text_bundle: default(),
            interaction: default(),
            hover_effect: default_hover_effect(),
        }
    }
}

impl TextButtonBundle {
    pub fn from_section(text: impl Into<String>, style: TextStyle) -> Self {
        TextButtonBundle {
            text_bundle: TextBundle {
                text: Text::from_section(text, style),
                ..default()
            },
            ..default()
        }
    }
}

#[derive(Component)]
pub struct Switch {
    pub state: bool,
}

#[derive(Bundle)]
pub struct SwitchBundle {
    pub switch: Switch,
    pub interaction: Interaction,
    pub node_bundle: NodeBundle,
}

#[derive(Component)]
pub struct CheckBox {
    pub state: bool,
}

impl CheckBox {
    fn get_hover_effect(&self) -> NodeHoverEffect {
        NodeHoverEffect {
            normal_color: if self.state {
                Color::WHITE
            } else {
                Color::NONE
            },
            hover_color: if self.state {
                Color::WHITE.with_a(0.7)
            } else {
                Color::GRAY.with_a(0.5)
            },
        }
    }
}

#[derive(Bundle)]
pub struct CheckBoxBundle {
    pub check_box: CheckBox,
    pub interaction: Interaction,
    pub node_bundle: NodeBundle,
    pub node_hover_effect: NodeHoverEffect,
}

impl CheckBoxBundle {
    const SIZE: Val = Val::Px(20.);

    pub fn new(initial_state: bool) -> Self {
        let check_box = CheckBox {
            state: initial_state,
        };
        let node_hover_effect = check_box.get_hover_effect();

        Self {
            check_box,
            interaction: Interaction::default(),
            node_bundle: NodeBundle {
                style: Style {
                    border: UiRect::all(Val::Px(1.)),
                    width: Self::SIZE,
                    height: Self::SIZE,
                    ..default()
                },
                border_color: Color::WHITE.into(),
                ..default()
            },
            node_hover_effect,
        }
    }
}

fn check_box_update(
    mut query: Query<(&mut CheckBox, &Interaction, &mut NodeHoverEffect), Changed<Interaction>>,
) {
    for (mut check_box, interaction, mut hover_effect) in &mut query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        check_box.state = !check_box.state;
        *hover_effect = check_box.get_hover_effect();
    }
}

pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, check_box_update);
    }
}
