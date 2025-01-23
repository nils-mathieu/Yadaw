use {
    crate::{BackendError, Host, backends::wasapi::com::ensure_com_initialized},
    std::rc::Rc,
};

mod com;
mod device;
mod host;
mod stream;
mod utility;

mod host_config;
pub use self::host_config::*;

/// The default host for the current platform.
pub fn get_host(config: WasapiHostConfig) -> Result<Box<dyn Host>, BackendError> {
    ensure_com_initialized()?;
    Ok(Box::new(self::host::WasapiHost::new(Rc::new(config))?))
}
