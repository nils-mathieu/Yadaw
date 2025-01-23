use {
    super::device::CoreAudioDevice,
    crate::{BackendError, Device, Host, RoleHint, backends::coreaudio::utility::backend_error},
    coreaudio_sys::{
        AudioDeviceID, AudioObjectGetPropertyData, AudioObjectGetPropertyDataSize,
        AudioObjectPropertyAddress, AudioObjectPropertySelector,
        kAudioHardwarePropertyDefaultInputDevice, kAudioHardwarePropertyDefaultOutputDevice,
        kAudioHardwarePropertyDevices, kAudioObjectPropertyElementMain,
        kAudioObjectPropertyScopeGlobal, kAudioObjectSystemObject, noErr,
    },
};

/// The [`Host`] implementation for CoreAudio.
pub struct CoreAudioHost;

impl CoreAudioHost {
    /// Returns the list of available device IDs.
    fn enumerate_device_ids(&self) -> Result<Vec<AudioDeviceID>, BackendError> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDevices,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };

        let mut size = 0;

        unsafe {
            let ret = AudioObjectGetPropertyDataSize(
                kAudioObjectSystemObject,
                &property_address,
                0,
                std::ptr::null(),
                &mut size,
            );

            if ret != noErr as i32 {
                return Err(backend_error(
                    "Failed to get the number of available devices",
                    ret,
                ));
            }
        }

        let count = size as usize / std::mem::size_of::<AudioDeviceID>();

        let mut device_ids: Vec<AudioDeviceID> = Vec::with_capacity(count);

        unsafe {
            let ret = AudioObjectGetPropertyData(
                kAudioObjectSystemObject,
                &property_address,
                0,
                std::ptr::null(),
                &mut size,
                device_ids.as_mut_ptr() as *mut _,
            );

            if ret != noErr as i32 {
                return Err(backend_error(
                    "Failed to get the list of available devices",
                    ret,
                ));
            }

            device_ids.set_len(count);
        }

        Ok(device_ids)
    }

    /// Returns the default device for the provided role hint.
    pub fn get_default_device(
        &self,
        selector: AudioObjectPropertySelector,
    ) -> Result<AudioDeviceID, BackendError> {
        let property_address = AudioObjectPropertyAddress {
            mSelector: selector,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        };

        let mut device_id: AudioDeviceID = 0;
        let mut size: u32 = std::mem::size_of::<AudioDeviceID>() as u32;

        unsafe {
            let ret = AudioObjectGetPropertyData(
                kAudioObjectSystemObject,
                &property_address,
                0,
                std::ptr::null(),
                &mut size,
                &mut device_id as *mut _ as _,
            );

            if ret != noErr as i32 {
                return Err(backend_error("Failed to get default device", ret));
            }
        }

        Ok(device_id)
    }
}

impl Host for CoreAudioHost {
    fn devices(&self) -> Result<Vec<Box<dyn Device>>, BackendError> {
        Ok(self
            .enumerate_device_ids()?
            .into_iter()
            .map(|id| Box::new(CoreAudioDevice::new(id)) as Box<dyn Device>)
            .collect())
    }

    fn default_input_device(&self, _: RoleHint) -> Result<Option<Box<dyn Device>>, BackendError> {
        self.get_default_device(kAudioHardwarePropertyDefaultInputDevice)
            .map(|id| Some(Box::new(CoreAudioDevice::new(id)) as Box<dyn Device>))
    }

    fn default_output_device(&self, _: RoleHint) -> Result<Option<Box<dyn Device>>, BackendError> {
        self.get_default_device(kAudioHardwarePropertyDefaultOutputDevice)
            .map(|id| Some(Box::new(CoreAudioDevice::new(id)) as Box<dyn Device>))
    }
}
