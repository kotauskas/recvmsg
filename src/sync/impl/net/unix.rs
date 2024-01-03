#![allow(unsafe_code)]

mod extract_address;
mod r#impl;
pub(crate) use extract_address::{extract_ip_address, extract_unix_address};
pub(crate) use r#impl::*;

use crate::{MsgBuf, RecvMsg, RecvResult, TruncatingRecvMsg, TryRecvResult};
use libc::sockaddr_storage;
use std::{
    io,
    mem::zeroed,
    net::{SocketAddr as InetAddr, UdpSocket},
    os::{
        fd::AsFd,
        unix::net::{SocketAddr as UnixAddr, UnixDatagram},
    },
};

pub(crate) fn sockaddr_storage() -> sockaddr_storage {
    unsafe { zeroed() }
}

impl TruncatingRecvMsg for &UdpSocket {
    type Error = io::Error;
    type AddrBuf = InetAddr;
    #[inline]
    fn recv_trunc(
        &mut self,
        peek: bool,
        buf: &mut MsgBuf<'_>,
        abuf: Option<&mut InetAddr>,
    ) -> io::Result<Option<bool>> {
        let mut fused_abuf = (sockaddr_storage(), 0);
        let ret = recv_trunc(self.as_fd(), peek, buf, abuf.is_some().then_some(&mut fused_abuf))?;
        if let Some(abuf) = abuf {
            *abuf = extract_ip_address(&fused_abuf.0)?;
        }
        Ok(ret)
    }
}

impl TruncatingRecvMsg for UdpSocket {
    type Error = io::Error;
    type AddrBuf = InetAddr;
    #[inline]
    fn recv_trunc(
        &mut self,
        peek: bool,
        buf: &mut MsgBuf<'_>,
        abuf: Option<&mut InetAddr>,
    ) -> io::Result<Option<bool>> {
        (&*self).recv_trunc(peek, buf, abuf)
    }
}

/// Linux-only, requires kernel 3.4 or newer.
#[cfg(any(target_os = "linux", target_os = "android"))]
impl crate::TruncatingRecvMsgWithFullSize for &UdpSocket {
    #[inline]
    fn recv_trunc_with_full_size(
        &mut self,
        peek: bool,
        buf: &mut MsgBuf<'_>,
        abuf: Option<&mut InetAddr>,
    ) -> io::Result<TryRecvResult> {
        let mut fused_abuf = (sockaddr_storage(), 0);
        let ret = recv_trunc_with_full_size(
            self.as_fd(),
            peek,
            buf,
            abuf.is_some().then_some(&mut fused_abuf),
        )?;
        if let Some(abuf) = abuf {
            *abuf = extract_ip_address(&fused_abuf.0)?;
        }
        Ok(ret)
    }
}

/// Linux-only, requires kernel 3.4 or newer.
#[cfg(any(target_os = "linux", target_os = "android"))]
impl crate::TruncatingRecvMsgWithFullSize for UdpSocket {
    #[inline]
    fn recv_trunc_with_full_size(
        &mut self,
        peek: bool,
        buf: &mut MsgBuf<'_>,
        abuf: Option<&mut InetAddr>,
    ) -> io::Result<TryRecvResult> {
        (&*self).recv_trunc_with_full_size(peek, buf, abuf)
    }
}

impl RecvMsg for &UdpSocket {
    type Error = io::Error;
    type AddrBuf = InetAddr;
    #[inline]
    fn recv_msg(
        &mut self,
        buf: &mut MsgBuf<'_>,
        abuf: Option<&mut InetAddr>,
    ) -> io::Result<RecvResult> {
        let mut fused_abuf = (sockaddr_storage(), 0);
        let ret = recv_msg(self.as_fd(), buf, abuf.is_some().then_some(&mut fused_abuf))?;
        if let Some(abuf) = abuf {
            *abuf = extract_ip_address(&fused_abuf.0)?;
        }
        Ok(ret)
    }
}

impl RecvMsg for UdpSocket {
    type Error = io::Error;
    type AddrBuf = InetAddr;
    #[inline]
    fn recv_msg(
        &mut self,
        buf: &mut MsgBuf<'_>,
        abuf: Option<&mut InetAddr>,
    ) -> io::Result<RecvResult> {
        (&mut &*self).recv_msg(buf, abuf)
    }
}

impl TruncatingRecvMsg for &UnixDatagram {
    type Error = io::Error;
    type AddrBuf = UnixAddr;
    #[inline]
    fn recv_trunc(
        &mut self,
        peek: bool,
        buf: &mut MsgBuf<'_>,
        abuf: Option<&mut UnixAddr>,
    ) -> io::Result<Option<bool>> {
        let mut fused_abuf = (sockaddr_storage(), 0);
        let ret = recv_trunc(self.as_fd(), peek, buf, abuf.is_some().then_some(&mut fused_abuf))?;
        if let Some(abuf) = abuf {
            *abuf = extract_unix_address(&fused_abuf.0, fused_abuf.1)?;
        }
        Ok(ret)
    }
}

impl TruncatingRecvMsg for UnixDatagram {
    type Error = io::Error;
    type AddrBuf = UnixAddr;
    #[inline]
    fn recv_trunc(
        &mut self,
        peek: bool,
        buf: &mut MsgBuf<'_>,
        abuf: Option<&mut UnixAddr>,
    ) -> io::Result<Option<bool>> {
        (&*self).recv_trunc(peek, buf, abuf)
    }
}

/// Linux-only, requires kernel 3.4 or newer.
#[cfg(any(target_os = "linux", target_os = "android"))]
impl crate::TruncatingRecvMsgWithFullSize for &UnixDatagram {
    #[inline]
    fn recv_trunc_with_full_size(
        &mut self,
        peek: bool,
        buf: &mut MsgBuf<'_>,
        abuf: Option<&mut UnixAddr>,
    ) -> io::Result<TryRecvResult> {
        let mut fused_abuf = (sockaddr_storage(), 0);
        let ret = recv_trunc_with_full_size(
            self.as_fd(),
            peek,
            buf,
            abuf.is_some().then_some(&mut fused_abuf),
        )?;
        if let Some(abuf) = abuf {
            *abuf = extract_unix_address(&fused_abuf.0, fused_abuf.1)?;
        }
        Ok(ret)
    }
}

/// Linux-only, requires kernel 3.4 or newer.
#[cfg(any(target_os = "linux", target_os = "android"))]
impl crate::TruncatingRecvMsgWithFullSize for UnixDatagram {
    #[inline]
    fn recv_trunc_with_full_size(
        &mut self,
        peek: bool,
        buf: &mut MsgBuf<'_>,
        abuf: Option<&mut UnixAddr>,
    ) -> io::Result<TryRecvResult> {
        (&*self).recv_trunc_with_full_size(peek, buf, abuf)
    }
}

impl RecvMsg for &UnixDatagram {
    type Error = io::Error;
    type AddrBuf = UnixAddr;
    #[inline]
    fn recv_msg(
        &mut self,
        buf: &mut MsgBuf<'_>,
        abuf: Option<&mut UnixAddr>,
    ) -> io::Result<RecvResult> {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            crate::sync::recv_via_try_recv(self, buf, abuf)
        }
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        {
            crate::sync::recv_via_recv_trunc(self, buf, abuf)
        }
    }
}

impl RecvMsg for UnixDatagram {
    type Error = io::Error;
    type AddrBuf = UnixAddr;
    #[inline]
    fn recv_msg(
        &mut self,
        buf: &mut MsgBuf<'_>,
        abuf: Option<&mut UnixAddr>,
    ) -> io::Result<RecvResult> {
        (&mut &*self).recv_msg(buf, abuf)
    }
}
