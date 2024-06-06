use bevy::{ecs::query::QueryFilter, prelude::*};
use bevy_rapier3d::dynamics::Velocity;

use crate::states::game_running;

use super::{
    bullet::{BulletSpawnEvent, BulletType},
    cruiser::CruiserTurret,
    spaceship::{bot::EnemyTarget, player::PlayerTurret},
    Enemy,
};

const TURRET_TURN_SPEED: f32 = 1.0;
const TURRET_SHOOT_RANGE: f32 = 75.;

#[derive(Component)]
pub struct Turret {
    pub bullet_timer: Timer,
    pub bullet_type: BulletType,
    pub base_orientation: Vec3,
    pub rotation_bounds: (f32, f32),
}

fn turret_update<Filter, Target>(
    mut turrets: Query<(&GlobalTransform, &mut Transform, &mut Turret), Filter>,
    target: Query<&Transform, (Without<Turret>, Target)>,
    time: Res<Time>,
    mut bullet_events: EventWriter<BulletSpawnEvent>,
) where
    Filter: QueryFilter,
    Target: QueryFilter,
{
    for (global_transform, mut transform, mut turret) in &mut turrets {
        let global = global_transform.compute_transform();

        let Some(nearest_transform) = target.iter().min_by_key(|t| {
            let direction = t.translation - global.translation;
            direction.length_squared() as i32
        }) else {
            continue;
        };

        turret.bullet_timer.tick(time.delta());

        let global_translation = global_transform.compute_transform();
        let direction = nearest_transform.translation - global_translation.translation;

        if direction.length_squared() > TURRET_SHOOT_RANGE.powi(2) {
            continue;
        }

        let (min, max) = turret.rotation_bounds;

        let angle = direction.angle_between(turret.base_orientation);

        if angle < min || angle > max {
            continue;
        }

        let turn_sign = global_translation.forward().cross(direction).y.signum();

        transform.rotate_y(turn_sign * TURRET_TURN_SPEED * time.delta_seconds());

        if !turret.bullet_timer.just_finished() {
            continue;
        }

        bullet_events.send(BulletSpawnEvent {
            bullet_type: turret.bullet_type,
            entity_velocity: Velocity::zero(),
            position: global_translation,
            direction,
        });
    }
}

pub struct TurretPlugin;
impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                turret_update::<With<CruiserTurret>, With<EnemyTarget>>,
                turret_update::<With<PlayerTurret>, With<Enemy>>,
            )
                .run_if(game_running()),
        );
    }
}
