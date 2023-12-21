use std::time::Duration;

use bevy::prelude::*;

#[derive(Component)]
pub struct DespawnAfter {
    pub time: Duration,
    pub spawn_time: Duration,
}

fn despawn_after_system(
    query: Query<(Entity, &DespawnAfter)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, despawn_after) in &mut query.iter() {
        if despawn_after.spawn_time + despawn_after.time < time.elapsed() {
            println!("Despawning entity {:?}", entity);
            commands.entity(entity).despawn_recursive();
        }
    }
}



pub struct DespawnAfterPlugin;

impl Plugin for DespawnAfterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, despawn_after_system);
    }
}