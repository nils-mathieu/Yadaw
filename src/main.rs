//! The Yadaw digital audio workstation.

use yadaw_ui::{
    dpi::PhysicalSize,
    elem,
    element::{ElemCtx, Event, EventResult},
    event::{self, NamedKey},
    parley::FontContext,
    winit::window::WindowAttributes,
    App,
};

mod ui;

fn main() {
    yadaw_ui::runtime::run(|app| {
        register_fonts(&app);

        let window = app.create_window(window_attributes());
        window.set_clear_color(self::ui::BACKGROUND_COLOR);

        window.set_root_element(elem::HookEvents::new(
            |_, cx, ev| global_event_handler(cx, ev),
            self::ui::app(),
        ));
    });
}

/// Handles global events that are not specific to any element.
fn global_event_handler(cx: &ElemCtx, event: &dyn Event) -> EventResult {
    if event.is::<event::CloseRequested>() {
        cx.app().exit();
    }

    if let Some(ev) = event.downcast::<event::KeyboardInput>() {
        if ev.logical_key == NamedKey::Escape && ev.state.is_pressed() {
            cx.app().exit();
        }
    }

    EventResult::Continue
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
///
/// # Panics
///
/// This function panics if an I/O error occurs while reading the fonts directory
fn register_fonts(app: &App) {
    try_register_fonts(app).expect("Failed to register fonts");
}

/// Attempts to register the fonts that will be used by the application.
fn try_register_fonts(app: &App) -> std::io::Result<()> {
    app.with_resources_mut(|res| {
        let fcx = res.get_or_insert_default::<FontContext>();

        let entries = std::fs::read_dir("assets/fonts")?;

        for entry in entries {
            let entry = entry?;

            if !entry.file_type()?.is_file() {
                continue;
            }

            let path = entry.path();
            let data = std::fs::read(&path)?;

            fcx.collection.register_fonts(data);
        }

        Ok(())
    })
}
