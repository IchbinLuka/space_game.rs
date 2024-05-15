use core::fmt;
use std::fmt::Display;
use std::{fs, io};

use bevy::prelude::*;
use bevy::window::{PresentMode, PrimaryWindow};
use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};

const SETTINGS_PATH: &str = "settings.json";

#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub shadows_enabled: bool,
    pub lang: String,
    pub antialiasing: AntialiasingSetting,
    pub vsync: VSyncSetting,
}

#[derive(Default, Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum AntialiasingSetting {
    Off,
    #[default]
    On,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct VSyncSetting(pub bool);

impl Default for VSyncSetting {
    fn default() -> Self {
        Self(true)
    }
}

impl From<VSyncSetting> for PresentMode {
    fn from(setting: VSyncSetting) -> Self {
        if setting.0 {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        }
    }
}

impl From<VSyncSetting> for String {
    fn from(setting: VSyncSetting) -> String {
        if setting.0 {
            t!("on").to_string()
        } else {
            t!("off").to_string()
        }
    }
}

impl AntialiasingSetting {
    pub fn values() -> Vec<Self> {
        vec![Self::Off, Self::On]
    }
}

impl From<AntialiasingSetting> for Msaa {
    fn from(setting: AntialiasingSetting) -> Self {
        match setting {
            AntialiasingSetting::Off => Msaa::Off,
            AntialiasingSetting::On => Msaa::default(),
        }
    }
}

impl From<AntialiasingSetting> for String {
    fn from(setting: AntialiasingSetting) -> String {
        match setting {
            AntialiasingSetting::Off => t!("off").to_string(),
            AntialiasingSetting::On => t!("on").to_string(),
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            shadows_enabled: true,
            lang: "en".to_string(),
            antialiasing: default(),
            vsync: default(),
        }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub enum PersistSettingsError {
    Io(io::Error),
    Serde(serde_json::Error),
    LocalStorageError,
}

impl Display for PersistSettingsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PersistSettingsError::Io(e) => write!(f, "IO error: {}", e),
            PersistSettingsError::Serde(e) => write!(f, "Serde error: {}", e),
            PersistSettingsError::LocalStorageError => write!(f, "Local storage error"),
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn get_local_storage() -> Result<web_sys::Storage, PersistSettingsError> {
    Ok(web_sys::window()
        .expect("could not find a window")
        .local_storage()
        .map_err(|_| PersistSettingsError::LocalStorageError)?
        .expect("local storage could not be loaded"))
}

pub fn persist_settings(settings: &Settings) -> Result<(), PersistSettingsError> {
    let contents = serde_json::to_string(settings).map_err(PersistSettingsError::Serde)?;
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let storage = get_local_storage()?;
            storage.set_item("settings", &contents).map_err(|_| PersistSettingsError::LocalStorageError)?;
            return Ok(());
        } else {
            fs::write(SETTINGS_PATH, contents).map_err(PersistSettingsError::Io)?;
            Ok(())
        }
    }
}

pub fn load_settings() -> Settings {
    #[allow(clippy::needless_late_init)]
    let content;
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let Ok(storage) = get_local_storage() else {
                error!("Failed to access local storage");
                return Settings::default();
            };
            content = storage.get_item("settings").ok().flatten();
        } else {
            content = fs::read_to_string(SETTINGS_PATH).ok()
        }
    };

    if let Some(content) = content
        && let Ok(settings) = serde_json::from_str(content.as_str())
    {
        settings
    } else {
        let settings = Settings::default();
        match persist_settings(&settings) {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to persist settings: {:#}", e);
            }
        }
        settings
    }
}

fn persist_settings_system(settings: Res<Settings>) {
    match persist_settings(&settings) {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to persist settings: {:#}", e);
        }
    }
}

fn setup_settings(
    settings: Res<Settings>,
    mut commands: Commands,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
    rust_i18n::set_locale(settings.lang.as_str());
    let msaa: Msaa = settings.antialiasing.into();
    commands.insert_resource(msaa);
    for mut window in window.iter_mut() {
        window.present_mode = settings.vsync.into();
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_settings)
            .insert_resource(load_settings())
            .add_systems(
                Update,
                persist_settings_system.run_if(
                    resource_changed::<Settings>
                        // Make sure this system does not run when settings are inserted
                        .and_then(not(resource_added::<Settings>)),
                ),
            );
    }
}
