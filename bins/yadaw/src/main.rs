use kui::{
    elements::Length,
    peniko::Color,
    winit::{dpi::PhysicalSize, window::WindowAttributes},
};

/// The glorious entry point of the Yadaw application.
fn main() {
    kui::run(|ctx| {
        let wnd = ctx.create_window(
            WindowAttributes::default()
                .with_title("Yadaw")
                .with_inner_size(PhysicalSize::new(1280, 720)),
        );

        wnd.set_root_element(
            kui::elements::anchor().align_center().with_child(
                kui::elements::div()
                    .with_brush(Color::from_rgb8(255, 0, 0))
                    .with_border_brush(Color::from_rgb8(0, 255, 0))
                    .with_border_thickness(Length::Pixels(2.0))
                    .with_width(Length::Pixels(128.0))
                    .with_height(Length::Pixels(128.0))
                    .with_radius(Length::Pixels(8.0)),
            ),
        )
    });
}
