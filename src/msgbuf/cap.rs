use super::MsgBuf;
use alloc::vec::Vec;
use core::{
    cmp::max,
    fmt::{self, Display, Formatter},
    mem::size_of,
    num::NonZeroUsize,
};

/// Capacity and reallocation.
impl MsgBuf<'_> {
    /// Returns the buffer's total capacity, including the already filled part.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.cap
    }
    /// Ensures that the buffer has at least the given capacity, allocating if necessary.
    pub fn ensure_capacity(&mut self, cap: usize) -> Result<(), QuotaExceeded> {
        #[rustfmt::skip] let Self { cap: old_cap, quota, is_one_msg, .. } = *self;
        let cap = max(cap, size_of::<MsgBuf>());
        if old_cap >= cap {
            return Ok(());
        }
        if let Some(quota) = quota {
            if cap > quota.get() {
                return Err(QuotaExceeded {
                    quota,
                    attempted_alloc: NonZeroUsize::new(cap).unwrap(),
                });
            }
        }
        self.init = self.fill;
        let vec = if let Some(mut vec) = self.take_owned() {
            vec.reserve(cap - self.init);
            vec
        } else {
            let mut vec = Vec::with_capacity(cap);
            vec.extend_from_slice(self.filled_part());
            vec
        };
        *self = Self::from(vec);
        self.quota = quota;
        self.is_one_msg = is_one_msg;
        Ok(())
    }
}

/// Error indicating that a buffer's memory allocation quota was exceeded during an operation that
/// had to perform a memory allocation.
#[derive(Copy, Clone, Debug)]
pub struct QuotaExceeded {
    /// The quota the buffer had at the time of the error.
    pub quota: NonZeroUsize,
    /// The size which the buffer was to attain.
    pub attempted_alloc: NonZeroUsize,
}
impl Display for QuotaExceeded {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        #[rustfmt::skip] let Self { quota, attempted_alloc } = self;
        write!(
            f,
            "quota of {quota} bytes exceeded by an attempted buffer reallocation to {attempted_alloc} bytes"
        )
    }
}
#[cfg(feature = "std")]
impl std::error::Error for QuotaExceeded {}
