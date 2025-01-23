use {
    crate::audio_thread::one_shot_player::{OneShotPlayer, OneShotPlayerControls},
    kui::WindowProxy,
    std::sync::Arc,
};

mod driver;
pub use self::driver::*;

mod audio_buffer;
pub use self::audio_buffer::*;

pub mod one_shot_player;

/// An event that might occur from the audio thread.
#[derive(Debug, Clone, Copy)]
pub enum AudioThreadEvent {
    /// The number of one-shot objects that are currently playing has changed.
    OneShotCountChanged(usize),
}

/// The controls for the audio thread.
///
/// An instance of this structure is shared between the audio thread and the ui thread.
pub struct AudioThreadControls {
    /// The window proxy used to send events to the ui thread.
    pub window_proxy: WindowProxy,
    /// the contorls of the one-shot player.
    pub one_shot: OneShotPlayerControls,
}

impl AudioThreadControls {
    /// Creates a new instance of the audio thread controls.
    pub fn new(window_proxy: WindowProxy) -> Self {
        Self {
            window_proxy,
            one_shot: OneShotPlayerControls::default(),
        }
    }
}

/// The state of the audio thread.
struct AudioThread {
    /// The controls of the audio thread.
    controls: Arc<AudioThreadControls>,
    /// The number of frames the audio thread is processing per second.
    frame_rate: f64,

    /// The player responsible for playing one-shot samples.
    one_shot_player: OneShotPlayer,
}

impl AudioThread {
    /// Creates a new audio thread.
    pub fn new(frame_rate: f64, controls: Arc<AudioThreadControls>) -> Self {
        Self {
            controls,
            frame_rate,
            one_shot_player: OneShotPlayer::default(),
        }
    }

    /// The function responsible for filling the audio buffer with data.
    ///
    /// # Remarks
    ///
    /// This function is running on a separate high-priority thread and should *never* block for
    /// any reason.
    ///
    /// This means that any operation that involves the kernel (unless it's specifically a real-time
    /// safe operation) should be avoided at all cost. That includes memory allocations, I/O, etc.
    fn fill_buffer(&mut self, mut buf: AudioBufferMut) {
        buf.channels_mut().for_each(|c| c.fill(0.0));

        self.one_shot_player.fill_buffer(
            self.frame_rate,
            &self.controls.one_shot,
            &self.controls.window_proxy,
            buf.reborrow(),
        );

        buf.channels_mut()
            .for_each(|c| c.iter_mut().for_each(|s| *s = s.clamp(-1.0, 1.0)));
    }
}
