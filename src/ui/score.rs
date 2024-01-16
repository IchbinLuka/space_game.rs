use::bevy::prelude::*;

use crate::{utils::sets::Set, AppState};

#[derive(Resource)]
pub struct Score(pub u32);


#[derive(Event)]
pub struct ScoreEvent {
    pub score: u32,
    pub world_pos: Vec3,
}

#[derive(Component)]
pub struct ScoreElement {
    pub score: u32
}

fn score_events(
    mut score_events: EventReader<ScoreEvent>,
    mut commands: Commands,
    camera_query: Query<(&GlobalTransform, &Camera)>, 
    mut score: ResMut<Score>,
) {
    let Ok((transform, camera)) = camera_query.get_single() else { return; };
    for event in score_events.read() {
        let Some(screen_pos) = camera.world_to_viewport(
            transform, 
            event.world_pos, 
        ) else {
            warn!("Could not get viewport position for node");
            continue;
        };

        score.0 += event.score;

        commands.spawn((
            ScoreElement {
                score: event.score
            }, 
            NodeBundle {
                style: Style {
                    left: Val::Px(screen_pos.x),
                    top: Val::Px(screen_pos.y),
                    ..default()
                }, 
                ..default()
            }, 
        )).with_children(|c| {
            c.spawn(TextBundle {
                text: Text { 
                    sections: vec![
                        TextSection {
                            value: format!("+{}", event.score),
                            style: TextStyle {
                                font_size: 20.0,
                                color: Color::WHITE,
                                ..default()
                            }, 
                        }
                    ], 
                    ..default()
                 }, 
                 ..default()
            });
        });
    }
}

fn score_update(
    mut score_query: Query<(&mut Style, &ScoreElement)>, 
    time: Res<Time>,
    window: Query<&Window>, 
) {
    const UI_ELEMENT_SPEED: f32 = 100.0;

    let Ok(window) = window.get_single() else { return; };

    for (mut style, _score_element) in &mut score_query {
        let Val::Px(left) = style.left else { continue; };
        let Val::Px(top) = style.top else { continue; };

        let delta = Vec2 { 
            x: window.width() / 2.0 - left, 
            y: -top, 
        }.normalize() * UI_ELEMENT_SPEED;

        style.top = Val::Px(top + delta.y * time.delta_seconds());
        style.left = Val::Px(left + delta.x * time.delta_seconds());
    }
}

fn score_setup(
    mut commands: Commands
) {
    commands.insert_resource(Score(0));

}

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<ScoreEvent>()
            .add_systems(Update, (
                score_events.in_set(Set::ScoreEvents), 
                score_update, 
            ))
            .add_systems(OnEnter(AppState::MainScene), (
                score_setup, 
            ));
    }
}