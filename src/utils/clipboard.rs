use bevy::prelude::Resource;
use cli_clipboard::{ClipboardContext, ClipboardProvider};

#[derive(Resource)]
pub struct Clipboard {
    context: ClipboardContext,
}

#[allow(dead_code)]
impl Clipboard {
    pub fn get_contents(&mut self) -> Option<String> {
        self.context.get_contents().ok()
    }

    pub fn set_contents(&mut self, content: String) {
        self.context.set_contents(content).unwrap();
    }

    pub fn clear(&mut self) {
        self.context.clear().unwrap();
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Clipboard {
            context: ClipboardContext::new().unwrap(),
        }
    }
}
