use super::{super::QuotaExceeded, owned::OwnedBuf, MsgBuf};
use core::{
    cmp::{max, min},
    mem::size_of,
    num::NonZeroUsize,
};

/// Capacity and reallocation.
impl<Owned: OwnedBuf> MsgBuf<'_, Owned> {
    /// Returns the buffer's total capacity, including the already filled part.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    fn plan_grow_amortized(&self, target: NonZeroUsize) -> Result<usize, QuotaExceeded> {
        let quota = self.quota.unwrap_or(isize::MAX as usize);
        // The growth function. Grows exponentially and tries to never go under twice size of the
        // struct itself to prevent laughably small allocations.
        let grown_wrt_cap = max(target.get(), self.cap * 2);
        let grown = max(grown_wrt_cap, size_of::<MsgBuf>() * 2);
        let new_cap = min(grown, quota);
        if new_cap < target.get() {
            Err(QuotaExceeded { quota, attempted_alloc: target })
        } else {
            Ok(new_cap)
        }
    }

    /// Ensures that the buffer has at least the given capacity, allocating if necessary.
    pub fn ensure_capacity(&mut self, new_cap: usize) -> Result<(), QuotaExceeded> {
        let old_cap = self.cap;
        let new_cap_exact =
            if let (true, Some(new_cap)) = (new_cap > old_cap, NonZeroUsize::new(new_cap)) {
                self.plan_grow_amortized(new_cap)?
            } else {
                return Ok(());
            };
        self.init = 0; // Avoids unnecessary copying
        self.fill = 0;
        let mut owned = self.take_owned().unwrap_or_default();
        owned.grow(new_cap_exact);
        self.put_owned(owned);
        Ok(())
    }
}
