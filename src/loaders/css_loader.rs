use gtk4::{CssProvider, gdk::Display};

use super::Loader;
use crate::utils::errors::*;

impl Loader {
    pub fn load_css() -> Result<(), FlashError> {
        let provider = CssProvider::new();

        let display = Display::default().ok_or(FlashError {
            error: FlashErrorType::EnvVar(String::from("Default Display")),
            traceback: format!("Failed to get default display..."),
        })?;

        // Load the base line css
        provider.load_from_resource("/dev/skxxtz/flashmd/main.css");
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        drop(provider);
        Ok(())
    }
}
