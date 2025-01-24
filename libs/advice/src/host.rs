use crate::{BackendError, Device};

/// A hint for the role of a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleHint {
    /// The device is used for games.
    Games,
    /// The device is used for notifications.
    Notifications,
    /// The device is used for multimedia playback (e.g. music, movies).
    Multimedia,
    /// The device is used for voice communication (e.g. video calls, voice chat).
    Communications,
}

/// Represents an host responsible for managing a collection of audio devices.
pub trait Host {
    /// Returns the devices that are managed by this [`Host`].
    fn devices(&self) -> Result<Vec<Box<dyn Device>>, BackendError>;

    /// Returns the default input device, if one is available.
    fn default_input_device(&self, role: RoleHint)
    -> Result<Option<Box<dyn Device>>, BackendError>;

    /// Returns the default output device, if one is available.
    fn default_output_device(
        &self,
        role: RoleHint,
    ) -> Result<Option<Box<dyn Device>>, BackendError>;
}
