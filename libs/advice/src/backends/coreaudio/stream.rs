use {
    super::{audio_unit::AudioUnit, utility::make_basic_desc},
    crate::{Error, ShareMode, Stream, StreamCallback, StreamConfig, StreamData},
    coreaudio_sys::{AudioDeviceID, kAudioUnitScope_Input},
};

/// The output stream for CoreAudio.
pub struct CoreAudioOutputStream(AudioUnit);

impl CoreAudioOutputStream {
    /// Creates a new [`CoreAudioOutputStream`].
    pub fn new(
        device: Option<AudioDeviceID>,
        config: &StreamConfig,
        mut callback: Box<dyn Send + FnMut(StreamCallback)>,
    ) -> Result<Self, Error> {
        if config.share_mode == ShareMode::Exclusive {
            return Err(Error::UnsupportedConfiguration);
        }

        let scope = kAudioUnitScope_Input; // ???
        let element = 0; // Output

        let mut audio_unit = match device {
            Some(device) => {
                let au = AudioUnit::new_output()?;
                au.set_current_device(device)?;
                au
            }
            None => AudioUnit::new_default_output()?,
        };

        let basic_desc = make_basic_desc(config.format, config.frame_rate, config.channel_count);
        audio_unit.set_stream_format(scope, element, &basic_desc)?;
        if let Some(buffer_size) = config.buffer_size {
            audio_unit.set_buffer_size(scope, element, buffer_size.get())?;
        }

        audio_unit.set_render_callback(scope, element, move |_, _, _, frame_count, buffers| {
            let buf_count = unsafe { (*buffers).mNumberBuffers as usize };
            assert_eq!(buf_count, 1, "CoreAudio buffer count is not 1");
            let data = unsafe { (*buffers).mBuffers.as_ptr().read().mData as *mut u8 };

            callback(StreamCallback {
                data: StreamData { interleaved: data },
                frame_count: frame_count as usize,
            })
        })?;
        audio_unit.initialize()?;

        Ok(Self(audio_unit))
    }
}

impl Stream for CoreAudioOutputStream {
    #[inline]
    fn start(&self) -> Result<(), Error> {
        self.0.output_stop()
    }

    #[inline]
    fn stop(&self) -> Result<(), Error> {
        self.0.output_start()
    }

    fn check_error(&self) -> Result<(), Error> {
        Ok(())
    }
}
