use std::{fs, io};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

const SETTINGS_PATH: &str = "settings.json";

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub shadows_enabled: bool,
    pub lang: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            shadows_enabled: true,
            lang: "en".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum PersistSettingsError {
    Io(io::Error),
    Serde(serde_json::Error),
}

pub fn persist_settings(settings: &Settings) -> Result<(), PersistSettingsError> {
    let contents = serde_json::to_string(settings).map_err(PersistSettingsError::Serde)?;

    fs::write(SETTINGS_PATH, contents).map_err(PersistSettingsError::Io)?;
    Ok(())
}

pub fn load_settings() -> Settings {
    if let Ok(content) = fs::read_to_string(SETTINGS_PATH)
        && let Ok(settings) = serde_json::from_str(content.as_str())
    {
        settings
    } else {
        let settings = Settings::default();
        match persist_settings(&settings) {
            Ok(_) => {}
            Err(e) => {
                match e {
                    PersistSettingsError::Io(e) => {
                        error!("Failed to persist settings: {:?}", e);
                    }
                    PersistSettingsError::Serde(e) => {
                        error!("Failed to encode settings: {:?}", e);
                    }
                }
            }
        }
        settings
    }
}

fn persist_settings_system(settings: Res<Settings>) {
    match persist_settings(&settings) {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to persist settings: {:?}", e);
        }
    }
}

fn setup_lang(settings: Res<Settings>) {
    rust_i18n::set_locale(settings.lang.as_str());
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_lang)
            .insert_resource(load_settings())
            .add_systems(
                Update,
                persist_settings_system.run_if(
                    resource_changed::<Settings>()
                        // Make sure this system does not run when settings are inserted
                        .and_then(not(resource_added::<Settings>())),
                ),
            );
    }
}
