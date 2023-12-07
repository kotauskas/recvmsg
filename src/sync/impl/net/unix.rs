#![allow(unsafe_code)]

use crate::{os::unix::recv_trunc_recvmsg_with_msghdr, MsgBuf, RecvMsg, RecvResult, TruncatingRecvMsg, TryRecvResult};
use libc::{msghdr, MSG_PEEK};
use std::{
    io,
    mem::zeroed,
    net::UdpSocket,
    os::{
        fd::{AsFd, BorrowedFd},
        unix::net::UnixDatagram,
    },
};

pub(crate) fn recv_trunc(fd: BorrowedFd<'_>, peek: bool, buf: &mut MsgBuf<'_>) -> io::Result<Option<bool>> {
    unsafe {
        let mut hdr = zeroed::<msghdr>();
        Ok(recv_trunc_recvmsg_with_msghdr(fd, &mut hdr, buf, if peek { MSG_PEEK } else { 0 })?.0)
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub(crate) fn recv_trunc_with_full_size(
    fd: BorrowedFd<'_>,
    peek: bool,
    buf: &mut MsgBuf<'_>,
) -> io::Result<TryRecvResult> {
    Ok(
        match unsafe {
            let mut hdr = zeroed::<msghdr>();
            recv_trunc_recvmsg_with_msghdr(fd, &mut hdr, buf, libc::MSG_TRUNC | if peek { MSG_PEEK } else { 0 })?
        } {
            (Some(true), sz) => TryRecvResult::Fit(sz),
            (Some(false), sz) => TryRecvResult::Spilled(sz),
            (None, ..) => TryRecvResult::EndOfStream,
        },
    )
}

impl TruncatingRecvMsg for &UdpSocket {
    type Error = io::Error;
    #[inline]
    fn recv_trunc(&mut self, peek: bool, buf: &mut MsgBuf<'_>) -> io::Result<Option<bool>> {
        recv_trunc(self.as_fd(), peek, buf)
    }
}

impl TruncatingRecvMsg for UdpSocket {
    type Error = io::Error;
    #[inline]
    fn recv_trunc(&mut self, peek: bool, buf: &mut MsgBuf<'_>) -> io::Result<Option<bool>> {
        (&*self).recv_trunc(peek, buf)
    }
}

/// Linux-only, requires kernel 3.4 or newer.
#[cfg(any(target_os = "linux", target_os = "android"))]
impl crate::TruncatingRecvMsgWithFullSize for &UdpSocket {
    #[inline]
    fn recv_trunc_with_full_size(&mut self, peek: bool, buf: &mut MsgBuf<'_>) -> io::Result<TryRecvResult> {
        recv_trunc_with_full_size(self.as_fd(), peek, buf)
    }
}

/// Linux-only, requires kernel 3.4 or newer.
#[cfg(any(target_os = "linux", target_os = "android"))]
impl crate::TruncatingRecvMsgWithFullSize for UdpSocket {
    #[inline]
    fn recv_trunc_with_full_size(&mut self, peek: bool, buf: &mut MsgBuf<'_>) -> io::Result<TryRecvResult> {
        (&*self).recv_trunc_with_full_size(peek, buf)
    }
}

impl RecvMsg for &UdpSocket {
    type Error = io::Error;
    #[inline]
    fn recv(&mut self, buf: &mut MsgBuf<'_>) -> io::Result<RecvResult> {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            crate::sync::recv_via_try_recv(self, buf)
        }
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        {
            crate::sync::recv_via_recv_trunc(self, buf)
        }
    }
}

impl RecvMsg for UdpSocket {
    type Error = io::Error;
    #[inline]
    fn recv(&mut self, buf: &mut MsgBuf<'_>) -> io::Result<RecvResult> {
        (&mut &*self).recv(buf)
    }
}

impl TruncatingRecvMsg for &UnixDatagram {
    type Error = io::Error;
    #[inline]
    fn recv_trunc(&mut self, peek: bool, buf: &mut MsgBuf<'_>) -> io::Result<Option<bool>> {
        recv_trunc(self.as_fd(), peek, buf)
    }
}

impl TruncatingRecvMsg for UnixDatagram {
    type Error = io::Error;
    #[inline]
    fn recv_trunc(&mut self, peek: bool, buf: &mut MsgBuf<'_>) -> io::Result<Option<bool>> {
        (&*self).recv_trunc(peek, buf)
    }
}

/// Linux-only, requires kernel 3.4 or newer.
#[cfg(any(target_os = "linux", target_os = "android"))]
impl crate::TruncatingRecvMsgWithFullSize for &UnixDatagram {
    #[inline]
    fn recv_trunc_with_full_size(&mut self, peek: bool, buf: &mut MsgBuf<'_>) -> io::Result<TryRecvResult> {
        recv_trunc_with_full_size(self.as_fd(), peek, buf)
    }
}

/// Linux-only, requires kernel 3.4 or newer.
#[cfg(any(target_os = "linux", target_os = "android"))]
impl crate::TruncatingRecvMsgWithFullSize for UnixDatagram {
    #[inline]
    fn recv_trunc_with_full_size(&mut self, peek: bool, buf: &mut MsgBuf<'_>) -> io::Result<TryRecvResult> {
        (&*self).recv_trunc_with_full_size(peek, buf)
    }
}

impl RecvMsg for &UnixDatagram {
    type Error = io::Error;
    #[inline]
    fn recv(&mut self, buf: &mut MsgBuf<'_>) -> io::Result<RecvResult> {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            crate::sync::recv_via_try_recv(self, buf)
        }
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        {
            crate::sync::recv_via_recv_trunc(self, buf)
        }
    }
}

impl RecvMsg for UnixDatagram {
    type Error = io::Error;
    #[inline]
    fn recv(&mut self, buf: &mut MsgBuf<'_>) -> io::Result<RecvResult> {
        (&mut &*self).recv(buf)
    }
}
