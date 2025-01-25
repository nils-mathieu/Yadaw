use {
    super::{
        stream::CoreAudioOutputStream,
        utility::{device_error, extract_cfstring},
    },
    crate::{
        ChannelLayouts, Device, DeviceFormats, Error, ShareMode, Stream, StreamCallback,
        StreamConfig, backends::coreaudio::utility::guard,
    },
    coreaudio_sys::{
        AudioDeviceID, AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize,
        AudioObjectPropertyAddress, AudioObjectPropertyScope, AudioObjectPropertySelector,
        AudioStreamBasicDescription, AudioValueRange, CFRelease, CFStringRef,
        kAudioDevicePropertyBufferFrameSizeRange, kAudioDevicePropertyDeviceNameCFString,
        kAudioDevicePropertyStreamFormats, kAudioObjectPropertyElementMain,
        kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyScopeInput,
        kAudioObjectPropertyScopeOutput, noErr,
    },
};

/// Represents a [`Device`] on the CoreAudio backend.
pub struct CoreAudioDevice {
    /// The ID of the represented device.
    device_id: AudioDeviceID,
    /// Whether the device is the default output device.
    is_default_output: bool,
}

impl CoreAudioDevice {
    /// Creates a new [`CoreAudioDevice`] with the provided device ID.
    pub fn new(id: AudioDeviceID, is_default_output: bool) -> Self {
        Self {
            device_id: id,
            is_default_output,
        }
    }

    /// Bets the format of the stream for the device.
    fn get_stream_formats(
        &self,
        scope: AudioObjectPropertyScope,
    ) -> Result<Vec<AudioStreamBasicDescription>, Error> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyStreamFormats,
            mScope: scope,
            mElement: kAudioObjectPropertyElementMain,
        };

        let mut size = 0;

        unsafe {
            let ret = AudioObjectGetPropertyDataSize(
                self.device_id,
                &property_address,
                0,
                core::ptr::null(),
                &mut size,
            );

            if ret != noErr as i32 {
                return Err(device_error("Failed to read stream format size", ret));
            }
        }

        let count = size as usize / std::mem::size_of::<AudioStreamBasicDescription>();
        let mut buffer: Vec<AudioStreamBasicDescription> = Vec::with_capacity(count);

        unsafe {
            let ret = AudioObjectGetPropertyData(
                self.device_id,
                &property_address,
                0,
                core::ptr::null(),
                &mut size,
                buffer.as_mut_ptr() as *mut _ as _,
            );

            if ret != noErr as i32 {
                return Err(device_error("Failed to read stream format", ret));
            }

            buffer.set_len(count);
        }

        Ok(buffer)
    }

    /// Gets the buffer size range for the device.
    fn get_buffer_size_range(
        &self,
        scope: AudioObjectPropertyScope,
    ) -> Result<AudioValueRange, Error> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyBufferFrameSizeRange,
            mScope: scope,
            mElement: kAudioObjectPropertyElementMain,
        };

        let mut value = AudioValueRange::default();

        unsafe {
            let mut size = std::mem::size_of::<AudioValueRange>() as u32;
            let ret = AudioObjectGetPropertyData(
                self.device_id,
                &property_address,
                0,
                core::ptr::null(),
                &mut size,
                &mut value as *mut _ as _,
            );

            if ret != noErr as i32 {
                return Err(device_error("Failed to read buffer size range", ret));
            }
        }

        Ok(value)
    }

    /// Reads a property from the device as a string.
    fn get_property_as_string(
        &self,
        selector: AudioObjectPropertySelector,
    ) -> Result<String, Error> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: selector,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };

        let mut property: CFStringRef = core::ptr::null();
        let mut property_size: u32 = std::mem::size_of::<CFStringRef>() as u32;

        unsafe {
            let ret = AudioObjectGetPropertyData(
                self.device_id,
                &property_address,
                0,
                core::ptr::null(),
                &mut property_size,
                &mut property as *mut _ as _,
            );

            if ret != noErr as i32 || property.is_null() {
                return Err(device_error("Failed to read device property", ret));
            }

            let _guard = guard(|| CFRelease(property as _));
            Ok(extract_cfstring(property).into_owned())
        }
    }

    /// Gets the formats available for the device.
    fn get_available_formats(
        &self,
        scope: AudioObjectPropertyScope,
    ) -> Result<Option<DeviceFormats>, Error> {
        let mut ret = DeviceFormats::DUMMY;

        ret.channel_layouts.insert(ChannelLayouts::INTERLEAVED);

        fn push_unique<T: PartialEq>(vec: &mut Vec<T>, item: T) {
            if !vec.contains(&item) {
                vec.push(item);
            }
        }

        for (format, frame_rate, channels) in self
            .get_stream_formats(scope)?
            .iter()
            .filter_map(super::utility::extract_basic_desc)
        {
            ret.formats.insert(format.into());
            push_unique(&mut ret.frame_rates, frame_rate);
            ret.max_channel_count = ret.max_channel_count.max(channels);
        }

        let buffer_sizes = self.get_buffer_size_range(scope)?;
        ret.min_buffer_size = buffer_sizes.mMinimum as u32;
        ret.max_buffer_size = buffer_sizes.mMaximum as u32;

        if ret.validate() {
            Ok(Some(ret))
        } else {
            Ok(None)
        }
    }
}

impl Device for CoreAudioDevice {
    fn name(&self) -> Result<Option<String>, Error> {
        self.get_property_as_string(kAudioDevicePropertyDeviceNameCFString)
            .map(Some)
    }

    fn output_formats(&self, share: ShareMode) -> Result<Option<DeviceFormats>, Error> {
        if share == ShareMode::Exclusive {
            return Ok(None);
        }

        self.get_available_formats(kAudioObjectPropertyScopeOutput)
    }

    fn input_formats(&self, share: ShareMode) -> Result<Option<DeviceFormats>, Error> {
        if share == ShareMode::Exclusive {
            return Ok(None);
        }

        self.get_available_formats(kAudioObjectPropertyScopeInput)
    }

    fn open_output_stream(
        &self,
        config: StreamConfig,
        callback: Box<dyn Send + FnMut(StreamCallback)>,
    ) -> Result<Box<dyn Stream>, Error> {
        Ok(Box::new(CoreAudioOutputStream::new(
            if self.is_default_output {
                None
            } else {
                Some(self.device_id)
            },
            &config,
            callback,
        )?))
    }

    fn open_input_stream(
        &self,
        _config: StreamConfig,
        _callback: Box<dyn Send + FnMut(StreamCallback)>,
    ) -> Result<Box<dyn Stream>, Error> {
        unimplemented!();
    }
}
