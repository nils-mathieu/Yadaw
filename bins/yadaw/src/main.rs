use kui::{
    elem,
    elements::{anchor, div, label},
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
                    brush: "#f00",
                    width: 100px,
                    height: 100px,

                    label {
                        text: "Helloooo, world!",
                    }
                }
            }
        });
    });
}
