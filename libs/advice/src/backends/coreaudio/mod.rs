use {
    self::host::CoreAudioHost,
    crate::{BackendError, Host},
};

mod audio_unit;
mod device;
mod host;
mod stream;
mod utility;

/// Returns the host implementation for CoreAudio.
pub fn get_host() -> Result<Box<dyn Host>, BackendError> {
    Ok(Box::new(CoreAudioHost))
}
