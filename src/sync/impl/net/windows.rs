use crate::{MsgBuf, RecvMsg, RecvResult, TruncatingRecvMsg};
use std::{io, net::UdpSocket};

// TODO RecvEx?

const WSAEMSGSIZE: i32 = 10040;

pub(crate) fn recv_trunc(
    buf: &mut MsgBuf<'_>,
    f: impl FnOnce(&mut [u8]) -> io::Result<usize>,
) -> io::Result<Option<bool>> {
    buf.set_fill(0);
    buf.is_one_msg = false;
    buf.fully_initialize();
    match f(buf.init_part_mut()) {
        Ok(0) => Ok(None),
        Ok(sz) => {
            buf.set_fill(sz);
            buf.is_one_msg = true;
            Ok(Some(true))
        }
        Err(e) if e.raw_os_error() == Some(WSAEMSGSIZE) => Ok(Some(false)),
        Err(e) => Err(e),
    }
}

impl TruncatingRecvMsg for &UdpSocket {
    type Error = io::Error;
    fn recv_trunc(&mut self, peek: bool, buf: &mut MsgBuf<'_>) -> io::Result<Option<bool>> {
        let op = if peek { UdpSocket::peek } else { UdpSocket::recv };
        recv_trunc(buf, |b| op(self, b))
    }
}

impl TruncatingRecvMsg for UdpSocket {
    type Error = io::Error;
    #[inline]
    fn recv_trunc(&mut self, peek: bool, buf: &mut MsgBuf<'_>) -> io::Result<Option<bool>> {
        (&*self).recv_trunc(peek, buf)
    }
}

impl RecvMsg for &UdpSocket {
    type Error = io::Error;
    #[inline]
    fn recv_msg(&mut self, buf: &mut MsgBuf<'_>) -> io::Result<RecvResult> {
        crate::sync::recv_via_recv_trunc(self, buf)
    }
}
impl RecvMsg for UdpSocket {
    type Error = io::Error;
    #[inline]
    fn recv_msg(&mut self, buf: &mut MsgBuf<'_>) -> io::Result<RecvResult> {
        RecvMsg::recv_msg(&mut &*self, buf)
    }
}
