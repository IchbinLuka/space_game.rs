use ::bevy::prelude::*;
use bevy::{render::view::RenderLayers, sprite::Anchor};

use crate::{entities::camera::RENDER_LAYER_2D, utils::sets::Set, AppState};

use super::fonts::FontsResource;

#[derive(Resource)]
pub struct Score(pub u32);

#[derive(Event)]
pub struct ScoreEvent {
    pub score: u32,
    pub world_pos: Vec3,
}

#[derive(Component)]
pub struct ScoreElement {
    pub score: u32,
}

#[derive(Component)]
pub struct ScoreCounter;

fn score_events(
    mut score_events: EventReader<ScoreEvent>,
    mut commands: Commands,
    camera_query: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    window: Query<&Window>,
    font_resource: Res<FontsResource>,
) {
    let Ok((transform, camera)) = camera_query.get_single() else {
        return;
    };
    let Ok(window) = window.get_single() else {
        return;
    };

    let screen_size = Vec2::new(window.width(), window.height());

    for event in score_events.read() {
        let Some(screen_pos) = camera.world_to_viewport(transform, event.world_pos) else {
            warn!("Could not get viewport position for node");
            continue;
        };

        let pos = Vec2::new(
            screen_pos.x - screen_size.x / 2.0,
            -screen_pos.y + screen_size.y / 2.0,
        );

        commands.spawn((
            Text2dBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: format!("+{}", event.score),
                        style: TextStyle {
                            font_size: 40.0,
                            color: Color::WHITE,
                            font: font_resource.mouse_memoirs.clone(),
                        },
                    }],
                    ..default()
                },
                text_anchor: Anchor::Center,
                transform: Transform::from_translation(pos.extend(0.)),
                ..default()
            },
            ScoreElement { score: event.score },
            RenderLayers::layer(RENDER_LAYER_2D),
        ));
    }
}

fn score_element_update(
    mut score_query: Query<(&mut Transform, &ScoreElement, Entity)>,
    time: Res<Time>,
    window: Query<&Window>,
    mut commands: Commands,
    mut score: ResMut<Score>,
) {
    const UI_ELEMENT_SPEED: f32 = 500.0;

    let Ok(window) = window.get_single() else {
        return;
    };

    let counter_location = Vec2::new(0.0, window.height() / 2.0);

    for (mut transform, score_element, entity) in &mut score_query {
        let delta = counter_location - transform.translation.xy();

        if delta.length() < 20.0 {
            score.0 += score_element.score;
            commands.entity(entity).despawn_recursive();
            continue;
        }

        let speed = delta.normalize() * UI_ELEMENT_SPEED;

        transform.translation += Vec3 {
            x: speed.x,
            y: speed.y,
            z: 0.0,
        } * time.delta_seconds();
    }
}

fn score_setup(mut commands: Commands, font_resource: Res<FontsResource>) {
    commands.insert_resource(Score(0));

    commands
        .spawn(NodeBundle {
            style: Style {
                align_content: AlignContent::Center,
                display: Display::Flex,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                ..default()
            },
            ..default()
        })
        .with_children(|c| {
            c.spawn((
                TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: t!("score", score = 0).to_string(),
                            style: TextStyle {
                                font_size: 60.0,
                                font: font_resource.mouse_memoirs.clone(),
                                ..default()
                            },
                        }],
                        ..default()
                    },
                    ..default()
                },
                ScoreCounter,
            ));
        });
}

fn score_update(mut score_query: Query<&mut Text, With<ScoreCounter>>, score: Res<Score>) {
    if !score.is_changed() {
        return;
    }
    for mut text in &mut score_query {
        text.sections[0].value = t!("score", score = score.0).to_string();
    }
}

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ScoreEvent>()
            .add_systems(
                Update,
                (
                    score_events.in_set(Set::ScoreEvents),
                    score_element_update,
                    score_update,
                )
                    .run_if(in_state(AppState::MainScene)),
            )
            .add_systems(OnEnter(AppState::MainScene), (score_setup,));
    }
}
