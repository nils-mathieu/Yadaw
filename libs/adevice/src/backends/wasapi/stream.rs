use {
    crate::{
        BackendError, Error, Stream, StreamCallback, StreamConfig, StreamData,
        backends::wasapi::utility::{
            backend_error, device_error, frames_to_duration, guard, make_waveformatex,
            share_mode_to_wasapi,
        },
    },
    std::sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    windows::Win32::{
        Foundation::{GetLastError, HANDLE, WAIT_FAILED},
        Media::Audio::{
            AUDCLNT_STREAMFLAGS_EVENTCALLBACK, IAudioCaptureClient, IAudioClient,
            IAudioRenderClient, WAVEFORMATEXTENSIBLE,
        },
        System::Threading::{
            CreateEventA, GetCurrentThread, INFINITE, SetEvent, SetThreadPriority,
            THREAD_PRIORITY_TIME_CRITICAL, WaitForMultipleObjectsEx,
        },
    },
};

/// Whether the stream should be playing or not.
const COMMAND_PLAYING: u8 = 1 << 0;
/// Whether the stream should be closing or not.
const COMMAND_CLOSING: u8 = 1 << 1;

/// The state that is shared between the [`WasapiStream`] and the high-priority thread.
struct SharedState {
    /// A set of flags that represent the commands requested by the [`WasapiStream`] to the
    /// high-priority thread.
    command: AtomicU8,
}

/// Represents a running stream on the WASAPI host.
pub struct WasapiStream {
    /// The state shared between the high-priority thread and the [`WasapiStream`].
    shared_state: Arc<SharedState>,
    /// The handle of an event that must be signaled when the `command` field of the shared state
    /// is updated.
    command_changed_event: HANDLE,
}

impl WasapiStream {
    /// Creates a new [`WasapiStream`] for rendering audio.
    pub fn new(
        audio_client: IAudioClient,
        config: StreamConfig,
        callback: Box<dyn Send + FnMut(StreamCallback)>,
    ) -> Result<Self, Error> {
        //
        // Initialize the audio client with the format supplied by the user.
        //

        let frame_rate = config.frame_rate as u32;

        let buffer_duration = config
            .buffer_size
            .map_or(0, |sz| frames_to_duration(sz.get(), frame_rate));

        let mut waveformat = WAVEFORMATEXTENSIBLE::default();
        if !make_waveformatex(
            config.channel_count,
            config.format,
            config.frame_rate as u32,
            &mut waveformat.Format,
        ) {
            return Err(Error::UnsupportedConfiguration);
        }

        unsafe {
            audio_client
                .Initialize(
                    share_mode_to_wasapi(config.share_mode),
                    AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
                    buffer_duration as i64,
                    0,
                    &waveformat.Format,
                    None,
                )
                .map_err(|err| device_error("IAudioClient::Initialize", err))?;
        }

        //
        // Create an event that will be signaled when the audio client is ready to receive more
        // data.
        //

        let buffer_available_event = unsafe {
            CreateEventA(None, false, false, None)
                .map_err(|err| device_error("CreateEvent", err))?
        };

        unsafe {
            audio_client
                .SetEventHandle(buffer_available_event)
                .map_err(|err| device_error("IAudioClient::SetEventHandle", err))?;
        }

        //
        // Create the event that will be used to signal the high-priority thread that the
        // commands have been updated.
        //

        let command_changed_event = unsafe {
            CreateEventA(None, false, false, None)
                .map_err(|err| device_error("CreateEvent", err))?
        };

        //
        // Create the render client.
        //

        let render_client = unsafe {
            audio_client
                .GetService::<IAudioRenderClient>()
                .map_err(|err| device_error("IAudioClient::GetSerice<IAudioRenderClient>", err))?
        };

        //
        // Create and run the high-priority thread.
        //

        let buffer_size = unsafe {
            audio_client
                .GetBufferSize()
                .map_err(|err| device_error("IAudioClient::GetBufferSize", err))?
        };

        let shared_state = Arc::new(SharedState {
            command: AtomicU8::new(0),
        });

        let mut thread_state = HighPriorityThread {
            audio_client,
            stream_client: StreamClient::Render(render_client),
            shared_state: shared_state.clone(),
            playing: false,
            events: [command_changed_event, buffer_available_event],
            buffer_size,
            callback,
        };

        std::thread::Builder::new()
            .name("adevice time-critical thread".into())
            .spawn(move || thread_state.run())
            .map_err(|err| {
                BackendError::new(format!("Failed to spawn high-priority thread: {err}"))
            })?;

        Ok(Self {
            shared_state,
            command_changed_event,
        })
    }
}

impl Stream for WasapiStream {
    fn start(&self) -> Result<(), Error> {
        self.shared_state
            .command
            .fetch_or(COMMAND_PLAYING, Ordering::SeqCst);
        unsafe {
            SetEvent(self.command_changed_event)
                .map_err(|err| device_error("Failed to signal event", err))
        }
    }

    fn stop(&self) -> Result<(), Error> {
        self.shared_state
            .command
            .fetch_and(!COMMAND_PLAYING, Ordering::SeqCst);
        unsafe {
            SetEvent(self.command_changed_event)
                .map_err(|err| device_error("Failed to signal event", err))
        }
    }

    fn check_error(&self) -> Result<(), Error> {
        unimplemented!()
    }
}

impl Drop for WasapiStream {
    fn drop(&mut self) {
        self.shared_state
            .command
            .fetch_or(COMMAND_CLOSING, Ordering::SeqCst);
    }
}

/// Requests the current thread to become a high-priority time-critical thread.
fn become_high_priority_thread() {
    unsafe {
        let id = GetCurrentThread();
        let _ = SetThreadPriority(id, THREAD_PRIORITY_TIME_CRITICAL);
    }
}

/// The client responsible for rendering or capturing audio data.
enum StreamClient {
    /// For output streams, the render client.
    Render(IAudioRenderClient),
    /// For input streams, the capture client.
    #[allow(
        dead_code,
        reason = "TODO: remove this when implementing input streams"
    )]
    Capture(IAudioCaptureClient),
}

/// The state of the high-priority thread working with the stream.
struct HighPriorityThread {
    /// The audio client that was used to create the stream.
    audio_client: IAudioClient,
    /// The render or capture client to use when rendering or capturing audio data.
    stream_client: StreamClient,

    /// The shared state between the high-priority thread and the [`WasapiStream`].
    shared_state: Arc<SharedState>,
    /// The events that the high-priority thread should wait on.
    ///
    /// # Content
    ///
    /// - `0`: The event that signals that the commands have been updated.
    ///
    /// - `1`: The event that signals that the audio client is ready to receive more data.
    events: [HANDLE; 2],

    /// Whether the audio client is currently running or not.
    playing: bool,

    /// The size of the buffer, in frames.
    buffer_size: u32,

    /// The user-defined callback responsible for actually rendering or capturing the audio data.
    callback: Box<dyn Send + FnMut(StreamCallback)>,
}

// SAFETY: IAudioRenderClient, IAudioCaptureClient, and other COM interfaces are not necessarily
// safe to share across threads, but they are safe to send to other threads as long as they are not
// used concurrently.
unsafe impl Send for HighPriorityThread {}

impl HighPriorityThread {
    /// Runs the high priority thread.
    pub fn run(&mut self) {
        become_high_priority_thread();

        let result = match self.stream_client {
            StreamClient::Render(_) => unsafe { self.run_output_fallible() },
            StreamClient::Capture(_) => unimplemented!(),
        };

        if let Err(err) = result {
            // TODO: Send the error to the main thread.
            panic!("Error in high-priority thread: {err}");
        }
    }

    /// Rusn the high-priority thread to completion, returns an error if something goes wrong.
    ///
    /// # Safety
    ///
    /// Must be called with `stream_client` set to `StreamClient::Render`.
    unsafe fn run_output_fallible(&mut self) -> Result<(), Error> {
        while self.process_commands()? {
            self.wait_for_stuff_to_happen()?;
            unsafe { self.render()? };
        }
        Ok(())
    }

    /// Process the commands that have been requested by the [`WasapiStream`].
    ///
    /// # Returns
    ///
    /// This function returns whether the stream should continue running or not.
    fn process_commands(&mut self) -> Result<bool, Error> {
        let new_commands = self.shared_state.command.load(Ordering::SeqCst);

        if new_commands & COMMAND_CLOSING != 0 {
            return Ok(false);
        }

        let should_play = new_commands & COMMAND_PLAYING != 0;

        if should_play != self.playing {
            self.playing = should_play;

            if self.playing {
                unsafe {
                    self.audio_client
                        .Start()
                        .map_err(|err| device_error("Failed to start audio client", err))?;
                }
            } else {
                unsafe {
                    self.audio_client
                        .Stop()
                        .map_err(|err| device_error("Failed to stop audio client", err))?;
                }
            }
        }

        Ok(true)
    }

    /// Whether the audio client should wait for something to happen (new commands, buffer, etc).
    fn wait_for_stuff_to_happen(&self) -> Result<(), Error> {
        let result = unsafe { WaitForMultipleObjectsEx(&self.events, false, INFINITE, false) };

        if result == WAIT_FAILED {
            let err = unsafe { GetLastError() };
            return Err(backend_error("WaitForMultipleObjectsEx", err.into()).into());
        }

        Ok(())
    }

    /// Executes the output callback once.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `stream_client` is set to `StreamClient::Render`.
    unsafe fn render(&mut self) -> Result<(), Error> {
        unsafe {
            let render_client = match self.stream_client {
                StreamClient::Render(ref render) => render,
                _ => std::hint::unreachable_unchecked(),
            };

            let padding = self
                .audio_client
                .GetCurrentPadding()
                .map_err(|err| device_error("GetCurrentPadding", err))?;

            let available_frames = self.buffer_size - padding;
            if available_frames == 0 {
                return Ok(());
            }

            let buf = render_client
                .GetBuffer(available_frames)
                .map_err(|err| device_error("IAudioRenderClient::GetBuffer", err))?;
            let _guard = guard(|| drop(render_client.ReleaseBuffer(available_frames, 0)));

            (self.callback)(StreamCallback {
                data: StreamData { interleaved: buf },
                frame_count: available_frames as usize,
            });

            Ok(())
        }
    }
}
