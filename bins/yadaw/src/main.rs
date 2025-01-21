use kui::{
    elem,
    elements::{anchor, div},
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

        wnd.set_root_element(elem! {
            anchor {
                align_center,
                div {
                    brush: Color::from_rgb8(255, 0, 0),
                    border_brush: Color::from_rgb8(0, 255, 0),
                    border_thickness: 2px,
                    width: 128px,
                    height: 128px,
                    radius: 8px,
                }
            }
        });
    });
}
