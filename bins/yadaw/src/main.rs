#![feature(impl_trait_in_assoc_type)]

use {
    crate::audio_thread::AudioThreadControls,
    kui::winit::{dpi::PhysicalSize, window::WindowAttributes},
    std::{path::Path, sync::Arc},
};

mod audio_file;
mod audio_thread;
mod settings;
mod ui;

/// The glorious entry point of the Yadaw application.
fn main() {
    let settings = self::settings::Settings::load()
        .unwrap_or_else(|err| panic!("Failed to load settings: {err}"));

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

        //
        // Setup the audio thread.
        //

        let atc = Arc::new(self::audio_thread::AudioThreadControls::new(
            window.make_proxy(),
        ));
        self::audio_thread::initialize_audio_thread(atc.clone());

        //
        // Play the welcome sound.
        //

        if settings.miscellaneous.play_startup_sound {
            play_welcome_sound(&atc);
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
fn play_welcome_sound(atc: &AudioThreadControls) {
    const WELCOME_SOUND_PATH: &str = "assets/sfx/welcome.wav";
    let path = Path::new(WELCOME_SOUND_PATH);

    let welcome_sound = match self::audio_file::AudioFile::load(path.into()) {
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

    atc.one_shot.play(welcome_sound.play(0.5));
}
