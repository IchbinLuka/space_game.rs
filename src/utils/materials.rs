use bevy::{pbr::StandardMaterial, utils::default, render::color::Color};
use bevy_mod_outline::OutlineVolume;

pub fn matte_material() -> StandardMaterial {
    StandardMaterial {
        metallic: 0.0,
        reflectance: 0.0,
        ..default()
    }
}

pub fn default_outline() -> OutlineVolume {
    OutlineVolume {
        visible: true,
        width: 4.0,
        colour: Color::BLACK,
    }
}