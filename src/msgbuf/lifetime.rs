use super::MsgBuf;

/// Lifetime management.
impl MsgBuf<'_> {
    /// Makes sure `self` is owned by making a new allocation equal in size to the borrowed
    /// capacity if it is borrowed.
    pub fn make_owned(self) -> MsgBuf<'static> {
        self.try_extend_lifetime().unwrap_or_else(|slf| {
            let mut buf = MsgBuf::from(Vec::with_capacity(slf.capacity()));
            Self { quota: buf.quota, fill: buf.fill, has_msg: buf.has_msg, .. } = slf;
            buf
        })
    }
    /// Attempts to extend lifetime to `'static`, failing if the buffer is borrowed.
    pub fn try_extend_lifetime(self) -> Result<MsgBuf<'static>, Self> {
        if self.borrow.is_none() || self.cap == 0 {
            let Self { ptr, cap, quota, init, borrow: _, fill, has_msg } = self;
            Ok(MsgBuf { ptr, cap, quota, init, borrow: None, fill, has_msg })
        } else {
            Err(self)
        }
    }
}
