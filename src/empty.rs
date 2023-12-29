use crate::{
    AsyncRecvMsg, AsyncTruncatingRecvMsg, AsyncTruncatingRecvMsgWithFullSize, MsgBuf, RecvMsg,
    RecvResult, TruncatingRecvMsg, TruncatingRecvMsgWithFullSize, TryRecvResult,
};
use core::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
};

/// Dummy message stream that is at end-of-stream from the outset.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Empty;

impl TruncatingRecvMsg for Empty {
    type Error = Infallible;
    #[inline(always)]
    fn recv_trunc(&mut self, _: bool, _: &mut MsgBuf<'_>) -> Result<Option<bool>, Self::Error> {
        Ok(None)
    }
}
impl TruncatingRecvMsgWithFullSize for Empty {
    fn recv_trunc_with_full_size(
        &mut self,
        _: bool,
        _: &mut MsgBuf<'_>,
    ) -> Result<TryRecvResult, Self::Error> {
        Ok(TryRecvResult::EndOfStream)
    }
}
impl RecvMsg for Empty {
    type Error = Infallible;
    #[inline(always)]
    fn recv_msg(&mut self, _: &mut MsgBuf<'_>) -> Result<RecvResult, Self::Error> {
        Ok(RecvResult::EndOfStream)
    }
}

impl AsyncTruncatingRecvMsg for Empty {
    type Error = Infallible;
    #[inline(always)]
    fn poll_recv_trunc(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        _: bool,
        _: &mut MsgBuf<'_>,
    ) -> Poll<Result<Option<bool>, Self::Error>> {
        Ok(None).into()
    }
}
impl AsyncTruncatingRecvMsgWithFullSize for Empty {
    #[inline(always)]
    fn poll_recv_trunc_with_full_size(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        _: bool,
        _: &mut MsgBuf<'_>,
    ) -> Poll<Result<TryRecvResult, Self::Error>> {
        Ok(TryRecvResult::EndOfStream).into()
    }
}
impl AsyncRecvMsg for Empty {
    type Error = Infallible;
    #[inline(always)]
    fn poll_recv_msg(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        _: &mut MsgBuf<'_>,
    ) -> Poll<Result<RecvResult, Self::Error>> {
        Ok(RecvResult::EndOfStream).into()
    }
}
