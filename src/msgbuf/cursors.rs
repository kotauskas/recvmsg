use super::{MsgBuf, MuU8};

/// Cursors of the buffer.
impl MsgBuf<'_> {
    /// Returns the length of the filled part, which is numerically equal to the offset from the
    /// *base* pointer at which the unfilled part starts.
    #[inline(always)]
    pub fn len_filled(&self) -> usize {
        self.fill
    }
    /// Returns the length of the initialized part, which is numerically equal to the offset from
    /// the *base* pointer at which the uninitialized part starts.
    #[inline(always)]
    pub fn len_init(&self) -> usize {
        self.init
    }
    /// Returns the length of the unfilled but initialized part, which is numerically equal to the
    /// offset from the *unfilled part start* pointer at which the uninitialized part starts.
    #[inline(always)]
    pub fn len_init_but_unfilled(&self) -> usize {
        self.init - self.fill
    }
    /// Returns the length of the unfilled part, including also the uninitialized one (because
    /// everything that is uninitialized is also unfilled).
    #[inline(always)]
    pub fn len_unfilled(&self) -> usize {
        self.cap - self.fill
    }
    /// Returns the length of the uninitialized part.
    #[inline(always)]
    pub fn len_uninit(&self) -> usize {
        self.cap - self.init
    }

    /// Fully initializes the buffer with zeroes.
    #[inline]
    pub fn fully_initialize(&mut self) {
        self.uninit_part().fill(MuU8::new(0));
        unsafe { self.set_init(self.cap) }
    }

    /// Sets the initialization cursor of the buffer to the given value.
    ///
    /// # Safety
    /// - The given amount of bytes after the prior initialization cursor **must** be well-initialized.
    ///   - This also implies that `new_len` may not exceed the capacity.
    #[inline]
    pub unsafe fn set_init(&mut self, new_init: usize) {
        assert!(
            new_init <= self.cap,
            "attempt to advance buffer initialization cursor past the capacity limit",
        );
        self.init = new_init;
    }
    /// Sets the fill cursor of the buffer to the given value.
    ///
    /// # Panics
    /// If the given length exceeds the initialization cursor.
    #[inline]
    pub fn set_fill(&mut self, new_len: usize) {
        assert!(
            new_len <= self.init,
            "attempt to advance buffer fill cursor past the initialized part"
        );
        self.fill = new_len;
    }
}
