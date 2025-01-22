use kui::{
    elem,
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
            kui::elements::button {
                child: make_button();
            }
        });
    });
}

fn make_button() -> impl kui::elements::interactive::InputAppearance {
    kui::elements::interactive::make_appearance(
        elem! {
            kui::elements::anchor {
                align_center;

                kui::elements::div {
                    radius: 8px;
                    brush: "#ff0000";
                    padding: 8px;

                    kui::elements::label {
                        text: "Click me!";
                        inline: true;
                        brush: "#000";
                    }
                }
            }
        },
        |elem, cx, state| {
            let div = &mut elem.child.style;

            if state.active() {
                div.brush = Some(Color::from_rgb8(200, 200, 200).into());
            } else if state.hover() {
                div.brush = Some(Color::from_rgb8(225, 225, 225).into());
            } else {
                div.brush = Some(Color::from_rgb8(255, 255, 255).into());
            }

            cx.window.request_redraw();
        },
    )
}
