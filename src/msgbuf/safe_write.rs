use super::{MsgBuf, MuU8, OwnedBuf, QuotaExceeded};

fn relax_init_slice(slice: &[u8]) -> &[MuU8] {
    unsafe { core::mem::transmute(slice) }
}

/// Writing from safe code.
impl<Owned: OwnedBuf> MsgBuf<'_, Owned> {
    /// Appends the given slice to the end of the filled part, allocating if necessary.
    pub fn extend_from_slice(&mut self, slice: &[u8]) -> Result<(), QuotaExceeded> {
        let extra = slice.len();
        let new_len = self.fill + extra;
        self.grow_to(new_len)?;
        self.unfilled_part()[..extra].copy_from_slice(relax_init_slice(slice));
        unsafe { self.advance_init_and_set_fill(new_len) };
        Ok(())
    }
}
