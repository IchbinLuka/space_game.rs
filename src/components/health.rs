use bevy::prelude::*;

use crate::entities::spaceship::player::LastHit;

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
pub struct Shield;

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

#[derive(Component)]
pub struct Regeneration {
    pub heal_cooldown: f32,
    pub regen_speed: f32,
}

fn regeneration(
    mut query: Query<(&mut Health, &Regeneration, Option<&LastHit>)>,
    time: Res<Time>,
) {
    for (mut health, regen, last_hit) in &mut query {
        if health.is_dead() {
            continue;
        }

        if let Some(last_hit) = last_hit &&
            let Some(last_hit) = last_hit.0 &&
            time.elapsed_seconds() - last_hit < regen.heal_cooldown
        {
            continue;
        }

        health.heal(2.0 * time.delta_seconds());
    }
}

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                despawn_on_death,
                regeneration,
            ));
    }
}
