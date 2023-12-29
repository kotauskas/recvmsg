use super::{MsgBuf, MuU8};
use core::{
    mem::transmute,
    ops::{Deref, DerefMut},
    slice,
};

/// Pointer arithmetic.
impl MsgBuf<'_> {
    fn base_uninit(&self) -> *const MuU8 {
        self.ptr.as_ptr().cast_const().cast()
    }
    fn base_uninit_mut(&mut self) -> *mut MuU8 {
        self.ptr.as_ptr().cast()
    }
    fn unfilled_start_mut(&mut self) -> *mut MuU8 {
        unsafe {
            // SAFETY: filled <= cap
            self.base_uninit_mut().add(self.fill)
        }
    }
    fn uninit_start_mut(&mut self) -> *mut MuU8 {
        unsafe {
            // SAFETY: init <= cap
            self.base_uninit_mut().add(self.init)
        }
    }
}

unsafe fn assume_init_slice(slice: &[MuU8]) -> &[u8] {
    unsafe { transmute(slice) }
}
unsafe fn assume_init_slice_mut(slice: &mut [MuU8]) -> &mut [u8] {
    unsafe { transmute(slice) }
}

/// Borrows the whole buffer.
///
/// Not particularly useful, although the resulting slice can be sub-sliced and transmuted. Exists
/// primarily due to the bound on `DerefMut`.
impl Deref for MsgBuf<'_> {
    type Target = [MuU8];
    #[inline]
    fn deref(&self) -> &[MuU8] {
        unsafe { slice::from_raw_parts(self.base_uninit(), self.cap) }
    }
}
/// Borrows the whole buffer.
impl DerefMut for MsgBuf<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [MuU8] {
        unsafe { slice::from_raw_parts_mut(self.base_uninit_mut(), self.cap) }
    }
}

/// Safe immutable access.
impl MsgBuf<'_> {
    /// Returns the most recently received message, or an empty slice if no message has been
    /// received yet.
    #[inline]
    pub fn msg(&self) -> Option<&[u8]> {
        if self.has_msg {
            Some(self.filled_part())
        } else {
            None
        }
    }
}

/// Parts, slicing and splitting.
impl MsgBuf<'_> {
    /// Borrows the filled part of the buffer.
    #[inline]
    pub fn filled_part(&self) -> &[u8] {
        unsafe { assume_init_slice(&self[..self.fill]) }
    }
    /// Mutably borrows the filled part of the buffer.
    #[inline]
    pub fn filled_part_mut(&mut self) -> &mut [u8] {
        let fill = self.fill;
        unsafe { assume_init_slice_mut(&mut self[..fill]) }
    }

    /// Mutably borrows the part of the buffer which is initialized but unfilled.
    #[inline]
    pub fn init_but_unfilled_part_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(
                self.unfilled_start_mut().cast(),
                self.len_init_but_unfilled(),
            )
        }
    }

    /// Borrows the initialized (but potentially partially unfilled) part of the buffer.
    #[inline]
    pub fn init_part(&self) -> &[u8] {
        unsafe { assume_init_slice(&self[..self.init]) }
    }
    /// Mutably borrows the initialized (but potentially partially unfilled) part of the buffer.
    #[inline]
    pub fn init_part_mut(&mut self) -> &mut [u8] {
        let init = self.init;
        unsafe { assume_init_slice_mut(&mut self[..init]) }
    }

    /// Splits the buffer into an initialized part and another `MsgBuf` that borrows from `self`.
    ///
    /// This is a bit of a low-level method, since it does not deal with message boundaries.
    #[inline]
    pub fn split_at_init(&mut self) -> (&[u8], MsgBuf<'_>) {
        let rh = unsafe { slice::from_raw_parts_mut(self.uninit_start_mut(), self.len_uninit()) };
        (self.init_part(), rh.into())
    }

    /// Splits the buffer into a filled part and another `MsgBuf` that borrows from `self`.
    #[inline]
    pub fn split_at_fill(&mut self) -> (&[u8], MsgBuf<'_>) {
        let mut rh: MsgBuf<'_> =
            unsafe { slice::from_raw_parts_mut(self.unfilled_start_mut(), self.len_unfilled()) }
                .into();
        unsafe {
            // SAFETY: this is the well-initialized but unfilled part.
            rh.set_init(self.len_init_but_unfilled());
        }
        (self.filled_part(), rh)
    }

    /// Borrows the uninitialized part of the buffer.
    ///
    /// If you need this to be a buffer object, use `.split_at_init().1`.
    #[inline]
    pub fn uninit_part(&mut self) -> &mut [MuU8] {
        unsafe { slice::from_raw_parts_mut(self.uninit_start_mut(), self.len_uninit()) }
    }
    /// Borrows the unfilled part of the buffer.
    ///
    /// If you need this to be a buffer object, use `.split_at_fill().1`.
    #[inline]
    pub fn unfilled_part(&mut self) -> &mut [MuU8] {
        unsafe { slice::from_raw_parts_mut(self.unfilled_start_mut(), self.len_unfilled()) }
    }
}
