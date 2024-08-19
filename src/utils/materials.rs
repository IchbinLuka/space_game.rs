use bevy::{color::Color, pbr::StandardMaterial, utils::default};
use bevy_mod_outline::OutlineVolume;

#[inline(always)]
pub fn _matte_material() -> StandardMaterial {
    StandardMaterial {
        metallic: 0.0,
        reflectance: 0.0,
        ..default()
    }
}

#[inline(always)]
pub fn default_outline() -> OutlineVolume {
    OutlineVolume {
        visible: true,
        width: 2.0,
        colour: Color::BLACK,
    }
}
