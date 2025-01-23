/// A collection of buffers for the audio stream.
pub struct AudioBufferMut<'a, T = f32> {
    /// The actual audio data.
    ///
    /// This is a list of `channel_count` buffers, each containing `frame_count` frames.
    data: *const *mut T,
    /// The number of frames in each buffer.
    frame_count: usize,
    /// The number of channels.
    channel_count: usize,

    /// The lifetime of the audio buffer.
    _lifetime: std::marker::PhantomData<&'a ()>,
}

#[allow(dead_code, reason = "TODO: remove this when stuff is used")]
impl<T> AudioBufferMut<'_, T> {
    /// Creates a new [`AudioBuffer<T>`] from the provided raw parts.
    ///
    /// # Safety
    ///
    /// The caller must make sure that the provided `data` pointer references `channel_count`
    /// buffers, each referencing `frame_count` frames.
    ///
    /// The data must remain valid for the lifetime `'a`.
    #[inline]
    pub unsafe fn from_raw_parts(
        data: *const *mut T,
        frame_count: usize,
        channel_count: usize,
    ) -> Self {
        Self {
            data,
            frame_count,
            channel_count,
            _lifetime: std::marker::PhantomData,
        }
    }

    /// Returns the number of channels in the audio buffer.
    #[inline(always)]
    pub fn channel_count(&self) -> usize {
        self.channel_count
    }

    /// Returns the number of frames in each channel of the audio buffer.
    #[inline(always)]
    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    /// Returns the frames for the channel with the provided index.
    ///
    /// # Safety
    ///
    /// The caller must make sure that the provided `channel` index is smaller than
    /// `.channel_count()`.
    #[inline]
    pub unsafe fn channel_unchecked(&self, channel: usize) -> &[T] {
        debug_assert!(channel < self.channel_count);
        unsafe { std::slice::from_raw_parts(*self.data.add(channel), self.frame_count) }
    }

    /// Returns the frames for the channel with the provided index.
    ///
    /// # Safety
    ///
    /// The caller must make sure that the provided `channel` index is smaller than
    /// `.channel_count()`.
    #[inline]
    pub unsafe fn channel_unchecked_mut(&mut self, channel: usize) -> &mut [T] {
        debug_assert!(channel < self.channel_count);
        unsafe { std::slice::from_raw_parts_mut(*self.data.add(channel), self.frame_count) }
    }

    /// Returns the frames for the channel with the provided index.
    ///
    /// # Returns
    ///
    /// Returns `None` if the provided `channel` index is out of bounds.
    #[inline]
    pub fn channel(&self, channel: usize) -> Option<&[T]> {
        if channel < self.channel_count {
            Some(unsafe { self.channel_unchecked(channel) })
        } else {
            None
        }
    }

    /// Returns the frames for the channel with the provided index.
    ///
    /// # Returns
    ///
    /// Returns `None` if the provided `channel` index is out of bounds.
    #[inline]
    pub fn channel_mut(&mut self, channel: usize) -> Option<&mut [T]> {
        if channel < self.channel_count {
            Some(unsafe { self.channel_unchecked_mut(channel) })
        } else {
            None
        }
    }

    /// Returns an iterator over the channels of the audio buffer.
    #[inline]
    pub fn channels(&self) -> impl Iterator<Item = &[T]> + '_ {
        (0..self.channel_count).map(move |channel| unsafe { self.channel_unchecked(channel) })
    }

    /// Returns an iterator over the channels of the audio buffer.
    #[inline]
    pub fn channels_mut(&mut self) -> impl Iterator<Item = &mut [T]> + '_ {
        (0..self.channel_count).map(move |channel| unsafe {
            // SAFETY: We can't just use `channel_unchecked_mut` because it borrows self
            // completely.
            std::slice::from_raw_parts_mut(*self.data.add(channel), self.frame_count)
        })
    }

    /// Re-borrows the buffer with a shorter lifetime without consuming the original reference.
    pub fn reborrow(&mut self) -> AudioBufferMut<T> {
        AudioBufferMut {
            data: self.data,
            frame_count: self.frame_count,
            channel_count: self.channel_count,
            _lifetime: std::marker::PhantomData,
        }
    }
}
