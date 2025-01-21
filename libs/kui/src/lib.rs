//! Kui is the UI toolkit and framework used by the `yadaw` project.
//!
//! It's pretty simple because the needs of the project aren't that complex. It can be used to
//! create vector-based UIs with a simple layout system inspired by the classic CSS flexbox model
//! and a simple event system.

pub extern crate winit;

mod private;

pub mod event_loop;

mod ctx;
pub use self::ctx::*;

mod window;
pub use self::window::*;

/// Runs the Kui application.
///
/// # Parameters
///
/// - `init_fn` is the initialization function that will be called when the application starts. The
///   function is given a [`Ctx`] object which can be used to interact with the application (to
///   create a window, for example).
///
/// # Panics
///
/// This function will panic if the application is unable to initialize the graphics API (`wgpu`)
/// or the event loop.
///
/// Also, on some platforms, this function requires to be called from the main thread (maily
/// macOS).
///
/// The behavior of this function is undefined when it is called re-entrantly.
///
/// # Returns
///
/// On most platforms, this function returns when the event loop is closed. On some platforms
/// (notably iOS), this function never returns because the event loop never gives up control
/// of the thread.
///
/// Users should not rely on this function returning to clean up state before closing
/// the application.
pub fn run(init_fn: impl FnOnce(Ctx)) {
    self::event_loop::run(Box::new(init_fn));
}
