//! The Yadaw digital audio workstation.

use yadaw_ui::{
    dpi::PhysicalSize,
    elem::{self, shapes::RoundedRectangle, Length},
    peniko::Color,
    winit::window::WindowAttributes,
};

fn main() {
    yadaw_ui::runtime::run(|app| {
        let window = app.create_window(window_attributes());

        window.set_root_element(
            elem::shapes::ShapeElement::<RoundedRectangle>::default()
                .with_brush(Color::RED)
                .with_corner_radius(Length::Pixels(64.0)),
        )
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
