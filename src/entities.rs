use bevy::app::Plugin;

pub mod camera;
pub mod spaceship;
pub mod bullet;
pub mod loading_screen;
pub mod asteroid;
pub mod planet;
pub mod explosion;

pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            camera::CameraComponentPlugin,
            spaceship::PlayerPlugin,
            bullet::BulletPlugin,
            loading_screen::LoadingScreenPlugin,
            asteroid::AsteroidPlugin,
            planet::PlanetPlugin,
            explosion::ExplosionPlugin,
        ));
    }
}