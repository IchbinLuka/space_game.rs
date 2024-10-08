use bevy::{app::Plugin, prelude::Component};

pub mod asteroid;
pub mod bullet;
pub mod camera;
pub mod cruiser;
pub mod explosion;
pub mod planet;
pub mod powerup;
pub mod space_station;
pub mod spaceship;
pub mod turret;

#[derive(Component)]
pub struct Enemy;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            camera::CameraComponentPlugin,
            spaceship::SpaceshipPlugin,
            bullet::BulletPlugin,
            asteroid::AsteroidPlugin,
            planet::PlanetPlugin,
            explosion::ExplosionPlugin,
            cruiser::CruiserPLugin,
            space_station::SpaceStationPlugin,
            powerup::PowerupPlugin,
            turret::TurretPlugin,
        ));
    }
}
