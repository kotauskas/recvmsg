use core::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
    mem::ManuallyDrop,
    ptr::NonNull,
};

/// Owned buffers for use with [`MsgBuf`](super::MsgBuf).
///
/// Implementors can only be characterized by their three properties they're constructed from and
/// deconstructed into:
/// - Base pointer, the start of the buffer and the handle for reallocation and deallocation
/// - Capacity, the length of the buffer
/// - Initialization cursor, the number of bytes at the beginning of the buffer which can be assumed
///   to be well-initialized, and are to be retained when growing
///
/// # Safety
/// - For an `OwnedBuf` with capacity 𝑐, the first 𝑐 bytes starting from the base pointer must be
///   valid for reads of `MaybeUninit<u8>` given a shared reference to it and writes given an
///   exclusive reference.
/// - The base pointer may not change unless `.grow()` is called.
/// - Capacity may not spuriously decrease.
/// - For an `OwnedBuf` with init cursor 𝑖, after a call to `.grow()`, the first 𝑖 bytes starting
///   from the base pointer must match the corresponding values before the call. In other words,
///   growth must retain the contents of the initialized part.
pub unsafe trait OwnedBuf: Default + Sized {
    /// Creates the owned buffer from its base pointer, capacity and the initialization cursor.
    ///
    /// # Safety
    /// - If `cap` is non-zero, `ptr` must be a value returned by an `into_raw_parts()` call on the
    ///   same type.
    /// - `ptr` must not be owned by any other instance of the type.
    /// - `init` must not exceed `cap`.
    unsafe fn from_raw_parts(ptr: NonNull<u8>, cap: usize, init: usize) -> Self;
    /// Returns the base pointer of the buffer.
    fn base_ptr(&self) -> NonNull<u8>;
    /// Returns the capacity of the buffer.
    fn capacity(&self) -> usize;
    /// Returns the initialization cursor of the buffer.
    ///
    /// This many bytes at the beginning are valid for reads.
    fn init_cursor(&self) -> usize;
    /// Grows the buffer up to the given capacity.
    ///
    /// Does not necessarily have to be able to decrease the buffer's capacity.
    fn grow(&mut self, new_cap: usize);
    /// Relinquishes ownership of the buffer and decomposes the object into the raw parts which it
    /// can be later reassembled from.
    #[inline]
    fn into_raw_parts(self) -> (NonNull<u8>, usize, usize) {
        let slf = ManuallyDrop::new(self);
        (slf.base_ptr(), slf.capacity(), slf.init_cursor())
    }
}

unsafe impl OwnedBuf for Vec<u8> {
    #[inline]
    unsafe fn from_raw_parts(ptr: NonNull<u8>, cap: usize, init: usize) -> Self {
        unsafe { Self::from_raw_parts(ptr.as_ptr(), init, cap) }
    }
    #[inline]
    fn base_ptr(&self) -> NonNull<u8> {
        unsafe {
            // SAFETY: Vec base is never null
            NonNull::new_unchecked(self.as_ptr().cast_mut())
        }
    }
    #[inline]
    fn capacity(&self) -> usize {
        self.capacity()
    }
    #[inline]
    fn init_cursor(&self) -> usize {
        self.len()
    }
    fn grow(&mut self, new_cap: usize) {
        let incr = new_cap.saturating_sub(self.len());
        self.reserve_exact(incr)
    }
}

#[rustfmt::skip] unsafe impl OwnedBuf for () {
    #[inline] unsafe fn from_raw_parts(_: NonNull<u8>, _: usize, _: usize) -> Self {}
    #[inline] fn base_ptr(&self) -> NonNull<u8> { NonNull::dangling() }
    #[inline] fn capacity(&self) -> usize { 0 }
    #[inline] fn init_cursor(&self) -> usize { 0 }
    #[inline] fn grow(&mut self, _: usize) {}
}

/// Custom allocation of [owned buffers](OwnedBuf).
///
/// It makes no sense for implementors to not be zero-sized (though they can be never-sized, at your
/// discretion).
pub trait GrowFn {
    /// Grows the buffer up to the given capacity.
    ///
    /// Does not necessarily have to be able to decrease the buffer's capacity.
    fn grow<Owned: OwnedBuf>(owned: &mut Owned, new_cap: usize);
}

/// The default [growth function](GrowFn) of an [owned buffer](OwnedBuf).
pub struct DefaultFn(());
impl GrowFn for DefaultFn {
    #[inline]
    fn grow<Owned: OwnedBuf>(owned: &mut Owned, new_cap: usize) {
        owned.grow(new_cap);
    }
}

/// Applies the given growth function to the given owned buffer type.
pub struct WithGrowFn<Owned, Gfn>(pub Owned, PhantomData<fn(Gfn)>);
#[rustfmt::skip]
unsafe impl<Owned: OwnedBuf, Gfn: GrowFn> OwnedBuf for WithGrowFn<Owned, Gfn> {
    #[inline]
    unsafe fn from_raw_parts(ptr: NonNull<u8>, cap: usize, init: usize) -> Self {
        let owned = unsafe { Owned::from_raw_parts(ptr, cap, init) };
        Self(owned, PhantomData)
    }
    #[inline] fn base_ptr(&self) -> NonNull<u8> { self.0.base_ptr() }
    #[inline] fn capacity(&self) -> usize { self.0.capacity() }
    #[inline] fn init_cursor(&self) -> usize { self.0.init_cursor() }
    #[inline] fn grow(&mut self, new_cap: usize) {
        Gfn::grow(&mut self.0, new_cap);
    }
}
impl<Owned: Default, Gfn> Default for WithGrowFn<Owned, Gfn> {
    #[inline]
    fn default() -> Self {
        Self(Owned::default(), PhantomData)
    }
}
impl<Owned: Debug, Gfn> Debug for WithGrowFn<Owned, Gfn> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("WithGrowFn").field(&self.0).finish()
    }
}
