//! The Yadaw digital audio workstation.

use yadaw_ui::{dpi::PhysicalSize, winit::window::WindowAttributes};

fn main() {
    yadaw_ui::runtime::run(|app| {
        app.create_window(window_attributes());
    });
}

/// Builds the [`WindowAttributes`] that will be used to create the main window
/// of the application.
fn window_attributes() -> WindowAttributes {
    WindowAttributes::default()
        .with_title("Yadaw")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_maximized(true)
}
