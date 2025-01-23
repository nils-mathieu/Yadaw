//! A simple abstraction layer around the raw audio interface of a platform.
//!
//! This is largely inspired (and copy/pasted) from `cpal`, adapting stuff to be more
//! aligned with the needs of Yadaw. It's also slightly lower level.

mod error;
pub use self::error::*;

mod host;
pub use self::host::*;

mod device;
pub use self::device::*;

mod stream;
pub use self::stream::*;

mod config;
pub use self::config::*;

mod backends;

/// Host-specific configuration.
pub enum HostConfig {
    /// Use the WASAPI host.
    #[cfg(all(feature = "wasapi", target_os = "windows"))]
    Wasapi(self::backends::wasapi::WasapiHostConfig),
}

/// Gets a specific host implementation with the provided configuration.
///
/// If you don't care about the specific host being used, simply use the [`default_host`] function
/// instead.
///
/// If the host is not available, `None` is returned.
pub fn get_host(config: HostConfig) -> Result<Option<Box<dyn Host>>, BackendError> {
    match config {
        #[cfg(all(feature = "wasapi", target_os = "windows"))]
        HostConfig::Wasapi(config) => backends::wasapi::default_host(config).map(Some),
    }
}

/// Gets the default host for the current platform.
///
/// If you need to configure the host, use [`get_host`] instead.
#[allow(dead_code)]
pub fn default_host() -> Result<Option<Box<dyn Host>>, BackendError> {
    #[cfg(all(feature = "wasapi", target_os = "windows"))]
    return self::backends::wasapi::default_host(Default::default()).map(Some);
}
