#![allow(unsafe_code)]

use crate::MsgBuf;
use core::cmp::min;
use libc::{iovec, msghdr, recvmsg, MSG_TRUNC};
use std::{
    io,
    os::fd::{AsRawFd, BorrowedFd},
};

/// Implements [`.recv_trunc()`](crate::TruncatingRecvMsg::recv_trunc) via `recvmsg()` with DI on
/// `msghdr` fields this crate is not concerned with. (The only modification made to the provided
/// `hdr` is to `msg_iov` and `msg_iovlen`, in addition to the system's doings.) The return value of
/// the `recvmsg()` call is additionally returned for convenience.
///
/// # Safety
/// Pointers in `hdr`, save for `msg_iov` and `msg_iovlen`, must be valid from the system's
/// perspective.
pub unsafe fn recv_trunc_recvmsg_with_msghdr(
    fd: BorrowedFd,
    hdr: &mut msghdr,
    buf: &mut MsgBuf<'_>,
    flags: i32,
) -> io::Result<(Option<bool>, usize)> {
    buf.set_fill(0);
    buf.has_msg = false;

    let out = &mut buf[..];
    let mut iov = iovec { iov_base: out.as_mut_ptr().cast(), iov_len: out.len() };
    hdr.msg_iov = &mut iov;
    hdr.msg_iovlen = 1;

    let bytes_recved = unsafe {
        // SAFETY: msghdr is zero-initialized except for the validly initialized iovec
        recvmsg(fd.as_raw_fd(), hdr, flags)
    };
    let bytes_recved = if bytes_recved < 0 {
        return Err(io::Error::last_os_error());
    } else {
        bytes_recved as usize
    };
    // For MSG_TRUNC on Linux
    let bytes_recved_to_set = min(bytes_recved, buf.capacity());

    unsafe { buf.set_init(bytes_recved_to_set) };
    buf.set_fill(bytes_recved_to_set);

    Ok((
        if bytes_recved > 0 {
            buf.has_msg = true;
            let fit = hdr.msg_flags & MSG_TRUNC == 0;
            Some(fit)
        } else {
            None // bytes_recved == 0
        },
        bytes_recved,
    ))
}
