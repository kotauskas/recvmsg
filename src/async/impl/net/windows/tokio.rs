use crate::{sync::r#impl::net::windows as syncimpl, AsyncRecvMsg, MsgBuf, RecvResult};
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use std::io;
use tokio::net::UdpSocket;

fn recv_trunc(slf: &UdpSocket, peek: bool, buf: &mut MsgBuf<'_>) -> io::Result<Option<bool>> {
    let op = if peek {
        |s: &UdpSocket, b: &'_ mut _| s.try_peek_from(b).map(|(s, _)| s)
    } else {
        |s: &UdpSocket, b: &'_ mut _| s.try_recv(b)
    };
    syncimpl::recv_trunc(buf, move |b| op(slf, b))
}

impl_atrm!(for [UdpSocket], with recv_trunc);

// TODO use macro when RecvEx is introduced
impl AsyncRecvMsg for &UdpSocket {
    type Error = io::Error;
    #[inline]
    fn poll_recv_msg(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut MsgBuf<'_>,
    ) -> Poll<io::Result<RecvResult>> {
        crate::r#async::poll_recv_via_poll_recv_trunc(self, cx, buf)
    }
}
impl AsyncRecvMsg for UdpSocket {
    type Error = io::Error;
    #[inline]
    fn poll_recv_msg(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut MsgBuf<'_>,
    ) -> Poll<io::Result<RecvResult>> {
        Pin::new(&mut &*self).poll_recv_msg(cx, buf)
    }
}

// TODO
//impl_atrmwfs!(for [net::UdpSocket], with syncimpl::recv_trunc_with_full_size);
