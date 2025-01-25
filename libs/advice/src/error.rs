/// Represents an error that might occur when interacting when the raw audio backend.
#[derive(Debug, Clone)]
pub struct BackendError(String);

impl BackendError {
    /// Creates a new [`BackendError`] with the given message.
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for BackendError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(&self.0)
    }
}

impl std::error::Error for BackendError {}

/// An error that might occur when interacting with the API.
#[derive(Debug, Clone)]
pub enum Error {
    /// Indicates that an error occurred in the backend.
    Backend(BackendError),
    /// The provided stream configuration is not supported by the device.
    UnsupportedConfiguration,
    /// Indicates that the device is not (or no longer) available.
    ///
    /// This usually occurs when the device is disconnected while used.
    DeviceNotAvailable,
    /// The device is in use and cannot be accessed.
    DeviceInUse,
}

impl std::fmt::Display for Error {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Backend(e) => std::fmt::Display::fmt(e, f),
            Error::UnsupportedConfiguration => f.pad("The provided stream configuration is not supported by the device"),
            Error::DeviceNotAvailable => f.pad("Device not (or no longer) available"),
            Error::DeviceInUse => f.pad("The device is in use and cannot be accessed"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Backend(e) => Some(e),
            Error::UnsupportedConfiguration => None,
            Error::DeviceNotAvailable => None,
            Error::DeviceInUse => None,
        }
    }
}

impl From<BackendError> for Error {
    #[inline]
    fn from(e: BackendError) -> Self {
        Error::Backend(e)
    }
}
