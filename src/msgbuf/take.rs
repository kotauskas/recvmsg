use super::{owned::OwnedBuf, MsgBuf, MuU8};
use core::{mem::MaybeUninit, slice};

/// Ownership utilities.
impl<'slice, Owned: OwnedBuf> MsgBuf<'slice, Owned> {
    /// Takes the owned buffer, leaving an empty one in its place. Returns `None` if the buffer is
    /// borrowed, in which case `self` is left untouched. (Zero-sized buffers are considered both
    /// borrowed and owned.)
    #[inline]
    pub fn take_owned(&mut self) -> Option<Owned> {
        let Self { ptr, cap, init, borrow, .. } = *self;
        if borrow.is_some() && cap > 0 {
            return None;
        }
        self.put_owned(Owned::default());
        Some(unsafe { Owned::from_raw_parts(ptr, cap, init) })
    }

    /// Takes the slice and returns it with its original lifetime (regardless of what lifetime
    /// `self` has), leaving the buffer empty. Returns `None` if the buffer is owned, in which case
    /// `self` is left untouched. (Zero-sized buffers are considered both borrowed and owned.)
    #[inline]
    pub fn take_borrowed(&mut self) -> Option<&'slice mut [MaybeUninit<u8>]> {
        let Self { ptr, cap, borrow, .. } = *self;
        if borrow.is_none() && cap > 0 {
            return None;
        };
        self.put_owned(Owned::default());
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
}
