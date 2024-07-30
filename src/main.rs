#![allow(clippy::type_complexity)] // Query types can be really complex
#![feature(let_chains)]

#[macro_use]
extern crate rust_i18n;

i18n!();

use bevy::{
    asset::AssetMetaCheck,
    log::{self, LogPlugin},
    pbr::DirectionalLightShadowMap,
    prelude::*,
    window::PresentMode,
};
use bevy_kira_audio::AudioPlugin;
use bevy_mod_outline::{AutoGenerateOutlineNormalsPlugin, OutlinePlugin};
use bevy_obj::ObjPlugin;
use bevy_rapier3d::prelude::*;
use bevy_round_ui::prelude::RoundUiPlugin;
use bevy_simple_text_input::TextInputPlugin;
use cfg_if::cfg_if;
use components::ComponentsPlugin;
use entities::EntitiesPlugin;
use materials::{toon::ToonMaterial, MaterialsPlugin};
use model::ModelPlugin;
use particles::ParticlesPlugin;
use postprocessing::PostprocessingPlugin;
use states::StatesPlugin;

use ui::UIPlugin;
use utils::{scene::ScenePlugin, UtilsPlugin};

mod api_constants;
mod components;
mod entities;
mod materials;
mod model;
mod particles;
mod postprocessing;
mod states;
mod ui;
mod utils;

fn setup_physics(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.gravity = Vec3::ZERO;
}

// https://taintedcoders.com/bevy/how-to/browser-fullscreen/
#[cfg(target_family = "wasm")]
fn update_canvas_size(mut window: Query<&mut Window, With<bevy::window::PrimaryWindow>>) {
    (|| {
        let mut window = window.get_single_mut().ok()?;
        let browser_window = web_sys::window()?;
        let width = browser_window.inner_width().ok()?.as_f64()?;
        let height = browser_window.inner_height().ok()?.as_f64()?;
        window.resolution.set(width as f32, height as f32);
        Some(())
    })();
}

#[cfg(feature = "debug")]
const LOG_LEVEL: log::Level = log::Level::INFO;

#[cfg(not(feature = "debug"))]
const LOG_LEVEL: log::Level = log::Level::ERROR;

fn main() {
    let mut app = App::new();
    app.insert_resource(AssetMetaCheck::Never)
        .insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins
                .set(LogPlugin {
                    level: LOG_LEVEL,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Space Game".into(),
                        present_mode: PresentMode::Immediate,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                }),
            AudioPlugin,
        ))
        .add_plugins((
            OutlinePlugin,
            TextInputPlugin,
            AutoGenerateOutlineNormalsPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            ObjPlugin,
            #[cfg(feature = "debug")]
            (
                bevy_screen_diagnostics::ScreenDiagnosticsPlugin {
                    style: Style {
                        top: Val::Px(10.),
                        left: Val::Px(10.),
                        ..default()
                    },
                    ..default()
                },
                bevy_screen_diagnostics::ScreenFrameDiagnosticsPlugin,
                bevy_screen_diagnostics::ScreenEntityDiagnosticsPlugin,
            ),
            RoundUiPlugin,
        ))
        .add_systems(Startup, setup_physics)
        .add_plugins((
            StatesPlugin,
            EntitiesPlugin,
            ComponentsPlugin,
            ParticlesPlugin,
            ScenePlugin,
            UIPlugin,
            PostprocessingPlugin,
            MaterialsPlugin,
            ModelPlugin,
            UtilsPlugin,
        ))
        .insert_resource(DirectionalLightShadowMap { size: 4096 });
    cfg_if! {
        if #[cfg(target_family = "wasm")] {
            // app.insert_resource(Msaa::Off);
            app.add_systems(Update, update_canvas_size);
        }
    }

    app.run();
}
