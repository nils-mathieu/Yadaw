#[cfg(all(feature = "coreaudio", target_os = "macos"))]
pub mod coreaudio;
#[cfg(all(feature = "wasapi", target_os = "windows"))]
pub mod wasapi;
