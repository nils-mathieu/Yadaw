use {
    crate::audio_thread::{AudioBufferMut, AudioThreadEvent},
    parking_lot::Mutex,
    std::sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

/// Describes a one-shot object that can be played once.
pub trait OneShot: Send {
    /// Fills the provided buffer with audio data.
    ///
    /// The provided [`AudioBufferMut`] should not be overwritten, instead data should be added to
    /// it, ignoring eventual clipping.
    fn fill_buffer(&mut self, frame_rate: f64, buf: AudioBufferMut) -> bool;
}

/// The shared state used to control the one shot player.
#[derive(Default)]
pub struct OneShotPlayerControls {
    /// When set, the one shot player should immediately stop playing.
    ///
    /// The player will automatically clear this flag to acknowledged the operation.
    clear: AtomicBool,

    /// A list of new one-shot objects to play.
    to_play: Mutex<Vec<Box<dyn OneShot>>>,

    /// The number of objects that are currently playing.
    ///
    /// This is written to regularly by the audio thread.
    now_playing: AtomicUsize,
}

impl OneShotPlayerControls {
    /// Creates a new [`OneShotPlayerControls`] instance.
    pub const fn new() -> Self {
        Self {
            clear: AtomicBool::new(false),
            to_play: Mutex::new(Vec::new()),
            now_playing: AtomicUsize::new(0),
        }
    }

    /// Schedules an one-shot object to be played.
    pub fn play(&self, obj: impl 'static + OneShot) {
        self.play_boxed(Box::new(obj));
    }

    /// Schedules an one-shot object to be played.
    pub fn play_boxed(&self, obj: Box<dyn OneShot>) {
        self.to_play.lock().push(obj);
    }

    /// Requests the one shot player to clear its playing list.
    #[inline]
    pub fn clear(&self) {
        self.clear.store(true, Ordering::Relaxed);
    }

    /// Returns the number of objects that are currently playing.
    #[inline]
    pub fn now_playing(&self) -> usize {
        self.now_playing.load(Ordering::Relaxed)
    }
}

static CONTROLS: OneShotPlayerControls = OneShotPlayerControls::new();

/// Returns the controls for the one-shot player.
#[inline]
pub fn one_shot_controls() -> &'static OneShotPlayerControls {
    &CONTROLS
}

/// A simple one-shot player (e.g. sample player).
///
/// Makes sure to release resources once they are no longer needed.
#[derive(Default)]
pub struct OneShotPlayer {
    /// The list of objects that are currently playing.
    playing: Vec<Box<dyn OneShot>>,
}

impl OneShotPlayer {
    /// Fills the provided buffer with audio data.
    ///
    /// Data is *added* to the buffer.
    pub fn fill_buffer(&mut self, frame_rate: f64, mut buf: AudioBufferMut) {
        let prev_playing = self.playing.len();

        if let Some(mut new) = CONTROLS.to_play.try_lock() {
            // FIXME: This allocates on the audio thread. BAD!
            self.playing.append(new.as_mut());
        }

        if CONTROLS.clear.swap(false, Ordering::Relaxed) {
            self.playing.clear();
        }

        self.playing
            .retain_mut(|obj| obj.fill_buffer(frame_rate, buf.reborrow()));

        CONTROLS
            .now_playing
            .store(self.playing.len(), Ordering::Relaxed);

        if prev_playing != self.playing.len() {
            crate::main_window()
                .send_event(AudioThreadEvent::OneShotCountChanged(self.playing.len()));
        }
    }
}
