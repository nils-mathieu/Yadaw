#![feature(impl_trait_in_assoc_type)]

use {
    kui::winit::{dpi::PhysicalSize, window::WindowAttributes},
    std::sync::Arc,
};

mod audio_file;
mod audio_thread;
mod ui;

/// The glorious entry point of the Yadaw application.
fn main() {
    let welcome_sound = Arc::new(
        self::audio_file::AudioFile::load("bins/yadaw/assets/sfx/welcome.wav".into()).unwrap(),
    );

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

        atc.one_shot.play(welcome_sound.play(0.5));

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
