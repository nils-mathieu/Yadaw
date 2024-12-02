//! The UI framework used by the Yadaw digital audio workstation.
//!
//! This library is a wrapper around multiple other libraries, mainly:
//!
//! - `winit` for window management and cross-platform input handling,
//!
//! - `wgpu` for GPU-accelerated rendering,
//!
//! - and `vello` for 2D graphics rendering.
//!
//! The main goal of this library is to provide a simple and easy-to-use API for creating
//! user interfaces in the Yadaw digital audio workstation.

pub mod elem;
pub mod element;
pub mod event;
pub mod private;
pub mod runtime;
pub mod scheme;

mod window;
pub use self::window::*;

mod app;
pub use self::app::*;

mod ui_resources;
pub use self::ui_resources::*;

pub use {
    parley,
    vello::{kurbo, peniko},
    winit,
    winit::dpi,
};
