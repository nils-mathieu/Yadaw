//! The Yadaw digital audio workstation.

use yadaw_ui::{
    dpi::PhysicalSize,
    elem::{self, Length},
    parley::FontContext,
    winit::window::WindowAttributes,
    App,
};

fn main() {
    yadaw_ui::runtime::run(|app| {
        register_fonts(&app);

        let window = app.create_window(window_attributes());

        window.set_root_element(
            elem::Text::new("Hello, world!\nTest")
                .with_basic_style()
                .with_font_family("nunito, sans-serif")
                .with_font_size(Length::Pixels(64.0)),
        );
    });
}

/// Builds the [`WindowAttributes`] that will be used to create the main window
/// of the application.
fn window_attributes() -> WindowAttributes {
    WindowAttributes::default()
        .with_title("Yadaw")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_maximized(true)
}

/// Registers the fonts that will be used by the application.
fn register_fonts(app: &App) {
    app.with_resources_mut(|res| {
        let fcx = res.get_or_insert_default::<FontContext>();

        let entries =
            std::fs::read_dir("assets/fonts").expect("Failed to read the fonts directory");

        for entry in entries {
            let entry = entry.expect("Failed to read a path directory entry");

            if !entry
                .file_type()
                .expect("Failed to read the type of a file")
                .is_file()
            {
                continue;
            }

            let path = entry.path();
            let data = std::fs::read(&path).expect("Failed to read a font file");

            fcx.collection.register_fonts(data);
        }
    });
}
