//! Kui is the UI toolkit and framework used by the `yadaw` project.
//!
//! It's pretty simple because the needs of the project aren't that complex. It can be used to
//! create vector-based UIs with a simple layout system inspired by the classic CSS flexbox model
//! and a simple event system.

/// Runs the Kui application.
///
/// # Panics
///
/// This function will panic if the application is unable to initialize the graphics API (`wgpu`)
/// or the event loop.
///
/// Also, on some platforms, this function requires to be called from the main thread (maily
/// macOS).
///
/// # Returns
///
/// On most platforms, this function returns when the event loop is closed. On some platforms
/// (notably iOS), this function never returns because the event loop never gives up control
/// of the thread.
///
/// Users should not rely on this function returning to clean up state before closing
/// the application.
pub fn run() {}
