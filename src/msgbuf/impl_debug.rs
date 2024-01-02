use super::{MsgBuf, OwnedBuf};
use core::fmt::{self, Debug, Formatter};

impl<Owned: OwnedBuf> Debug for MsgBuf<'_, Owned> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let quota: &(dyn Debug + '_) = match self.quota {
            Some(ref q) => q,
            None => &None::<usize>,
        };
        f.debug_struct("MsgBuf")
            .field("ptr", &self.ptr)
            .field("cap", &self.cap)
            .field("owned", &self.borrow.is_none())
            .field("quota", quota)
            .field("init", &self.init)
            .field("fill", &self.fill)
            .field("has_msg", &self.has_msg)
            .finish()
    }
}
