use {
    crate::audio_thread::{AudioBufferMut, AudioBufferOwned, AudioThread, IntoSample},
    advice::{StreamCallback, StreamConfig},
};

/// Initializes the audio thread for the application.
pub fn initialize_audio_thread() {
    let host = advice::default_host()
        .unwrap_or_else(|err| panic!("Failed to initialize the audio host: {err}"))
        .unwrap_or_else(|| panic!("No audio backend available"));

    let output_device = host
        .default_output_device(advice::RoleHint::Games)
        .unwrap_or_else(|err| panic!("Failed to get the default output device: {err}"))
        .unwrap_or_else(|| panic!("No default output device available"));

    let config = output_device
        .output_formats(advice::ShareMode::Share)
        .unwrap_or_else(|err| panic!("Failed to get the available output formats: {err}"))
        .unwrap_or_else(|| panic!("No output formats available for the output device"))
        .to_stream_config(
            advice::ShareMode::Share,
            2,
            &[],
            advice::ChannelLayout::Planar,
            256,
            44100.0,
        );

    let handler = unsafe { make_stream_handler(&config) };

    let stream = output_device
        .open_output_stream(config, handler)
        .unwrap_or_else(|err| panic!("Failed to build the output stream: {err}"));
    stream
        .start()
        .unwrap_or_else(|err| panic!("Failed to start the output stream: {err}"));

    // Leak the stream so that it is not closed when the function returns.
    std::mem::forget(stream);
}

/// Makes the output stream handler for the provided parameters.
///
/// # Safety
///
/// The caller must make sure that the returned handler is only used with a stream created with the
/// same configuration.
unsafe fn make_stream_handler(config: &StreamConfig) -> Box<dyn Send + FnMut(StreamCallback)> {
    unsafe fn make_stream_handler_interleaved<T>(
        config: &StreamConfig,
    ) -> Box<dyn Send + FnMut(StreamCallback)>
    where
        f32: IntoSample<T>,
    {
        let mut audio_thread = AudioThread::new(config.frame_rate);
        let mut buffer = AudioBufferOwned::new(config.channel_count as usize);
        Box::new(move |callback| unsafe {
            buffer.resize(callback.frame_count(), 0.0); // FIXME: Remove this allocation
            audio_thread.fill_buffer(buffer.as_audio_buffer_mut());
            buffer
                .as_audio_buffer_ref()
                .convert_to_interleaved_unchecked(callback.data().interleaved as *mut T);
        })
    }

    unsafe fn make_stream_handler_planar<T>(
        config: &StreamConfig,
    ) -> Box<dyn Send + FnMut(StreamCallback)>
    where
        f32: IntoSample<T>,
    {
        // let mut converter = StreamConverter::new(config.channel_count as usize);
        let mut buffer = AudioBufferOwned::new(config.channel_count as usize);
        let mut audio_thread = AudioThread::new(config.frame_rate);
        Box::new(move |callback| unsafe {
            buffer.resize(callback.frame_count(), 0.0); // FIXME: Remove this allocation
            audio_thread.fill_buffer(buffer.as_audio_buffer_mut());
            buffer.as_audio_buffer_ref().convert_to_planar_unchecked(
                AudioBufferMut::from_raw_parts(
                    callback.data().planar as *const *mut T,
                    callback.frame_count(),
                    buffer.channel_count(),
                ),
            );
        })
    }

    unsafe fn make_stream_handler_planar_f32(
        config: &StreamConfig,
    ) -> Box<dyn Send + FnMut(StreamCallback)> {
        let channel_count = config.channel_count;
        let mut audio_thread = AudioThread::new(config.frame_rate);
        Box::new(move |callback| unsafe {
            audio_thread.fill_buffer(AudioBufferMut::from_raw_parts(
                callback.data().planar as *const *mut f32,
                callback.frame_count(),
                channel_count as usize,
            ));
        })
    }

    unsafe {
        use advice::{ChannelLayout::*, Format::*};
        match (config.channel_layout, config.format) {
            (Interleaved, F32) => make_stream_handler_interleaved::<f32>(config),
            (Interleaved, I16) => make_stream_handler_interleaved::<i16>(config),
            (Planar, F32) => make_stream_handler_planar_f32(config),
            (Planar, I16) => make_stream_handler_planar::<i16>(config),
            (channel_layout, sample_format) => panic!(
                "Unsupported channel layout and format combination: {channel_layout:?}, {sample_format:?}"
            ),
        }
    }
}
