use kui::winit::{dpi::PhysicalSize, window::WindowAttributes};

/// The glorious entry point of the Yadaw application.
fn main() {
    kui::run(|ctx| {
        ctx.create_window(
            WindowAttributes::default()
                .with_title("Yadaw")
                .with_inner_size(PhysicalSize::new(1280, 720)),
        );
    });
}
