use super::{MsgBuf, MuU8};
use alloc::vec::Vec;
use core::{mem::MaybeUninit, slice};

/// Ownership utilities.
impl<'buf> MsgBuf<'buf> {
    /// Takes the owned buffer, leaving an empty one in its place. Returns `None` if the buffer is
    /// borrowed, in which case `self` is left untouched. (Zero-sized buffers are considered both
    /// borrowed and owned.)
    #[inline]
    pub fn take_owned(&mut self) -> Option<Vec<u8>> {
        #[rustfmt::skip] let Self { ptr, cap, fill, borrow, .. } = *self;
        if borrow.is_some() && cap > 0 {
            return None;
        }
        self.clear();
        Some(unsafe { Vec::from_raw_parts(ptr.as_ptr(), fill, cap) })
    }

    /// Takes the slice and returns it with its original lifetime (regardless of what lifetime
    /// `self` has), leaving the buffer empty. Returns `None` if the buffer is owned, in which case
    /// `self` is left untouched. (Zero-sized buffers are considered both borrowed and owned.)
    #[inline]
    pub fn take_borrowed(&mut self) -> Option<&'buf mut [MaybeUninit<u8>]> {
        let Self { ptr, cap, borrow, .. } = *self;
        #[allow(clippy::question_mark)]
        if borrow.is_none() && cap > 0 {
            return None;
        };
        self.clear();
        // SAFETY: `self` at this point is empty, and the slice we're getting here is a direct
        // descendant (reborrow) of that slice, which means that this is the only instance of that
        // slice in existence at this level in the borrow stack (I *think* that's how this sort of
        // thing is called). What we're essentially doing here is avoiding a reborrow of `self` and
        // instead "unearthing" the 'buf lifetime within the return value.
        //
        // pizzapants184 told me on RPLCS (in #dark-arts) that Polonius would be smart enough to
        // allow this in safe code (and also kindly provided me with a snippet which this whole
        // function is based on). I haven't tried using the `polonius_the_crab` crate because that's
        // a whole extra dependency, but it should be doable with that crate if need be.
        Some(unsafe { slice::from_raw_parts_mut(ptr.as_ptr().cast::<MuU8>(), cap) })
    }

    fn clear(&mut self) {
        (self.init, self.fill, self.cap, self.has_msg) = (0, 0, 0, false);
    }
}
