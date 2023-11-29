use super::MsgBuf;
use alloc::{boxed::Box, vec::Vec};
use core::{
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
    ptr::NonNull,
};

/// Sets `fill` = `init` = `vec.len()`.
impl From<Vec<u8>> for MsgBuf<'_> {
    #[inline]
    fn from(vec: Vec<u8>) -> Self {
        let mut vec = ManuallyDrop::new(vec);
        Self {
            ptr: NonNull::new(vec.as_mut_ptr()).unwrap_or(NonNull::dangling()),
            cap: vec.capacity(),
            quota: None,
            init: vec.len(),
            fill: vec.len(),
            borrow: None,
            is_one_msg: !vec.is_empty(),
        }
    }
}

impl<'buf> From<&'buf mut [MaybeUninit<u8>]> for MsgBuf<'buf> {
    #[inline]
    fn from(borrowed: &'buf mut [MaybeUninit<u8>]) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(borrowed.as_mut_ptr().cast()) },
            cap: borrowed.len(),
            quota: None,
            init: 0,
            fill: 0,
            borrow: Some(PhantomData),
            is_one_msg: false,
        }
    }
}
impl From<Box<[MaybeUninit<u8>]>> for MsgBuf<'_> {
    fn from(bx: Box<[MaybeUninit<u8>]>) -> Self {
        let mut bx = ManuallyDrop::new(bx);
        Self {
            ptr: NonNull::new(bx.as_mut_ptr()).unwrap_or(NonNull::dangling()).cast(),
            cap: bx.len(),
            quota: None,
            init: 0,
            fill: 0,
            borrow: None,
            is_one_msg: false,
        }
    }
}

/// Sets `fill` = `init` = `borrowed.len()`.
impl<'buf> From<&'buf mut [u8]> for MsgBuf<'buf> {
    #[inline]
    fn from(borrowed: &'buf mut [u8]) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(borrowed.as_mut_ptr().cast()) },
            cap: borrowed.len(),
            quota: None,
            init: borrowed.len(),
            fill: borrowed.len(),
            borrow: Some(PhantomData),
            is_one_msg: !borrowed.is_empty(),
        }
    }
}
/// Sets `fill` = `init` = `bx.len()`.
impl From<Box<[u8]>> for MsgBuf<'_> {
    fn from(bx: Box<[u8]>) -> Self {
        let mut bx = ManuallyDrop::new(bx);
        Self {
            ptr: NonNull::new(bx.as_mut_ptr()).unwrap_or(NonNull::dangling()),
            cap: bx.len(),
            quota: None,
            init: bx.len(),
            fill: bx.len(),
            borrow: None,
            is_one_msg: !bx.is_empty(),
        }
    }
}
