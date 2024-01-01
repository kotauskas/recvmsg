use super::{MsgBuf, MuU8};
use alloc::{boxed::Box, vec::Vec};
use core::{
    mem::{ManuallyDrop, MaybeUninit},
    slice,
    {marker::PhantomData, ptr::NonNull},
};

impl Default for MsgBuf<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            ptr: NonNull::dangling(),
            cap: 0,
            init: 0,
            borrow: None,
            fill: 0,
            has_msg: false,
            quota: None,
        }
    }
}

impl<'slice> MsgBuf<'slice> {
    /// Forgets old buffer in place, if there was one, and replaces it with the given `vec`.
    pub(super) fn put_vec(&mut self, vec: Vec<u8>) {
        let mut vec = ManuallyDrop::new(vec);
        self.ptr = NonNull::new(vec.as_mut_ptr()).unwrap_or(NonNull::dangling());
        self.cap = vec.capacity();
        self.borrow = None;
        self.init = vec.len();
        self.fill = 0;
    }
    #[inline]
    fn with_put_vec(mut self, vec: Vec<u8>) -> Self {
        self.put_vec(vec);
        self
    }
    /// Forgets old buffer in place, if there was one, and replaces it with the given `slice`.
    fn put_slice(&mut self, slice: &'slice mut [MuU8]) {
        self.ptr = NonNull::new(slice.as_mut_ptr().cast()).unwrap_or(NonNull::dangling());
        self.cap = slice.len();
        self.borrow = Some(PhantomData);
        self.init = 0;
        self.fill = 0;
    }
    #[inline]
    fn with_put_slice(mut self, slice: &'slice mut [MuU8]) -> Self {
        self.put_slice(slice);
        self
    }
}

/// Sets `init` = `vec.len()`.
impl From<Vec<u8>> for MsgBuf<'_> {
    #[inline]
    fn from(vec: Vec<u8>) -> Self {
        Self::default().with_put_vec(vec)
    }
}

impl<'buf> From<&'buf mut [MaybeUninit<u8>]> for MsgBuf<'buf> {
    #[inline]
    fn from(borrowed: &'buf mut [MaybeUninit<u8>]) -> Self {
        Self::default().with_put_slice(borrowed)
    }
}
impl From<Box<[MaybeUninit<u8>]>> for MsgBuf<'_> {
    fn from(bx: Box<[MaybeUninit<u8>]>) -> Self {
        let mut muvec = ManuallyDrop::new(Vec::from(bx));
        unsafe { Vec::from_raw_parts(muvec.as_mut_ptr().cast::<u8>(), 0, muvec.capacity()) }.into()
    }
}

/// Sets `init` = `borrowed.len()`.
impl<'buf> From<&'buf mut [u8]> for MsgBuf<'buf> {
    #[inline]
    fn from(borrowed: &'buf mut [u8]) -> Self {
        let (base, len) = (borrowed.as_mut_ptr(), borrowed.len());
        let mut slf: Self = unsafe { slice::from_raw_parts_mut(base.cast::<MuU8>(), len) }.into();
        unsafe { slf.set_init(slf.cap) };
        slf
    }
}
/// Sets `init` = `bx.len()`.
impl From<Box<[u8]>> for MsgBuf<'_> {
    #[inline]
    fn from(bx: Box<[u8]>) -> Self {
        Vec::from(bx).into()
    }
}
