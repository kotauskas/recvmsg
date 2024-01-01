use super::{MsgBuf, MuU8, QuotaExceeded};
use core::cmp::max;

fn relax_init_slice(slice: &[u8]) -> &[MuU8] {
    unsafe { core::mem::transmute(slice) }
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
