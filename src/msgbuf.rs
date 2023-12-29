#![allow(unsafe_code)]

mod cap;
pub use cap::QuotaExceeded;
mod conv;
mod cursors;
mod slicing;
mod take;
#[cfg(test)]
mod tests;

use alloc::vec::Vec;
use core::{
    cmp::max,
    marker::PhantomData,
    mem::{transmute, MaybeUninit},
    num::NonZeroUsize,
    panic::UnwindSafe,
    ptr::NonNull,
};

type MuU8 = MaybeUninit<u8>;

/// A message reception buffer that can either contain a borrowed slice or own a `Vec`.
///
/// This type can be created from a buffer that lives on the stack:
/// ```
/// # use {core::mem::MaybeUninit, recvmsg::MsgBuf};
/// // An uninitialized buffer:
/// let mut arr = [MaybeUninit::new(0); 32];
/// let buf = MsgBuf::from(arr.as_mut());
/// assert_eq!(buf.capacity(), 32);
/// assert_eq!(buf.init_part().len(), 0); // Assumes nothing about the buffer.
/// assert_eq!(buf.filled_part().len(), 0);
/// assert!(!buf.is_one_msg); // No message is contained.
///
/// // A fully initialized and filled buffer:
/// let mut arr = [0; 32];
/// let buf = MsgBuf::from(arr.as_mut());
/// assert_eq!(buf.capacity(), 32);
/// // Whole buffer can be passed to methods that take &mut [u8]:
/// assert_eq!(buf.init_part().len(), 32);
/// // Whole buffer is assumed to be filled (this
/// // does not interfere with subsequent reception):
/// assert_eq!(buf.filled_part().len(), 32);
/// assert!(buf.is_one_msg); // Assumed to already contain a single received message.
/// ```
///
/// Or one on the heap via [`Box`]:
/// ```
/// # extern crate alloc;
/// # use {alloc::boxed::Box, core::mem::MaybeUninit, recvmsg::MsgBuf};
/// // An uninitialized buffer (yes, the annotations are necessary):
/// let buf = MsgBuf::from(Box::<[MaybeUninit<_>]>::from([MaybeUninit::new(0); 32]));
/// assert_eq!(buf.capacity(), 32);
/// assert_eq!(buf.len_init(), 0);
/// assert_eq!(buf.len_filled(), 0);
/// assert!(!buf.is_one_msg);
///
/// // A fully initialized and filled buffer:
/// let buf = MsgBuf::from(Box::<[u8]>::from([0; 32]));
/// assert_eq!(buf.capacity(), 32);
/// assert_eq!(buf.len_init(), 32);
/// assert_eq!(buf.len_filled(), 32);
/// assert!(buf.is_one_msg);
/// ```
///
/// Or in a `Vec`:
/// ```
/// # extern crate alloc;
/// # use {alloc::vec::Vec, core::mem::MaybeUninit, recvmsg::MsgBuf};
/// // An uninitialized buffer:
/// let buf = MsgBuf::from(Vec::with_capacity(31)); // Size can be odd too!
/// assert_eq!(buf.capacity(), 31);
/// assert_eq!(buf.len_init(), 0);
/// assert_eq!(buf.len_filled(), 0);
/// assert!(!buf.is_one_msg);
///
/// // A partially initialized buffer:
/// let mut vec = Vec::with_capacity(32);
/// vec.resize(6, 0);
/// let buf = MsgBuf::from(vec);
/// assert_eq!(buf.capacity(), 32);
/// assert_eq!(buf.len_init(), 6);
/// assert_eq!(buf.len_filled(), 6);
/// assert!(buf.is_one_msg);
/// ```
#[derive(Debug)]
pub struct MsgBuf<'buf> {
    ptr: NonNull<u8>,
    // All cursors count from `ptr`, not from each other.
    /// How much is allocated.
    cap: usize,
    /// How much is initialized. Numerically equal to the starting index of writes. May not exceed
    /// `cap`.
    init: usize,
    /// The length of the logically filled part of the buffer. Usually equal to the length of the
    /// last received message. May not exceed `init`.
    fill: usize,
    /// Designates whether the buffer is borrowed or owned. `Option` is completely decorative and
    /// acts as a fancy boolean here.
    borrow: Option<PhantomData<&'buf mut [MuU8]>>,
    /// Highest allowed capacity for growth operations. This will only take effect on the next
    /// memory allocation.
    ///
    /// Note that `Vec` may slightly overshoot this quota due to amortization heuristics or simply
    /// due to the allocator providing excess capacity. This is accounted for via use of
    /// [`Vec::reserve_exact()`] instead of [`Vec::reserve()`] when the requested capacity is within
    /// a factor of two from the quota, which is modelled after `Vec`'s actual exponential growth
    /// behavior and thus should prevent overshoots in all but the most exceptional of situations.
    pub quota: Option<NonZeroUsize>,
    /// Whether the well-initialized part of the buffer constitutes exactly one message.
    pub is_one_msg: bool,
}
impl UnwindSafe for MsgBuf<'_> {} // Who else remembers that this trait is a thing?
impl Default for MsgBuf<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            ptr: NonNull::dangling(),
            cap: 0,
            quota: None,
            init: 0,
            fill: 0,
            borrow: None,
            is_one_msg: false,
        }
    }
}

impl Drop for MsgBuf<'_> {
    fn drop(&mut self) {
        self.take_owned(); // If owned, returns `Some(vec)`, which is then dropped.
    }
}

fn relax_init_slice(slice: &[u8]) -> &[MuU8] {
    unsafe { transmute(slice) }
}

/// Writing from safe code.
impl MsgBuf<'_> {
    /// Appends the given slice to the end of the filled part, allocating if necessary.
    pub fn extend_from_slice(&mut self, slice: &[u8]) -> Result<(), QuotaExceeded> {
        let extra = slice.len();
        let new_len = self.fill + extra;
        self.ensure_capacity(new_len)?;
        self.unfilled_part()[..extra].copy_from_slice(relax_init_slice(slice));
        // The slice might be smaller than the unfilled-but-initialized part, in which case passing
        // `new_len` would inexplicably move the initialization cursor back.
        unsafe { self.set_init(max(new_len, self.init)) };
        self.set_fill(new_len);
        Ok(())
    }
}

/// Lifetime management.
impl MsgBuf<'_> {
    /// Makes sure `self` is owned by making a new allocation equal in size to the borrowed
    /// capacity if it is borrowed.
    #[rustfmt::skip] // FUCK off
    pub fn make_owned(self) -> MsgBuf<'static> {
        if self.borrow.is_none() {
            let MsgBuf { ptr, cap, quota, init, fill: len, borrow: _, is_one_msg } = self;
            MsgBuf { ptr, cap, quota, init, fill: len, borrow: None, is_one_msg }
        } else {
            MsgBuf::from(Vec::with_capacity(self.cap))
        }
    }
    /// Attempts to extend lifetime to `'static`, failing if the buffer is borrowed.
    #[rustfmt::skip]
    pub fn try_extend_lifetime(self) -> Result<MsgBuf<'static>, Self> {
        if self.borrow.is_none() || self.cap == 0 {
            let MsgBuf { ptr, cap, quota, init, fill: len, borrow: _, is_one_msg } = self;
            Ok(MsgBuf { ptr, cap, quota, init, fill: len, borrow: None, is_one_msg })
        } else {
            Err(self)
        }
    }
}
