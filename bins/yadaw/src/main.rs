use kui::{
    elem,
    elements::text::TextResource,
    peniko::Color,
    winit::{dpi::PhysicalSize, window::WindowAttributes},
};

/// The glorious entry point of the Yadaw application.
fn main() {
    kui::run(|ctx| {
        initialize_fonts(&ctx).unwrap_or_else(|err| panic!("Failed to register fonts: {err}"));

        let wnd = ctx.create_window(
            WindowAttributes::default()
                .with_title("Yadaw")
                .with_surface_size(PhysicalSize::new(1280, 720)),
        );

        wnd.set_root_element(elem! {
            kui::elements::anchor {
                align_center;

                kui::elements::button {
                    child: make_button();
                }
            }
        });
    });
}

fn make_button() -> impl kui::elements::interactive::InputAppearance {
    kui::elements::interactive::make_appearance(
        elem! {
            kui::elements::div {
                radius: 8px;
                brush: "#ff0000";
                padding_left: 16px;
                padding_right: 16px;
                padding_top: 8px;
                padding_bottom: 8px;

                kui::elements::label {
                    text: "Click me!";
                    inline: true;
                    brush: "#000";
                    font_stack: "Funnel Sans";
                }
            }
        },
        |elem, cx, state| {
            let div = &mut elem.style;

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

/// Initializes the fonts for the application.
fn initialize_fonts(ctx: &kui::Ctx) -> std::io::Result<()> {
    const SUPPORTED_EXTENSIONS: &[&[u8]] = &[b"ttf"];

    ctx.with_resource_or_default(|res: &mut TextResource| {
        for entry in std::fs::read_dir("bins/yadaw/assets/fonts")? {
            let entry = entry?;

            if !entry.file_type()?.is_file() {
                continue;
            }

            let path = entry.path();

            let ext = path.extension().unwrap_or_default().as_encoded_bytes();
            if !SUPPORTED_EXTENSIONS.contains(&ext) {
                continue;
            }

            res.register_font(std::fs::read(path)?);
        }
        Ok(())
    })
}
