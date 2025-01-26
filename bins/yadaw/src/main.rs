#![feature(impl_trait_in_assoc_type)]

use {
    self::audio_file::AudioFile,
    kui::winit::{dpi::PhysicalSize, window::WindowAttributes},
    std::{
        path::Path,
        sync::{Arc, OnceLock},
    },
};

mod audio_file;
mod audio_thread;
mod settings;
mod ui;

/// The proxy to the main window of the application.
///
/// This is used to send messages to the UI thread.
static MAIN_WINDOW: OnceLock<kui::WindowProxy> = OnceLock::new();

/// Returns a proxy to the main window of the application.
#[inline]
fn main_window() -> &'static kui::WindowProxy {
    MAIN_WINDOW.get_or_init(|| panic!("The main window has not been initialized"))
}

/// The glorious entry point of the Yadaw application.
fn main() {
    self::settings::initialize();

    kui::run(|ctx| {
        self::ui::initialize_fonts(&ctx)
            .unwrap_or_else(|err| panic!("Failed to register fonts: {err}"));

        //
        // Create and populate the window with stuff.
        //

        let window = ctx.create_window(
            WindowAttributes::default()
                .with_title("Yadaw")
                .with_surface_size(PhysicalSize::new(1280, 720))
                .with_visible(false),
        );

        debug_assert!(MAIN_WINDOW.get().is_none());
        let _ = MAIN_WINDOW.set(window.make_proxy());

        //
        // Setup the audio thread.
        //

        self::audio_thread::initialize_audio_thread();

        //
        // Play the welcome sound.
        //

        if self::settings::get().miscellaneous.play_startup_sound {
            play_welcome_sound();
        }

        //
        // Show the window.
        //

        window.set_root_element(kui::elem! {
            kui::elements::anchor {
                align_center;
                child: self::ui::magic_menu::magic_menu();
            }
        });

        window.show();
    });
}

/// Plays the welcome sound.
fn play_welcome_sound() {
    const WELCOME_SOUND_PATH: &str = "assets/sfx/welcome.wav";
    let path = Path::new(WELCOME_SOUND_PATH);

    let welcome_sound = match AudioFile::load(path.into()) {
        Ok(s) => Arc::new(s),
        Err(e) => {
            let fullpath = match std::env::current_dir() {
                Ok(cur) => cur.join(path),
                Err(_) => path.to_path_buf(),
            };
            log::error!(
                "Failed to load welcome sound `{}`: {}",
                fullpath.display(),
                e,
            );
            return;
        }
    };

    welcome_sound.play(0.5);
}
