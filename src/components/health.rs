use bevy::prelude::*;


#[derive(Component)]
pub struct Health {
    pub health: f32,
    pub max_health: f32,
}

impl Health {
    pub fn take_damage(&mut self, damage: f32) {
        self.health = (self.health - damage).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    #[inline]
    pub fn is_dead(&self) -> bool {
        self.health <= 0.0
    }

    pub fn new(health: f32) -> Self {
        Self {
            health,
            max_health: health,
        }
    }
}


#[derive(Component)]
pub struct DespawnOnDeath;

fn despawn_on_death(
    mut commands: Commands,
    mut query: Query<(Entity, &Health), (Changed<Health>, With<DespawnOnDeath>)>,
) {
    for (entity, health) in &mut query {
        if health.is_dead() {
            commands.entity(entity).despawn_recursive();
        }
    }
}


pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, despawn_on_death);
    }
}