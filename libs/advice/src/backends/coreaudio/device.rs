use {
    super::utility::{device_error, extract_cfstring},
    crate::{
        BackendError, ChannelLayouts, Device, DeviceFormats, Error, ShareMode, Stream,
        StreamCallback, StreamConfig, backends::coreaudio::utility::guard,
    },
    coreaudio_sys::{
        AudioBufferList, AudioDeviceID, AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize,
        AudioObjectPropertyAddress, AudioObjectPropertyScope, AudioObjectPropertySelector,
        AudioValueRange, CFRelease, CFStringRef, kAudioDevicePropertyAvailableNominalSampleRates,
        kAudioDevicePropertyDeviceNameCFString, kAudioDevicePropertyStreamConfiguration,
        kAudioObjectPropertyElementMain, kAudioObjectPropertyElementMaster,
        kAudioObjectPropertyScopeGlobal, kAudioObjectPropertyScopeInput,
        kAudioObjectPropertyScopeOutput, noErr,
    },
    std::alloc::Layout,
};

/// Represents a [`Device`] on the CoreAudio backend.
pub struct CoreAudioDevice {
    /// The ID of the represented device.
    device_id: AudioDeviceID,
}

impl CoreAudioDevice {
    /// Creates a new [`CoreAudioDevice`] with the provided device ID.
    pub fn new(id: AudioDeviceID) -> Self {
        Self { device_id: id }
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

    /// Returns the stream configuration for this audio device.
    fn get_channel_count(&self, scope: AudioObjectPropertyScope) -> Result<u16, Error> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyStreamConfiguration,
            mScope: scope,
            mElement: kAudioObjectPropertyElementMaster,
        };

        let mut data_size: u32 = 0;

        unsafe {
            let ret = AudioObjectGetPropertyDataSize(
                self.device_id,
                &property_address,
                0,
                core::ptr::null(),
                &mut data_size,
            );

            if ret != noErr as i32 {
                return Err(device_error("kAudioDevicePropertyStreamConfiguration", ret));
            }
        }

        // NOTE: We can't use `Vec` here because the allocation needs to be properly
        // aligned.
        let (_guard, buf) = unsafe {
            let layout = Layout::from_size_align_unchecked(
                data_size as usize,
                std::mem::align_of::<AudioBufferList>(),
            );

            let buf = std::alloc::alloc(layout);

            if buf.is_null() {
                std::alloc::handle_alloc_error(layout);
            }

            (
                guard(move || std::alloc::dealloc(buf, layout)),
                buf as *mut AudioBufferList,
            )
        };

        unsafe {
            let ret = AudioObjectGetPropertyData(
                self.device_id,
                &property_address,
                0,
                core::ptr::null(),
                &mut data_size,
                buf as *mut _ as *mut _,
            );

            if ret != noErr as i32 {
                return Err(device_error("kAudioDevicePropertyStreamConfiguration", ret));
            }
        }

        let buffers = unsafe {
            let buffer_list = &*buf;
            std::slice::from_raw_parts(
                buffer_list.mBuffers.as_ptr(),
                buffer_list.mNumberBuffers as usize,
            )
        };

        match buffers {
            [] => Ok(0),
            [buf] => Ok(buf.mNumberChannels as u16),
            _ => Err(Error::Backend(BackendError::new(
                "Unsupported number of buffers",
            ))),
        }
    }

    /// Returns the list of available sample rates for the device.
    fn get_nominal_sample_rates(
        &self,
        scope: AudioObjectPropertyScope,
    ) -> Result<Vec<AudioValueRange>, Error> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioDevicePropertyAvailableNominalSampleRates,
            mScope: scope,
            mElement: kAudioObjectPropertyElementMain,
        };

        let mut data_size: u32 = 0;

        unsafe {
            let ret = AudioObjectGetPropertyDataSize(
                self.device_id,
                &property_address,
                0,
                core::ptr::null(),
                &mut data_size,
            );

            if ret != noErr as i32 {
                return Err(device_error(
                    "kAudioDevicePropertyAvailableNominalSampleRates",
                    ret,
                ));
            }
        }

        let count = data_size as usize / std::mem::size_of::<AudioValueRange>();
        let mut buffer: Vec<AudioValueRange> = Vec::with_capacity(count);

        unsafe {
            let ret = AudioObjectGetPropertyData(
                self.device_id,
                &property_address,
                0,
                core::ptr::null(),
                &mut data_size,
                buffer.as_mut_ptr() as *mut _,
            );

            if ret != noErr as i32 {
                return Err(device_error(
                    "kAudioDevicePropertyAvailableNominalSampleRates",
                    ret,
                ));
            }

            buffer.set_len(count);
        }

        Ok(buffer)
    }

    /// Gets the formats available for the device.
    fn get_available_formats(
        &self,
        scope: AudioObjectPropertyScope,
    ) -> Result<Option<DeviceFormats>, Error> {
        let mut ret = DeviceFormats::DUMMY;

        ret.channel_layouts.insert(ChannelLayouts::INTERLEAVED);
        ret.max_channel_count = self.get_channel_count(scope)?;
        ret.frame_rates = self
            .get_nominal_sample_rates(scope)?
            .into_iter()
            .map(|range| range.mMinimum)
            .collect();

        // WIP: Finish querying the data we need.

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
        todo!();
    }

    fn open_input_stream(
        &self,
        config: StreamConfig,
        callback: Box<dyn Send + FnMut(StreamCallback)>,
    ) -> Result<Box<dyn Stream>, Error> {
        todo!();
    }
}
