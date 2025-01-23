use {
    kui::{
        elem,
        elements::text::TextResource,
        peniko::Color,
        winit::{dpi::PhysicalSize, window::WindowAttributes},
    },
    std::sync::atomic::{AtomicBool, Ordering},
};

static PAUSED: AtomicBool = AtomicBool::new(true);

/// The glorious entry point of the Yadaw application.
fn main() {
    initialize_audio_thread()
        .unwrap_or_else(|err| panic!("Failed to initialize audio thread: {err}"));

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
                    on_click: move || {PAUSED.fetch_xor(true, Ordering::Relaxed);};
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

/// Initializes the audio thread.
fn initialize_audio_thread() -> Result<(), adevice::Error> {
    let host = match adevice::default_host()? {
        Some(host) => host,

        // No host is available on the current platform. There is nothing we can do about it.
        None => return Ok(()),
    };

    let device = match host.default_output_device(adevice::RoleHint::Games)? {
        Some(device) => device,

        // No output device is available. There is nothing we can do about it.
        None => return Ok(()),
    };

    let stream_config = device
        .output_formats(adevice::ShareMode::Share)?
        .unwrap()
        .to_stream_config(
            adevice::ShareMode::Share,
            2,
            &[],
            adevice::ChannelLayout::Separate,
            512,
            44100.0,
        );

    assert!(stream_config.format == adevice::Format::F32Le);
    assert!(stream_config.channel_count == 2);
    assert!(stream_config.channel_encoding == adevice::ChannelLayout::Interleaved);

    let mut phase: f64 = 0.0;
    let mut amp: f64 = 0.0;

    let stream = device.open_output_stream(
        stream_config,
        Box::new(move |mut callback| {
            let data = unsafe { callback.get_interleaved_output_buffer::<f32>(2) };

            let target_amp = if PAUSED.load(Ordering::Relaxed) {
                0.0
            } else {
                0.8
            };

            let mut i = 0;
            while i < data.len() {
                if i % 2 == 0 {
                    amp += (target_amp - amp) * 0.005;
                }

                let val = amp * (std::f64::consts::TAU * phase).sin();
                data[i] = val as f32;
                data[i + 1] = val as f32;

                phase += 440.0 / 44100.0;
                if phase >= 1.0 {
                    phase -= 1.0;
                }
                i += 2;
            }
        }),
    )?;
    stream.start()?;
    Box::leak(stream);
    Ok(())
}
