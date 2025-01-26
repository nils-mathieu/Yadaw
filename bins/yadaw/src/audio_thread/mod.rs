use crate::audio_thread::one_shot_player::OneShotPlayer;

mod driver;
pub use self::driver::*;

mod audio_buffer;
pub use self::audio_buffer::*;

mod one_shot_player;
pub use self::one_shot_player::*;

/// An event that might occur from the audio thread.
#[derive(Debug, Clone, Copy)]
pub enum AudioThreadEvent {
    /// The number of one-shot objects that are currently playing has changed.
    OneShotCountChanged(usize),
}

/// The state of the audio thread.
struct AudioThread {
    /// The number of frames the audio thread is processing per second.
    frame_rate: f64,

    /// The player responsible for playing one-shot samples.
    one_shot_player: OneShotPlayer,
}

impl AudioThread {
    /// Creates a new audio thread.
    pub fn new(frame_rate: f64) -> Self {
        Self {
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

        self.one_shot_player
            .fill_buffer(self.frame_rate, buf.reborrow());

        buf.channels_mut()
            .for_each(|c| c.iter_mut().for_each(|s| *s = s.clamp(-1.0, 1.0)));
    }
}
