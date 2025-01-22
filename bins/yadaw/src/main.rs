use kui::{
    elem,
    elements::{div, flex},
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
            flex {
                horizontal,
                align_end,
                justify_end,
                gap: 16px,

                div {
                    width: 100px,
                    height: 200px,
                    brush: "#f00",
                    radius: 8px,
                }


                div {
                    width: 100px,
                    height: 100px,
                    brush: "#0f0",
                    radius: 8px,
                }

                div {
                    width: 100px,
                    height: 150px,
                    brush: "#00f",
                    radius: 8px,
                }
            }
        });
    });
}
