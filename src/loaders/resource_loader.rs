use crate::utils::errors::{FlashError, FlashErrorType};
use gtk4::gio::{Resource, glib, resources_register};

use super::Loader;

impl Loader {
    pub fn load_resources() -> Result<(), FlashError> {
        let res_bytes = include_bytes!("../../resources.gresources");
        let resource =
            Resource::from_data(&glib::Bytes::from_static(res_bytes)).map_err(|e| FlashError {
                error: FlashErrorType::ResourceError,
                traceback: e.to_string(),
            })?;
        resources_register(&resource);
        Ok(())
    }
}
