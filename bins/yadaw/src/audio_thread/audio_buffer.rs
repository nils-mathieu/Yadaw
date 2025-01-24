use std::{mem::forget, ptr::NonNull};

/// An exclusive reference to a collection of buffers that contain audio data.
///
/// # Data layout
///
/// The audio data is stored in a "planar" layout, meaning that each channel has its own buffer
/// which contains all the frames for that channel.
pub struct AudioBufferMut<'a, T = f32> {
    data: &'a [*mut T],
    frame_count: usize,
}

unsafe impl<T: Send> Send for AudioBufferMut<'_, T> {}
unsafe impl<T: Sync> Sync for AudioBufferMut<'_, T> {}

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
            data: unsafe { std::slice::from_raw_parts(data, channel_count) },
            frame_count,
        }
    }

    /// Returns the number of channels in the audio buffer.
    #[inline(always)]
    pub fn channel_count(&self) -> usize {
        self.data.len()
    }

    /// Returns the number of frames in each channel of the audio buffer.
    #[inline(always)]
    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    /// Gets the pointer to the frames a particular channel in the audio buffer.
    ///
    /// # Safety
    ///
    /// The caller must make sure that the provided `channel` index is smaller than
    /// `.channel_count()`.
    #[inline]
    pub unsafe fn channel_mut_ptr(&mut self, channel: usize) -> *mut T {
        unsafe { *self.data.get_unchecked(channel) }
    }

    /// Gets a pointer to the frames of a particular channel in the audio buffer.
    ///
    /// # Safety
    ///
    /// The caller must make sure that the provided `channel` index is smaller than
    /// `.channel_count()`.
    #[inline]
    pub unsafe fn channel_ptr(&self, channel: usize) -> *const T {
        unsafe { *self.data.get_unchecked(channel) }
    }

    /// Returns the frames for the channel with the provided index.
    ///
    /// # Safety
    ///
    /// The caller must make sure that the provided `channel` index is smaller than
    /// `.channel_count()`.
    #[inline]
    pub unsafe fn channel_unchecked(&self, channel: usize) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.channel_ptr(channel), self.frame_count) }
    }

    /// Returns the frames for the channel with the provided index.
    ///
    /// # Safety
    ///
    /// The caller must make sure that the provided `channel` index is smaller than
    /// `.channel_count()`.
    #[inline]
    pub unsafe fn channel_unchecked_mut(&mut self, channel: usize) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.channel_mut_ptr(channel), self.frame_count) }
    }

    /// Returns the frames for the channel with the provided index.
    ///
    /// # Returns
    ///
    /// Returns `None` if the provided `channel` index is out of bounds.
    #[inline]
    pub fn channel(&self, channel: usize) -> Option<&[T]> {
        if channel < self.channel_count() {
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
        if channel < self.channel_count() {
            Some(unsafe { self.channel_unchecked_mut(channel) })
        } else {
            None
        }
    }

    /// Returns an iterator over the channels of the audio buffer.
    #[inline]
    pub fn channels(&self) -> impl Iterator<Item = &[T]> + '_ {
        self.data
            .iter()
            .map(move |&p| unsafe { std::slice::from_raw_parts(p, self.frame_count) })
    }

    /// Returns an iterator over the channels of the audio buffer.
    #[inline]
    pub fn channels_mut(&mut self) -> impl Iterator<Item = &mut [T]> + '_ {
        self.data
            .iter()
            .map(move |&p| unsafe { std::slice::from_raw_parts_mut(p, self.frame_count) })
    }

    /// Re-borrows the buffer with a shorter lifetime without consuming the original reference.
    pub fn reborrow(&mut self) -> AudioBufferMut<T> {
        AudioBufferMut {
            data: self.data,
            frame_count: self.frame_count,
        }
    }
}

/// An exclusive reference to a collection of buffers that contain audio data.
///
/// # Data layout
///
/// The audio data is stored in a "planar" layout, meaning that each channel has its own buffer
/// which contains all the frames for that channel.
#[derive(Clone, Copy)]
pub struct AudioBufferRef<'a, T = f32> {
    /// The actual audio data.
    ///
    /// This is a list of `channel_count` buffers, each containing `frame_count` frames.
    data: &'a [*const T],
    /// The number of frames in each buffer.
    frame_count: usize,
}

unsafe impl<T: Sync> Send for AudioBufferRef<'_, T> {}
unsafe impl<T: Sync> Sync for AudioBufferRef<'_, T> {}

impl<T> AudioBufferRef<'_, T> {
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
        data: *const *const T,
        frame_count: usize,
        channel_count: usize,
    ) -> Self {
        Self {
            data: unsafe { std::slice::from_raw_parts(data, channel_count) },
            frame_count,
        }
    }

    /// Returns the number of channels in the audio buffer.
    #[inline(always)]
    pub fn channel_count(&self) -> usize {
        self.data.len()
    }

    /// Returns the number of frames in each channel of the audio buffer.
    #[inline(always)]
    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    /// Returns a pointer to the frames of a particular channel in the audio buffer.
    #[inline]
    pub unsafe fn channel_ptr(&self, channel: usize) -> *const T {
        unsafe { *self.data.get_unchecked(channel) }
    }

    /// Returns the frames for the channel with the provided index.
    ///
    /// # Safety
    ///
    /// The caller must make sure that the provided `channel` index is smaller than
    /// `.channel_count()`.
    #[inline]
    pub unsafe fn channel_unchecked(&self, channel: usize) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.channel_ptr(channel), self.frame_count) }
    }

    /// Returns the frames for the channel with the provided index.
    ///
    /// # Returns
    ///
    /// Returns `None` if the provided `channel` index is out of bounds.
    #[inline]
    pub fn channel(&self, channel: usize) -> Option<&[T]> {
        if channel < self.channel_count() {
            Some(unsafe { self.channel_unchecked(channel) })
        } else {
            None
        }
    }

    /// Returns an iterator over the channels of the audio buffer.
    #[inline]
    pub fn channels(&self) -> impl Iterator<Item = &[T]> + '_ {
        self.data
            .iter()
            .map(move |&p| unsafe { std::slice::from_raw_parts(p, self.frame_count) })
    }
}

/// An owned audio buffer.
///
/// # Data layout
///
/// The audio data is stored in a "planar" layout, meaning that each channel has its own buffer
/// which contains all the frames for that channel.
pub struct AudioBufferOwned<T = f32> {
    /// The actual audio data.
    ///
    /// This is a list of `channel_count` buffers, each containing `frame_count` frames.
    data: NonNull<*mut T>,

    /// The number of channels represented in the audio buffer.
    ///
    /// This is the number of sub-buffers in `data`.
    channel_count: usize,

    /// The number of frames in each sub-buffer.
    ///
    /// This is the number of samples in each sub-buffer.
    frame_count: usize,

    /// The number of samples allocated in each sub-buffer.
    cap: usize,

    /// Tells the drop checker that this type owns a `T`.
    _marker: std::marker::PhantomData<T>,
}

unsafe impl<T: Send> Send for AudioBufferOwned<T> {}
unsafe impl<T: Sync> Sync for AudioBufferOwned<T> {}

impl<T> AudioBufferOwned<T> {
    /// Creates a new audio buffer with the provided number of channels and frames.
    ///
    /// # Panics
    ///
    /// This function will panic if the allocation fails.
    pub fn new(channel_count: usize) -> Self {
        unsafe {
            let layout = std::alloc::Layout::array::<*mut T>(channel_count)
                .unwrap_or_else(|_| capacity_overflow());
            let data = std::alloc::alloc_zeroed(layout) as *mut *mut T;
            let data = NonNull::new(data).unwrap_or_else(|| std::alloc::handle_alloc_error(layout));

            Self {
                data,
                channel_count,
                frame_count: 0,
                cap: 0,
                _marker: std::marker::PhantomData,
            }
        }
    }

    /// Returns the current capacity of the audio buffer.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    /// Returns the channel count of the audio buffer.
    #[inline]
    pub fn channel_count(&self) -> usize {
        self.channel_count
    }

    /// Returns the frame count of the audio buffer.
    #[inline]
    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    /// Ensures that the provided capacity is available in the audio buffer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `new_cap > self.capapacty()`.
    pub unsafe fn ensure_capacity_unchecked(&mut self, new_cap: usize) {
        /// A guard that frees all channels when dropped.
        ///
        /// This is required because we *always* need to make sure that all channels have the
        /// same length.
        ///
        /// If an allocation fails, the previous channels might have successfully been allocated
        /// with a new length.
        ///
        /// We can't have half the channels with the new length and half with the old length.
        ///
        /// The only infallible way to ensure that the invariant is upheld is to free all channels
        /// so that they all have a length of zero.
        struct AllocGuard<'a, T> {
            data: &'a mut [*mut T],
            prev_cap: usize,
            new_cap: usize,

            /// Channels before this index have been successfully allocated with the new capacity.
            /// Channels after (and at) this index have not yet been reallocated.
            cursor: usize,
        }

        impl<T> Drop for AllocGuard<'_, T> {
            fn drop(&mut self) {
                for i in 0..self.cursor {
                    unsafe {
                        std::alloc::dealloc(
                            *self.data.get_unchecked(i) as *mut u8,
                            std::alloc::Layout::array::<T>(self.new_cap).unwrap_unchecked(),
                        );
                    }
                }

                if self.prev_cap != 0 {
                    for i in self.cursor..self.data.len() {
                        unsafe {
                            std::alloc::dealloc(
                                *self.data.get_unchecked(i) as *mut u8,
                                std::alloc::Layout::array::<T>(self.prev_cap).unwrap_unchecked(),
                            );
                        }
                    }
                }
            }
        }

        let mut guard = AllocGuard {
            data: unsafe { std::slice::from_raw_parts_mut(self.data.as_ptr(), self.channel_count) },
            prev_cap: self.cap,
            new_cap,
            cursor: 0,
        };

        unsafe {
            if self.cap == 0 {
                // First allocation.

                while guard.cursor < self.channel_count {
                    let layout = std::alloc::Layout::array::<T>(new_cap)
                        .unwrap_or_else(|_| capacity_overflow());
                    let ptr = std::alloc::alloc(layout) as *mut T;
                    if ptr.is_null() {
                        std::alloc::handle_alloc_error(layout);
                    }

                    *guard.data.get_unchecked_mut(guard.cursor) = ptr;
                    guard.cursor += 1;
                }
            } else {
                // Reallocate.

                while guard.cursor < self.channel_count {
                    let old_layout = std::alloc::Layout::array::<T>(self.cap).unwrap_unchecked();
                    let ptr = std::alloc::realloc(
                        *guard.data.get_unchecked(guard.cursor) as *mut u8,
                        old_layout,
                        std::mem::size_of::<T>()
                            .checked_mul(new_cap)
                            .unwrap_or_else(|| capacity_overflow()),
                    ) as *mut T;
                    if ptr.is_null() {
                        std::alloc::handle_alloc_error(old_layout);
                    }

                    *guard.data.get_unchecked_mut(guard.cursor) = ptr;
                    guard.cursor += 1;
                }
            }
        }

        self.cap = new_cap;
        forget(guard);
    }

    /// Ensures that at least `additional` frames can be added to the audio buffer without
    /// reallocating.
    pub fn reserve(&mut self, additional: usize) {
        let new_cap = self
            .frame_count
            .checked_add(additional)
            .unwrap_or_else(|| capacity_overflow());
        if new_cap > self.cap {
            unsafe { self.ensure_capacity_unchecked(new_cap) }
        }
    }

    /// Returns a pointer to the frames of a particular channel in the audio buffer.
    ///
    /// # Safety
    ///
    /// The caller must make sure that the provided `channel` index is smaller than
    /// `.channel_count()`.
    #[inline]
    pub unsafe fn channel_ptr(&self, channel: usize) -> *const T {
        unsafe { *self.data.as_ptr().add(channel) }
    }

    /// Returns a pointer to the frames of a particular channel in the audio buffer.
    ///
    /// # Safety
    ///
    /// The caller must make sure that the provided `channel` index is smaller than
    /// `.channel_count()`.
    #[inline]
    pub unsafe fn channel_mut_ptr(&mut self, channel: usize) -> *mut T {
        unsafe { *self.data.as_ptr().add(channel) }
    }

    /// Returns the channel with the provided index.
    ///
    /// # Safety
    ///
    /// The caller must make sure that the provided `channel` index is smaller than
    /// `.channel_count()`.
    #[inline]
    pub unsafe fn channel_unchecked(&self, channel: usize) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.channel_ptr(channel), self.frame_count) }
    }

    /// Returns the channel with the provided index.
    ///
    /// # Safety
    ///
    /// The caller must make sure that the provided `channel` index is smaller than
    /// `.channel_count()`.
    #[inline]
    pub unsafe fn channel_unchecked_mut(&mut self, channel: usize) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.channel_mut_ptr(channel), self.frame_count) }
    }

    /// Returns the channel with the provided index.
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

    /// Returns the channel with the provided index.
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
        (0..self.channel_count).map(move |c| unsafe { self.channel_unchecked(c) })
    }

    /// Returns an iterator over the channels of the audio buffer.
    #[inline]
    pub fn channels_mut(&mut self) -> impl Iterator<Item = &mut [T]> + '_ {
        (0..self.channel_count).map(move |c| unsafe {
            let p = self.channel_mut_ptr(c);
            std::slice::from_raw_parts_mut(p, self.frame_count)
        })
    }

    /// Returns an [`AudioBufferRef`] that references the same audio data.
    #[inline]
    pub fn as_audio_buffer_mut(&mut self) -> AudioBufferMut<T> {
        unsafe {
            AudioBufferMut::from_raw_parts(self.data.as_ptr(), self.frame_count, self.channel_count)
        }
    }

    /// Returns an [`AudioBufferRef`] that references the same audio data.
    #[inline]
    pub fn as_audio_buffer_ref(&self) -> AudioBufferRef<T> {
        unsafe {
            AudioBufferRef::from_raw_parts(
                self.data.as_ptr() as *const *const T,
                self.frame_count,
                self.channel_count,
            )
        }
    }

    /// Extends the audio buffer with the provided audio data.
    ///
    /// # Panics
    ///
    /// This function panics if the provided audio buffer does not have the same number of
    /// planes.
    pub fn extend_from_buf(&mut self, data: AudioBufferRef<T>) {
        assert_eq!(
            self.channel_count,
            data.channel_count(),
            "The number of channels must match",
        );

        unsafe {
            let amount = data.frame_count();
            self.extend_unchecked_by_channel(amount, |c, dst| {
                std::ptr::copy_nonoverlapping(data.channel_ptr(c), dst, amount);
            });
        }
    }

    /// Extends the audio buffer with the provided audio data.
    ///
    ///
    /// # Callback
    ///
    /// The provided callback is called `channel_count` times.
    ///
    /// ```rust,ignore
    /// fn callback(channel: usize, dst: *mut T);
    /// ```
    ///
    /// # Safety
    ///
    /// The caller must make sure that the provided callback does not panic, and that exactly
    /// `amount` frames are written to the provided buffer.
    pub unsafe fn extend_unchecked_by_channel(
        &mut self,
        amount: usize,
        mut cb: impl FnMut(usize, *mut T),
    ) {
        self.reserve(amount);

        for c in 0..self.channel_count {
            unsafe { cb(c, self.channel_mut_ptr(c).add(self.frame_count)) }
        }

        // SAFETY: We checked the result of that operation in `.reserve()`.
        self.frame_count = unsafe { self.frame_count.unchecked_add(amount) };
    }

    /// Extends the audio buffer with the provided audio data.
    ///
    /// # Callback
    ///
    /// The provided callback is called `amount * channel_count` times.
    ///
    /// ```rust,ignore
    /// fn callback(channel: usize, frame: usize) -> T;
    /// ```
    ///
    /// The channel number is bumped when all frames for that channel have been filled.
    ///
    /// # Safety
    ///
    /// The caller must make sure that the provided callback does not panic.
    pub unsafe fn extend_unchecked_by_sample(
        &mut self,
        amount: usize,
        mut cb: impl FnMut(usize, usize) -> T,
    ) {
        unsafe {
            self.extend_unchecked_by_channel(amount, |c, dst| {
                for f in 0..amount {
                    dst.add(f).write(cb(c, f));
                }
            });
        }
    }
}

#[inline(never)]
#[cold]
fn capacity_overflow() -> ! {
    panic!("capacity overflow")
}
