use bevy::app::Plugin;

pub mod asteroid;
pub mod bullet;
pub mod camera;
pub mod cruiser;
pub mod explosion;
pub mod loading_screen;
pub mod planet;
pub mod spaceship;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            camera::CameraComponentPlugin,
            spaceship::SpaceshipPlugin,
            bullet::BulletPlugin,
            loading_screen::LoadingScreenPlugin,
            asteroid::AsteroidPlugin,
            planet::PlanetPlugin,
            explosion::ExplosionPlugin,
            cruiser::CruiserPLugin,
        ));
    }
}
