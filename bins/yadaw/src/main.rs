use kui::{
    elem,
    elements::{
        button,
        button::{ButtonAppearanceFn, ButtonState},
        div, flex,
    },
    peniko::Color,
    winit::{dpi::PhysicalSize, window::WindowAttributes},
};

/// The glorious entry point of the Yadaw application.
fn main() {
    kui::run(|ctx| {
        let wnd = ctx.create_window(
            WindowAttributes::default()
                .with_title("Yadaw")
                .with_surface_size(PhysicalSize::new(1280, 720)),
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

                button {
                    on_click: || println!("Button clicked!"),
                    child: ButtonAppearanceFn::new(
                        |el,  cx, state| {
                            el.style.brush =
                                if state.contains(ButtonState::ACTIVE) {
                                    Some(Color::from_rgb8(0, 0, 255).into())
                                } else if state.contains(ButtonState::HOVER) {
                                    Some(Color::from_rgb8(255, 0, 0).into())
                                } else {
                                    Some(Color::from_rgb8(0, 255, 0).into())
                                };
                            cx.window.request_redraw();
                        },
                        elem! {
                            div {
                                width: 100px,
                                height: 100px,
                                brush: "#0f0",
                                radius: 8px,
                            }
                        }
                    ),
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
