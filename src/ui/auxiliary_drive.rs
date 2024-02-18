use bevy::prelude::*;
use bevy_round_ui::autosize::RoundUiAutosizeMaterial;

use crate::entities::spaceship::player::Player;
use crate::states::ON_GAME_STARTED;

use super::fonts::FontsResource;

#[derive(Component)]
pub struct AuxiliaryDriveUI;

fn auxiliary_drive_setup(mut commands: Commands, font_resource: Res<FontsResource>) {
    const PADDING: f32 = 5.;

    commands.spawn((
        TextBundle {
            text: Text::from_section(
                t!("auxiliary_drive", state = t!("state_off")),
                TextStyle {
                    font_size: 60.0,
                    font: font_resource.mouse_memoirs.clone(),
                    ..default()
                },
            ),
            style: Style {
                right: Val::Px(10.),
                bottom: Val::Px(10.),
                position_type: PositionType::Absolute,
                padding: UiRect::all(Val::Px(PADDING)),
                ..default()
            },
            ..default()
        },
        RoundUiAutosizeMaterial,
        AuxiliaryDriveUI,
    ));
}

fn auxiliary_drive_update(
    mut query: Query<&mut Text, (Without<Player>, With<AuxiliaryDriveUI>)>,
    player_query: Query<&Player, Changed<Player>>,
) {
    for mut text in query.iter_mut() {
        for player in player_query.iter() {
            text.sections[0].value = t!(
                "auxiliary_drive",
                state = if player.auxiliary_drive {
                    t!("state_on")
                } else {
                    t!("state_off")
                }
            )
            .to_string();
        }
    }
}

pub struct AuxiliaryDriveUIPlugin;

impl Plugin for AuxiliaryDriveUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(ON_GAME_STARTED, (auxiliary_drive_setup,))
            .add_systems(Update, (auxiliary_drive_update,));
    }
}
