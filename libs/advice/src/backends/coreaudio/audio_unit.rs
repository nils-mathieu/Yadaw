use {
    super::utility::device_error,
    crate::{BackendError, Error},
    coreaudio_sys::{
        AURenderCallbackStruct, AudioBufferList, AudioComponentDescription, AudioComponentFindNext,
        AudioComponentInstanceDispose, AudioComponentInstanceNew, AudioDeviceID,
        AudioOutputUnitStart, AudioOutputUnitStop, AudioStreamBasicDescription, AudioTimeStamp,
        AudioUnit as AudioUnitSys, AudioUnitElement, AudioUnitInitialize,
        AudioUnitRenderActionFlags, AudioUnitScope, AudioUnitSetProperty, OSStatus, OSType, UInt32,
        kAudioDevicePropertyBufferFrameSize, kAudioOutputUnitProperty_CurrentDevice,
        kAudioUnitManufacturer_Apple, kAudioUnitProperty_SetRenderCallback,
        kAudioUnitProperty_StreamFormat, kAudioUnitScope_Global, kAudioUnitSubType_DefaultOutput,
        kAudioUnitSubType_HALOutput, kAudioUnitType_Output, noErr,
    },
    std::{any::Any, ffi::c_void},
};

/// Represents a wrapper audio unit.
pub struct AudioUnit {
    inner: AudioUnitSys,

    /// A closure to be kept alive.
    _callback: Option<Box<dyn Any>>,
}

impl AudioUnit {
    /// Creates a new audio unit.
    pub fn new(ty: OSType, sub_ty: OSType, manufacturer: OSType) -> Result<Self, BackendError> {
        let audio_component_desc = AudioComponentDescription {
            componentType: ty,
            componentSubType: sub_ty,
            componentManufacturer: manufacturer,
            componentFlags: 0,
            componentFlagsMask: 0,
        };

        let audio_component =
            unsafe { AudioComponentFindNext(core::ptr::null_mut(), &audio_component_desc) };
        if audio_component.is_null() {
            return Err(BackendError::new("Failed to create audio unit"));
        }

        let mut audio_unit: AudioUnitSys = std::ptr::null_mut();
        unsafe {
            let ret = AudioComponentInstanceNew(audio_component, &mut audio_unit);

            if ret != noErr as i32 {
                return Err(BackendError::new("Failed to create audio unit"));
            }
        }

        Ok(Self {
            inner: audio_unit,
            _callback: None,
        })
    }

    /// Creates a new output audio unit.
    pub fn new_output() -> Result<Self, BackendError> {
        Self::new(
            kAudioUnitType_Output,
            kAudioUnitSubType_HALOutput,
            kAudioUnitManufacturer_Apple,
        )
    }

    /// Creates a new default output audio unit.
    pub fn new_default_output() -> Result<Self, BackendError> {
        Self::new(
            kAudioUnitType_Output,
            kAudioUnitSubType_DefaultOutput,
            kAudioUnitManufacturer_Apple,
        )
    }

    /// Sets the stream format of the audio unit.
    pub fn set_stream_format(
        &self,
        scope: AudioUnitScope,
        element: AudioUnitElement,
        format: &AudioStreamBasicDescription,
    ) -> Result<(), Error> {
        let ret = unsafe {
            AudioUnitSetProperty(
                self.inner,
                kAudioUnitProperty_StreamFormat,
                scope,
                element,
                format as *const _ as *const _,
                std::mem::size_of::<AudioStreamBasicDescription>() as u32,
            )
        };
        if ret != noErr as i32 {
            return Err(device_error("set kAudioUnitProperty_StreamFormat", ret));
        }

        Ok(())
    }

    /// Sets the render callback associated with this audio unit.
    pub fn set_render_callback_raw(
        &self,
        scope: AudioUnitScope,
        element: AudioUnitElement,
        user_data: *mut c_void,
        callback: extern "C" fn(
            user_data: *mut c_void,
            action_flags: *mut AudioUnitRenderActionFlags,
            timestamp: *const AudioTimeStamp,
            bus_number: UInt32,
            frame_count: UInt32,
            data: *mut AudioBufferList,
        ) -> OSStatus,
    ) -> Result<(), Error> {
        let cb_struct = AURenderCallbackStruct {
            inputProc: Some(callback),
            inputProcRefCon: user_data,
        };

        unsafe {
            let ret = AudioUnitSetProperty(
                self.inner,
                kAudioUnitProperty_SetRenderCallback,
                scope,
                element,
                &cb_struct as *const _ as *const _,
                std::mem::size_of::<AURenderCallbackStruct>() as u32,
            );
            if ret != noErr as i32 {
                return Err(device_error("kAudioUnitProperty_SetRenderCallback", ret));
            }
        }

        Ok(())
    }

    /// Sets the render callback associated with this audio unit.
    pub fn set_render_callback<F>(
        &mut self,
        scope: AudioUnitScope,
        element: AudioUnitElement,
        f: F,
    ) -> Result<(), Error>
    where
        F: 'static
            + Send
            + FnMut(&mut AudioUnitRenderActionFlags, &AudioTimeStamp, u32, u32, *mut AudioBufferList),
    {
        extern "C" fn callback<F>(
            user_data: *mut c_void,
            action_flags: *mut AudioUnitRenderActionFlags,
            timestamp: *const AudioTimeStamp,
            bus_number: UInt32,
            frame_count: UInt32,
            data: *mut AudioBufferList,
        ) -> OSStatus
        where
            F: FnMut(
                &mut AudioUnitRenderActionFlags,
                &AudioTimeStamp,
                u32,
                u32,
                *mut AudioBufferList,
            ),
        {
            let f = unsafe { &mut *(user_data as *mut F) };
            f(
                unsafe { &mut *action_flags },
                unsafe { &*timestamp },
                bus_number,
                frame_count,
                data,
            );
            noErr as OSStatus
        }

        let mut boxed = Box::new(f);

        let f = &mut *boxed as *mut _ as *mut c_void;
        self.set_render_callback_raw(scope, element, f, callback::<F>)?;

        self._callback = Some(boxed);

        Ok(())
    }

    /// Initializes the audio unit.
    pub fn initialize(&self) -> Result<(), Error> {
        let ret = unsafe { AudioUnitInitialize(self.inner) };
        if ret != noErr as i32 {
            return Err(device_error("Failed to initialize audio unit", ret));
        }
        Ok(())
    }

    /// Starts the output of the audio unit.
    pub fn output_start(&self) -> Result<(), Error> {
        let ret = unsafe { AudioOutputUnitStart(self.inner) };
        if ret != noErr as i32 {
            return Err(device_error("Failed to start audio unit", ret));
        }
        Ok(())
    }

    /// Stops the output of the audio unit.
    pub fn output_stop(&self) -> Result<(), Error> {
        let ret = unsafe { AudioOutputUnitStop(self.inner) };
        if ret != noErr as i32 {
            return Err(device_error("Failed to stop audio unit", ret));
        }
        Ok(())
    }

    /// Sets the current device of the audio unit.
    pub fn set_current_device(&self, device_id: AudioDeviceID) -> Result<(), Error> {
        unsafe {
            let ret = AudioUnitSetProperty(
                self.inner,
                kAudioOutputUnitProperty_CurrentDevice,
                kAudioUnitScope_Global,
                0,
                &device_id as *const _ as *const _,
                std::mem::size_of::<AudioDeviceID>() as u32,
            );

            if ret != noErr as i32 {
                return Err(device_error("Failed to set current device", ret));
            }
        }

        Ok(())
    }

    /// Sets the buffer size of the audio unit.
    pub fn set_buffer_size(
        &self,
        scope: AudioUnitScope,
        element: AudioUnitElement,
        size: u32,
    ) -> Result<(), Error> {
        unsafe {
            let ret = AudioUnitSetProperty(
                self.inner,
                kAudioDevicePropertyBufferFrameSize,
                scope,
                element,
                &size as *const _ as *const _,
                std::mem::size_of::<u32>() as u32,
            );

            if ret != noErr as i32 {
                return Err(device_error("Failed to set buffer size", ret));
            }
        }

        Ok(())
    }
}

impl Drop for AudioUnit {
    fn drop(&mut self) {
        unsafe { AudioComponentInstanceDispose(self.inner) };
    }
}
