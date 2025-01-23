use {
    self::host::CoreAudioHost,
    crate::{BackendError, Host},
};

mod device;
mod host;
mod utility;

/// Returns the host implementation for CoreAudio.
pub fn get_host() -> Result<Box<dyn Host>, BackendError> {
    Ok(Box::new(CoreAudioHost))
}
