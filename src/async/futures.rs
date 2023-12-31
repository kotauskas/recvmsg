use super::*;
use crate::MsgBuf;
use core::future::Future;

macro_rules! futdoc {
    ($trait:ident :: $mtd:ident $($tt:tt)+) => {
        #[doc = concat!(
            "Future type returned by [`.", stringify!($mtd), "(`](", stringify!($trait), "::", stringify!($mtd), ")."
        )]
        $($tt)+
    };
}

futdoc! { TruncatingRecvMsgExt::recv_trunc
#[derive(Debug)]
pub struct RecvTrunc<'io, 'buf, 'slice, TRM: ?Sized> {
    pub(super) recver: &'io mut TRM,
    pub(super) peek: bool,
    pub(super) buf: &'buf mut MsgBuf<'slice>,
}}
impl<TRM: TruncatingRecvMsg + Unpin + ?Sized> Future for RecvTrunc<'_, '_, '_, TRM> {
    type Output = Result<Option<bool>, TRM::Error>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self { recver, peek, buf } = self.get_mut();
        Pin::new(&mut **recver).poll_recv_trunc(cx, *peek, buf)
    }
}

futdoc! { TruncatingRecvMsgExt::discard_msg
#[derive(Debug)]
pub struct DiscardMsg<'io, TRM: ?Sized> { pub(super) recver: &'io mut TRM }}
impl<TRM: TruncatingRecvMsg + Unpin + ?Sized> Future for DiscardMsg<'_, TRM> {
    type Output = Result<(), TRM::Error>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self { recver } = self.get_mut();
        Pin::new(&mut **recver).poll_discard_msg(cx)
    }
}

futdoc! { TruncatingRecvMsgWithFullSizeExt::recv_trunc_with_full_size
#[derive(Debug)]
pub struct RecvTruncWithFullSize<'io, 'buf, 'slice, TRMWFS: ?Sized> {
    pub(super) recver: &'io mut TRMWFS,
    pub(super) peek: bool,
    pub(super) buf: &'buf mut MsgBuf<'slice>,
}}
impl<TRMWFS: TruncatingRecvMsgWithFullSize + Unpin + ?Sized> Future
    for RecvTruncWithFullSize<'_, '_, '_, TRMWFS>
{
    type Output = Result<TryRecvResult, TRMWFS::Error>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self { recver, peek, buf } = self.get_mut();
        Pin::new(&mut **recver).poll_recv_trunc_with_full_size(cx, *peek, buf)
    }
}

futdoc! { TruncatingRecvMsgWithFullSizeExt::try_recv_msg
#[derive(Debug)]
pub struct TryRecv<'io, 'buf, 'slice, TRMWFS: ?Sized> {
    recver: &'io mut TRMWFS,
    state: TryRecvState<'buf, 'slice>,
}}
impl<'io, 'buf, 'slice, TRMWFS: ?Sized> TryRecv<'io, 'buf, 'slice, TRMWFS> {
    pub(super) fn new(recver: &'io mut TRMWFS, buf: &'buf mut MsgBuf<'slice>) -> Self {
        Self { recver, state: TryRecvState::Recving { buf } }
    }
}

#[derive(Debug)]
enum TryRecvState<'buf, 'slice> {
    Recving { buf: &'buf mut MsgBuf<'slice> },
    Discarding { sz: usize },
    End,
}

impl<TRMWFS: TruncatingRecvMsgWithFullSize + Unpin + ?Sized> Future
    for TryRecv<'_, '_, '_, TRMWFS>
{
    type Output = Result<TryRecvResult, TRMWFS::Error>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let slf = self.get_mut();
        match &mut slf.state {
            TryRecvState::Recving { buf } => {
                let Poll::Ready(rslt) =
                    Pin::new(&mut *slf.recver).poll_recv_trunc_with_full_size(cx, true, buf)?
                else {
                    return Poll::Pending;
                };
                match rslt {
                    TryRecvResult::Fit(sz) => {
                        debug_assert_eq!(buf.len_filled(), sz);
                        slf.state = TryRecvState::Discarding { sz };
                        Pin::new(slf).poll(cx)
                    }
                    TryRecvResult::Spilled(sz) => {
                        buf.set_fill(0);
                        buf.has_msg = false;
                        Poll::Ready(Ok(TryRecvResult::Spilled(sz)))
                    }
                    TryRecvResult::EndOfStream => Poll::Ready(Ok(TryRecvResult::EndOfStream)),
                }
            }
            TryRecvState::Discarding { sz } => {
                match Pin::new(&mut *slf.recver).poll_discard_msg(cx) {
                    Poll::Ready(r) => {
                        let sz = *sz;
                        slf.state = TryRecvState::End;
                        Poll::Ready(match r {
                            Ok(()) => Ok(TryRecvResult::Fit(sz)),
                            Err(e) => Err(e),
                        })
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
            TryRecvState::End => panic!("attempt to poll a future which has already completed"),
        }
    }
}

futdoc! { ReliableRecvMsgExt::recv_msg
#[derive(Debug)]
pub struct Recv<'io, 'buf, 'slice: 'buf, RM: ?Sized> {
    pub(super) recver: &'io mut RM,
    pub(super) buf: &'buf mut MsgBuf<'slice>,
}}
impl<'buf, RM: RecvMsg + Unpin + ?Sized> Future for Recv<'_, 'buf, '_, RM> {
    type Output = Result<RecvResult, RM::Error>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Recv { recver, buf } = self.get_mut();
        Pin::new(&mut **recver).poll_recv_msg(cx, buf)
    }
}
