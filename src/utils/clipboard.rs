use bevy::prelude::Resource;

pub trait ClipboardManager {
    fn get_contents(&mut self) -> Option<String>;
    fn set_contents(&mut self, content: String);
    fn clear(&mut self);
}

#[cfg(not(target_family = "wasm"))]
pub mod native {
    use super::*;
    use cli_clipboard::{ClipboardContext, ClipboardProvider};

    #[derive(Resource)]
    pub struct NativeClipboard {
        context: ClipboardContext,
    }

    #[allow(dead_code)]
    impl ClipboardManager for NativeClipboard {
        fn get_contents(&mut self) -> Option<String> {
            self.context.get_contents().ok()
        }

        fn set_contents(&mut self, content: String) {
            self.context.set_contents(content).unwrap();
        }

        fn clear(&mut self) {
            self.context.clear().unwrap();
        }
    }

    impl Default for NativeClipboard {
        fn default() -> Self {
            NativeClipboard {
                context: ClipboardContext::new().unwrap(),
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub mod web {
    use super::*;

    #[derive(Resource)]
    pub struct WebClipboard;

    impl ClipboardManager for WebClipboard {
        fn get_contents(&mut self) -> Option<String> {
            let window = web_sys::window().expect("no global `window` exists");
            let navigator = window.navigator();
            navigator.clipboard().read_text().as_string()
        }

        fn set_contents(&mut self, content: String) {
            let window = web_sys::window().expect("no global `window` exists");
            let navigator = window.navigator();
            let _ = navigator.clipboard().write_text(&content);
        }

        fn clear(&mut self) {}
    }

    impl Default for WebClipboard {
        fn default() -> Self {
            WebClipboard
        }
    }
}

#[cfg(not(target_family = "wasm"))]
pub type Clipboard = native::NativeClipboard;

#[cfg(target_family = "wasm")]
pub type Clipboard = web::WebClipboard;
