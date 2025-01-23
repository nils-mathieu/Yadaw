use {
    crate::audio_thread::AudioThreadEvent,
    kui::{
        elem,
        elements::text::TextResource,
        event::EventResult,
        peniko::Color,
        winit::{dpi::PhysicalSize, window::WindowAttributes},
    },
    std::sync::Arc,
};

mod audio_thread;

struct SinePluck {
    frequency: f64,
    amplitude: f64,
    phase: f64,
}

impl SinePluck {
    pub fn new(freq: f64) -> Self {
        Self {
            frequency: freq,
            amplitude: 1.0,
            phase: 0.0,
        }
    }
}

impl self::audio_thread::one_shot_player::OneShot for SinePluck {
    fn fill_buffer(&mut self, frame_rate: f64, mut buf: audio_thread::AudioBufferMut) -> bool {
        for frame_index in 0..buf.frame_count() {
            let val = self.amplitude * (self.phase * std::f64::consts::TAU).sin();
            self.phase += self.frequency / frame_rate;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
            self.amplitude -= self.amplitude * 0.0001;

            for channel in buf.channels_mut() {
                channel[frame_index] = val as f32;
            }
        }

        self.amplitude > 0.001
    }
}

/// The glorious entry point of the Yadaw application.
fn main() {
    kui::run(|ctx| {
        initialize_fonts(&ctx).unwrap_or_else(|err| panic!("Failed to register fonts: {err}"));

        let wnd = ctx.create_window(
            WindowAttributes::default()
                .with_title("Yadaw")
                .with_surface_size(PhysicalSize::new(1280, 720)),
        );

        let audio_thread_controls = Arc::new(self::audio_thread::AudioThreadControls::new(
            wnd.make_proxy(),
        ));
        self::audio_thread::initialize_audio_thread(audio_thread_controls.clone());

        wnd.set_root_element(elem! {
            kui::elements::flex {
                vertical;
                align_center;
                justify_center;
                gap: 8px;

                child: make_button("Play", {
                    let atc = audio_thread_controls.clone();
                    move || atc.one_shot.play(SinePluck::new(440.0))
                });
                child: make_button("Stop", {
                    let atc = audio_thread_controls.clone();
                    move || atc.one_shot.clear()
                });
                kui::elements::hook_events {
                    kui::elements::label {
                        text: "Running one shots: 0";
                        brush: "#fff";
                        inline: true;
                        font_stack: "Funnel Sans";
                    }
                    on_event: |elem, cx, ev| {
                        if let Some(AudioThreadEvent::OneShotCountChanged(val)) = ev.downcast_ref::<AudioThreadEvent>() {
                            elem.set_text(format!("Running one shots: {val}"));
                            cx.window.request_relayout();
                        }
                        EventResult::Continue
                    };
                }
            }
        });
    });
}

/// Creates a new button element.
fn make_button(text: impl Into<String>, mut on_click: impl FnMut()) -> impl kui::Element {
    elem! {
        kui::elements::button {
            child: kui::elements::interactive::make_appearance(
                elem! {
                    kui::elements::div {
                        radius: 8px;
                        brush: "#ff0000";
                        padding_left: 16px;
                        padding_right: 16px;
                        padding_top: 8px;
                        padding_bottom: 8px;

                        kui::elements::label {
                            text: text;
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
            );

            act_on_press: true;
            on_click: move |_, _| on_click();
        }
    }
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
