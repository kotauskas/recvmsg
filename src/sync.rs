//! Non-async reliable message reception trait and its helpers.

mod fwd;
mod via;
pub use via::*;

pub(crate) mod r#impl {
    #[cfg(feature = "std_net")]
    pub(crate) mod net {
        #[cfg(unix)]
        pub(crate) mod unix;
        #[cfg(windows)]
        pub(crate) mod windows;

        #[cfg(test)]
        mod tests;
    }
}

use crate::{MsgBuf, RecvResult, TryRecvResult};

/// Receiving from socket-like connections with message boundaries with truncation detection.
pub trait TruncatingRecvMsg {
    /// The I/O error type.
    ///
    /// This exists not only to make error handling around this trait more flexible, but also to
    /// allow the crate to be `#![no_std]`.
    type Error;

    /// The buffer used for sender address reception.
    ///
    /// Conventionally `()` if sender addresses are not available.
    type AddrBuf;

    /// Receives one message into the given buffer, returning:
    /// - `Ok(Some(true))` if the message has been successfully received;
    /// - `Ok(Some(false))` if it was truncated due to insufficient buffer size;
    /// - `Ok(None)` to indicate end of communication ("EOF");
    /// - `Err(..)` if an I/O error occured.
    ///
    /// If `peek` is `true`, the message is not taken off the queue, meaning that a subsequent call
    /// will return the same message, with bigger buffer sizes receiving more of the message if it
    /// was truncated.
    ///
    /// In the `Ok(..)` cases, if `abuf` is `Some(..)`, it is filled with the address of the sender.
    ///
    /// # Contract notes
    /// - **Must** set `buf.is_one_msg` to `true` when returning `Ok(..)`.
    /// - **Must not** affect the capacity of `buf`.
    /// - **Must not** decrease the initialization cursor or the fill cursor of `buf`.
    /// - **Must** set the fill cursor to the size of the received message (size *after* truncation,
    ///   not actual size of the message) and not modify it in any other circumstances.
    fn recv_trunc(
        &mut self,
        peek: bool,
        buf: &mut MsgBuf<'_>,
        abuf: Option<&mut Self::AddrBuf>,
    ) -> Result<Option<bool>, Self::Error>;

    /// Discards the message at the front of the queue. If at end-of-communication, succeeds with no
    /// effect.
    fn discard_msg(&mut self) -> Result<(), Self::Error> {
        self.recv_trunc(false, &mut MsgBuf::from(&mut [0][..]), None)?;
        Ok(())
    }
}

/// Like [`TruncatingRecvMsg`], but reports the exact true size of truncated messages.
pub trait TruncatingRecvMsgWithFullSize: TruncatingRecvMsg {
    /// Like [`.recv_trunc()`](TruncatingRecvMsg::recv_trunc), but returns the true length of the
    /// message *(size before truncation)*.
    fn recv_trunc_with_full_size(
        &mut self,
        peek: bool,
        buf: &mut MsgBuf<'_>,
        abuf: Option<&mut Self::AddrBuf>,
    ) -> Result<TryRecvResult, Self::Error>;

    /// Attempts to receive one message using the given buffer. If the message at the front of the
    /// queue does not fit, no (re)allocation is done and the message is neither written to the
    /// buffer nor taken off the underlying queue.
    ///
    /// In the `Ok(..)` case, if `abuf` is `Some(..)`, it is filled with the address of the sender.
    ///
    /// If the operation could not be completed for external reasons, an error from the outermost
    /// `Result` is returned.
    ///
    /// This method simplifies use of `.recv_trunc_with_full_size()` by keeping `buf` consistent in
    /// error conditions and making the call to `.discard_msg()` implicitly as needed.
    fn try_recv_msg(
        &mut self,
        buf: &mut MsgBuf<'_>,
        abuf: Option<&mut Self::AddrBuf>,
    ) -> Result<TryRecvResult, Self::Error> {
        Ok(match self.recv_trunc_with_full_size(true, buf, abuf)? {
            TryRecvResult::Fit(sz) => {
                debug_assert_eq!(buf.len_filled(), sz);
                self.discard_msg()?;
                TryRecvResult::Fit(sz)
            }
            TryRecvResult::Spilled(sz) => {
                buf.set_fill(0);
                buf.has_msg = false;
                TryRecvResult::Spilled(sz)
            }
            TryRecvResult::EndOfStream => TryRecvResult::EndOfStream,
        })
    }
}

/// Receiving from socket-like connections with message boundaries without truncation.
pub trait RecvMsg {
    /// The I/O error type.
    ///
    /// This exists not only to make error handling around this trait more flexible, but also to
    /// allow the crate to be `#![no_std]`.
    type Error;

    /// The buffer used for sender address reception.
    ///
    /// Conventionally `()` if sender addresses are not available.
    type AddrBuf;

    /// Receives one message using the given buffer, (re)allocating the buffer if necessary.
    ///
    /// In the `Ok(..)` case, if `abuf` is `Some(..)`, it is filled with the address of the sender.
    ///
    /// If the operation could not be completed for external reasons, an error from the outermost
    /// `Result` is returned.
    fn recv_msg(
        &mut self,
        buf: &mut MsgBuf<'_>,
        abuf: Option<&mut Self::AddrBuf>,
    ) -> Result<RecvResult, Self::Error>;
}
