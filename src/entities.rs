use bevy::app::Plugin;

pub mod asteroid;
pub mod bullet;
pub mod camera;
pub mod cruiser;
pub mod explosion;
pub mod planet;
pub mod space_station;
pub mod spaceship;

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
        ));
    }
}
