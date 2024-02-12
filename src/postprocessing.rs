use bevy::app::{App, Plugin};

// pub mod outline;

pub struct PostprocessingPlugin;

impl Plugin for PostprocessingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // outline::OutlinePlugin, 
        ));
    }
}