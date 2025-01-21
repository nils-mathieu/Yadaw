use {
    kui::winit::{dpi::PhysicalSize, window::WindowAttributes},
    std::time::Duration,
};

/// The glorious entry point of the Yadaw application.
fn main() {
    kui::run(|ctx| {
        let window = ctx.create_window(
            WindowAttributes::default()
                .with_title("Yadaw")
                .with_inner_size(PhysicalSize::new(1280, 720)),
        );

        let ctx2 = ctx.clone();
        ctx.call_after(Duration::from_secs(3), move || {
            window.close();
            ctx2.create_window(
                WindowAttributes::default()
                    .with_title("Yadaw 2")
                    .with_inner_size(PhysicalSize::new(1280, 720)),
            );

            let ctx3 = ctx2.clone();
            ctx2.call_after(Duration::from_secs(3), move || {
                ctx3.exit();
            });
        });
    });
}
