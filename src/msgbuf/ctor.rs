use super::{owned::OwnedBuf, MsgBuf, MuU8};
use alloc::{boxed::Box, vec::Vec};
use core::{
    mem::{ManuallyDrop, MaybeUninit},
    slice,
    {marker::PhantomData, ptr::NonNull},
};

impl<Owned: OwnedBuf> Default for MsgBuf<'_, Owned> {
    #[inline]
    fn default() -> Self {
        Self {
            ptr: NonNull::dangling(),
            cap: 0,
            init: 0,
            borrow: None,
            own: PhantomData,
            fill: 0,
            has_msg: false,
            quota: None,
        }
    }
}

impl<'slice, Owned: OwnedBuf> MsgBuf<'slice, Owned> {
    /// Forgets old buffer in place, if there was one, and replaces it with the given `owned`.
    pub(super) fn put_owned(&mut self, owned: Owned) {
        let owned = ManuallyDrop::new(owned);
        self.ptr = owned.base_ptr();
        self.cap = owned.capacity();
        self.borrow = None;
        self.init = owned.init_cursor();
        self.fill = 0;
    }
    #[inline]
    fn with_put_owned(mut self, owned: Owned) -> Self {
        self.put_owned(owned);
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

/// Sets `init` = `owned.len()`.
impl<Owned: OwnedBuf> From<Owned> for MsgBuf<'_, Owned> {
    fn from(owned: Owned) -> Self {
        Self::default().with_put_owned(owned)
    }
}

impl<'slice, Owned: OwnedBuf> From<&'slice mut [MaybeUninit<u8>]> for MsgBuf<'slice, Owned> {
    #[inline]
    fn from(borrowed: &'slice mut [MaybeUninit<u8>]) -> Self {
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
impl<'slice, Owned: OwnedBuf> From<&'slice mut [u8]> for MsgBuf<'slice, Owned> {
    #[inline]
    fn from(borrowed: &'slice mut [u8]) -> Self {
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
