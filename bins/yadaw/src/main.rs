use kui::{
    elem,
    elements::text::TextResource,
    parley::FontStyle,
    peniko::Color,
    winit::{dpi::PhysicalSize, window::WindowAttributes},
};

/// The glorious entry point of the Yadaw application.
fn main() {
    kui::run(|ctx| {
        initialize_fonts(&ctx);

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
                    font_weight: 800.0;
                    font_width: 1.2;
                    font_style: FontStyle::Italic;
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
fn initialize_fonts(ctx: &kui::Ctx) {
    /// Registers the fonts present in the `fonts` directory.
    fn register_all_fonts(res: &mut TextResource) -> std::io::Result<()> {
        for entry in std::fs::read_dir("bins/yadaw/assets/fonts")? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default().as_encoded_bytes() == b"ttf" {
                let bytes = std::fs::read(path)?;
                res.register_font(bytes);
            }
        }
        Ok(())
    }

    ctx.with_resource_or_default(|res: &mut TextResource| {
        register_all_fonts(res).unwrap_or_else(|err| panic!("Failed to register fonts: {err}"));
    });
}
