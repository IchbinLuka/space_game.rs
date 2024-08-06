use bevy::prelude::*;
use bevy_round_ui::{autosize::RoundUiAutosizeMaterial, prelude::RoundUiMaterial};
use bevy_simple_text_input::TextInputInactive;

use super::{theme::default_hover_effect, NodeHoverEffect, TextHoverEffect, UiRes};

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

/*

MaterialNodeBundle {
    material: ui_res.card_background_material.clone(),
    style: Style {
        padding: UiRect::all(Val::Px(20.)),
        flex_direction: FlexDirection::Column,
        width: Val::Px(400.),
        ..default()
    },
    ..default()
},

*/

#[derive(Bundle)]
pub struct CardBundle {
    node: MaterialNodeBundle<RoundUiMaterial>,
    auto_size: RoundUiAutosizeMaterial,
}

impl CardBundle {
    pub fn new(ui_res: &UiRes) -> Self {
        Self {
            node: MaterialNodeBundle {
                material: ui_res.card_background_material.clone(),
                style: Style {
                    padding: UiRect::all(Val::Px(20.)),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            auto_size: RoundUiAutosizeMaterial,
        }
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.node.style = style;
        self
    }
}

#[derive(Component)]
pub struct FocusTextInputOnInteraction;

#[derive(Component)]
pub struct TextInputDisabled;

fn focus_text_input_on_interaction(
    mut text_fields: Query<
        (&mut TextInputInactive, &Interaction),
        (
            With<FocusTextInputOnInteraction>,
            Without<TextInputDisabled>,
        ),
    >,
) {
    for (mut inactive, interaction) in &mut text_fields {
        if *interaction == Interaction::Pressed {
            inactive.0 = false;
        }
    }
}

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (check_box_update, focus_text_input_on_interaction));
    }
}
