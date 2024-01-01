use super::{super::QuotaExceeded, MsgBuf};
use alloc::vec::Vec;
use core::{cmp::max, mem::size_of, num::NonZeroUsize};

// TODO amortize manually

/// Capacity and reallocation.
impl MsgBuf<'_> {
    /// Returns the buffer's total capacity, including the already filled part.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.cap
    }
    /// Ensures that the buffer has at least the given capacity, allocating if necessary.
    pub fn ensure_capacity(&mut self, cap: usize) -> Result<(), QuotaExceeded> {
        let Self { cap: old_cap, quota, .. } = *self;
        let new_cap = max(cap, size_of::<MsgBuf>());
        if old_cap >= new_cap {
            return Ok(());
        }
        if let Some(quota) = quota {
            if new_cap > quota {
                return Err(QuotaExceeded {
                    quota,
                    attempted_alloc: NonZeroUsize::new(new_cap).unwrap(),
                });
            }
        }
        self.init = 0; // Avoids unnecessary copying
        self.fill = 0;
        let vec = if let Some(mut vec) = self.take_owned() {
            // Assumes `Vec`'s only heuristic is exponential growth, averting it if on track to
            // exceed the quota.
            let exp_would_overshoot = quota.map(|quota| new_cap * 2 > quota).unwrap_or(false);
            let op = if exp_would_overshoot { Vec::reserve_exact } else { Vec::reserve };
            let incr = new_cap - vec.len();
            op(&mut vec, incr);
            vec
        } else {
            Vec::with_capacity(new_cap)
        };
        self.put_vec(vec);
        Ok(())
    }
}
