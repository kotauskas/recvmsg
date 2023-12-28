use crate::{sync::r#impl::net::unix as syncimpl, MsgBuf, RecvResult};
use std::{io, os::fd::AsFd};
use tokio::net;

fn recv_trunc(slf: &impl AsFd, peek: bool, buf: &mut MsgBuf<'_>) -> io::Result<Option<bool>> {
    syncimpl::recv_trunc(slf.as_fd(), peek, buf)
}
fn recv_msg(slf: &impl AsFd, buf: &mut MsgBuf<'_>) -> io::Result<RecvResult> {
    syncimpl::recv_msg(slf.as_fd(), buf)
}
#[cfg(any(target_os = "linux", target_os = "android"))]
fn recv_trunc_with_full_size(
    slf: &impl AsFd,
    peek: bool,
    buf: &mut MsgBuf<'_>,
) -> io::Result<crate::TryRecvResult> {
    syncimpl::recv_trunc_with_full_size(slf.as_fd(), peek, buf)
}

impl_atrm!(for [net::UdpSocket, net::UnixDatagram], with recv_trunc);
impl_arm!(for [net::UdpSocket, net::UnixDatagram], with recv_msg);
#[cfg(any(target_os = "linux", target_os = "android"))]
impl_atrmwfs!(for [net::UdpSocket, net::UnixDatagram], with recv_trunc_with_full_size);
