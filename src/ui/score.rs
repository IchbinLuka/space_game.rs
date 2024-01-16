use::bevy::prelude::*;

use crate::utils::sets::Set;




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



pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<ScoreEvent>()
            .add_systems(Update, (
                score_events.in_set(Set::ScoreEvents), 
            ));
    }
}