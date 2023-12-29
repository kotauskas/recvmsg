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
        #[rustfmt::skip] let Self { cap: old_cap, quota, has_msg: is_one_msg, .. } = *self;
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
        self.init = self.fill; // Avoids unnecessary copying of unfilled but initialized data
        let vec = if let Some(mut vec) = self.take_owned() {
            // Assumes `Vec`'s only heuristic is exponential growth, averting it if on track to
            // exceed the quota.
            let exp_would_overshoot = quota.map(|quota| cap * 2 > quota.get()).unwrap_or(false);
            let op = if exp_would_overshoot { Vec::reserve_exact } else { Vec::reserve };
            let incr = cap - vec.len();
            op(&mut vec, incr);
            vec
        } else {
            let mut vec = Vec::with_capacity(cap);
            vec.extend_from_slice(self.filled_part());
            vec
        };
        *self = Self::from(vec);
        self.quota = quota;
        self.has_msg = is_one_msg;
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
