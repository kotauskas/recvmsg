use super::*;
use crate::panic_try_recv_retcon;
use core::future::Future;

#[cfg_attr(debug_assertions, track_caller)]
fn dbgtrp(msg: &str) {
    if cfg!(debug_assertions) {
        panic!("{msg}");
    }
}

/// Implements [`TruncatingRecvMsg::poll_recv_trunc()`] via
/// [`TruncatingRecvMsgWithFullSize::poll_recv_trunc_with_full_size()`].
pub fn poll_recv_trunc_via_poll_recv_trunc_with_full_size<
    ATRMWFS: TruncatingRecvMsgWithFullSize + ?Sized,
>(
    slf: Pin<&mut ATRMWFS>,
    cx: &mut Context<'_>,
    peek: bool,
    buf: &mut MsgBuf<'_>,
) -> Poll<Result<Option<bool>, ATRMWFS::Error>> {
    let cap = buf.len();
    let rslt = ready!(slf.poll_recv_trunc_with_full_size(cx, peek, buf)?);
    debug_assert_eq!(buf.len(), cap, "`recv_trunc_with_size()` changed buffer capacity");
    Ok(match rslt {
        TryRecvResult::Fit(..) => Some(true),
        TryRecvResult::Spilled(..) => Some(false),
        TryRecvResult::EndOfStream => None,
    })
    .into()
}

/// Implements [`RecvMsg::poll_recv_msg()`] via [`TruncatingRecvMsg::poll_recv_trunc()`].
pub fn poll_recv_via_poll_recv_trunc<ATRM: TruncatingRecvMsg + ?Sized>(
    mut slf: Pin<&mut ATRM>,
    cx: &mut Context<'_>,
    buf: &mut MsgBuf<'_>,
) -> Poll<Result<RecvResult, ATRM::Error>> {
    let mut fit_first = true;
    let mut first = true;
    loop {
        let rr = match Pin::new(&mut slf).poll_recv_trunc(cx, true, buf) {
            Poll::Ready(r) => r,
            Poll::Pending => {
                if !first {
                    #[rustfmt::skip] dbgtrp("\
.poll_recv_trunc() returned Poll::Pending after having returned Poll::Ready with peek = true");
                }
                return Poll::Pending;
            }
        };
        let fit = match rr {
            Ok(Some(fit)) => fit,
            Ok(None) => return Ok(RecvResult::EndOfStream).into(),
            Err(e) => {
                buf.set_fill(0);
                buf.has_msg = false;
                return Err(e).into();
            }
        };
        if first && !fit {
            fit_first = false;
        }
        first = false;
        if fit {
            break;
        } else {
            buf.set_fill(0);
            if let Err(qe) = buf.ensure_capacity(buf.len() * 2) {
                return Poll::Ready(Ok(RecvResult::QuotaExceeded(qe)));
            }
        }
    }
    match slf.poll_discard_msg(cx) {
        Poll::Ready(Ok(())) => {}
        Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
        Poll::Pending => panic!(".poll_discard_msg() returned Poll::Pending after successful peek"),
    }
    Ok(if fit_first {
        RecvResult::Fit(buf.len_filled())
    } else {
        RecvResult::Spilled(buf.len_filled())
    })
    .into()
}

/// Implements [`RecvMsg::poll_recv_msg()`] via
/// [`TruncatingRecvMsgWithFullSizeExt::try_recv_msg()`].
pub fn poll_recv_via_poll_try_recv<TRMWFS: TruncatingRecvMsgWithFullSize + ?Sized>(
    mut slf: Pin<&mut TRMWFS>,
    cx: &mut Context<'_>,
    buf: &mut MsgBuf<'_>,
) -> Poll<Result<RecvResult, TRMWFS::Error>> {
    let mut poll_try_recv = |buf: &mut MsgBuf<'_>| Pin::new(&mut slf.try_recv_msg(buf)).poll(cx);
    let ok = match ready!(poll_try_recv(buf)?).into() {
        RecvResult::Spilled(sz) => {
            if let Err(qe) = buf.ensure_capacity(sz) {
                return Ok(RecvResult::QuotaExceeded(qe)).into();
            }
            let fitsz = match ready!(poll_try_recv(buf)?) {
                TryRecvResult::Fit(sz) => sz,
                TryRecvResult::Spilled(..) => panic_try_recv_retcon(),
                TryRecvResult::EndOfStream => return Ok(RecvResult::EndOfStream).into(),
            };
            debug_assert_eq!(sz, fitsz);
            RecvResult::Spilled(sz)
        }
        fit_or_end => fit_or_end,
    };
    Ok(ok).into()
}
