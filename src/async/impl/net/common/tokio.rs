#![allow(unused_macros)]

macro_rules! impl_atrm {
(for [$($ty:ty),+ $(,)?], with $lfn:path) => { mod _impl_atrm {
#[allow(unused_imports)]
use super::*;

use crate::{r#async::ioloop, AsyncTruncatingRecvMsg, MsgBuf};
use std::{
    io,
    pin::Pin,
    net::SocketAddr,
    task::{Context, Poll},
};

$(
    impl AsyncTruncatingRecvMsg for &$ty {
        type Error = io::Error;
        type AddrBuf = SocketAddr;
        fn poll_recv_trunc(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            peek: bool,
            buf: &mut MsgBuf<'_>,
            mut abuf: Option<&mut SocketAddr>,
        ) -> Poll<io::Result<Option<bool>>> {
            ioloop(
                self.get_mut(), cx,
                |slf: &mut Self| $lfn(slf, peek, buf, abuf.as_deref_mut()),
                |slf: &mut Self, cx| slf.poll_recv_ready(cx),
            )
        }
    }
    impl AsyncTruncatingRecvMsg for $ty {
        type Error = io::Error;
        type AddrBuf = SocketAddr;
        #[inline]
        fn poll_recv_trunc(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            peek: bool,
            buf: &mut MsgBuf<'_>,
            abuf: Option<&mut SocketAddr>,
        ) -> Poll<io::Result<Option<bool>>> {
            Pin::new(&mut &*self).poll_recv_trunc(cx, peek, buf, abuf)
        }
    }
)+
}};}

macro_rules! impl_atrmwfs {
(for [$($ty:ty),+ $(,)?], with $lfn:path) => { mod _impl_atrmwfs {
#[allow(unused_imports)]
use super::*;

use crate::{r#async::ioloop, AsyncTruncatingRecvMsgWithFullSize, MsgBuf, TryRecvResult};
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

$(
    impl AsyncTruncatingRecvMsgWithFullSize for &$ty {
        fn poll_recv_trunc_with_full_size(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            peek: bool,
            buf: &mut MsgBuf<'_>,
            mut abuf: Option<&mut SocketAddr>,
        ) -> Poll<io::Result<TryRecvResult>> {
            ioloop(
                self.get_mut(), cx,
                |slf: &mut Self| $lfn(slf, peek, buf, abuf.as_deref_mut()),
                |slf: &mut Self, cx| slf.poll_recv_ready(cx),
            )
        }
    }
    impl AsyncTruncatingRecvMsgWithFullSize for $ty {
        #[inline]
        fn poll_recv_trunc_with_full_size(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            peek: bool,
            buf: &mut MsgBuf<'_>,
            mut abuf: Option<&mut SocketAddr>,
        ) -> Poll<io::Result<TryRecvResult>> {
            Pin::new(&mut &*self).poll_recv_trunc_with_full_size(cx, peek, buf, abuf.as_deref_mut())
        }
    }
)+
}};}

macro_rules! impl_arm {
(for [$($ty:ty),+ $(,)?], with $lfn:path) => { mod _impl_arm {
#[allow(unused_imports)]
use super::*;

use crate::{r#async::ioloop, AsyncRecvMsg, MsgBuf, RecvResult};
use std::{
    io,
    pin::Pin,
    net::SocketAddr,
    task::{Context, Poll},
};

$(
    impl AsyncRecvMsg for &$ty {
        type Error = io::Error;
        type AddrBuf = SocketAddr;
        fn poll_recv_msg(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut MsgBuf<'_>,
            mut abuf: Option<&mut SocketAddr>,
        ) -> Poll<io::Result<RecvResult>> {
            ioloop(
                self.get_mut(), cx,
                |slf: &mut Self| $lfn(slf, buf, abuf.as_deref_mut()),
                |slf: &mut Self, cx| slf.poll_recv_ready(cx),
            )

        }
    }
    impl AsyncRecvMsg for $ty {
        type Error = io::Error;
        type AddrBuf = SocketAddr;
        #[inline]
        fn poll_recv_msg(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut MsgBuf<'_>,
            abuf: Option<&mut SocketAddr>,
        ) -> Poll<io::Result<RecvResult>> {
            Pin::new(&mut &*self).poll_recv_msg(cx, buf, abuf)
        }
    }
)+
}};}
