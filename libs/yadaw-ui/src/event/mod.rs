//! The events that can be dispatched by the UI framework.

pub use winit::event::{DeviceId, ElementState};

mod keyboard;
pub use self::keyboard::*;

mod mouse;
pub use self::mouse::*;

/// Indicates that a window has been requested to close.
#[derive(Debug, Clone, Copy)]
pub struct CloseRequested;
