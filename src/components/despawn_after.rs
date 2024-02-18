
use std::time::Duration;

use bevy::prelude::*;

use crate::states::game_running;

#[derive(Component)]
pub struct DespawnTimer(pub Timer);

impl DespawnTimer {
    pub fn new(duration: Duration) -> Self {
        Self(Timer::new(duration, TimerMode::Once))
    }
}

fn despawn_after_system(
    mut query: Query<(Entity, &mut DespawnTimer)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut timer) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub struct DespawnAfterPlugin;

impl Plugin for DespawnAfterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, despawn_after_system.run_if(game_running()));
    }
}
