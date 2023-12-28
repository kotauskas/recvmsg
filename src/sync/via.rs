use super::*;
use crate::panic_try_recv_retcon;

/// Implements [`TruncatingRecvMsg::recv_trunc()`] via
/// [`TruncatingRecvMsgWithFullSize::recv_trunc_with_full_size()`].
pub fn recv_trunc_via_recv_trunc_with_full_size<TRMWFS: TruncatingRecvMsgWithFullSize + ?Sized>(
    slf: &mut TRMWFS,
    peek: bool,
    buf: &mut MsgBuf<'_>,
) -> Result<Option<bool>, TRMWFS::Error> {
    let cap = buf.len();
    let rslt = slf.recv_trunc_with_full_size(peek, buf)?;
    debug_assert_eq!(buf.len(), cap, "`recv_trunc_with_size()` changed buffer capacity");
    Ok(match rslt {
        TryRecvResult::Fit(..) => Some(true),
        TryRecvResult::Spilled(..) => Some(false),
        TryRecvResult::EndOfStream => None,
    })
}

/// Implements [`RecvMsg::recv_msg()`] via [`TruncatingRecvMsg::recv_trunc()`].
pub fn recv_via_recv_trunc<TRM: TruncatingRecvMsg + ?Sized>(
    slf: &mut TRM,
    buf: &mut MsgBuf<'_>,
) -> Result<RecvResult, TRM::Error> {
    let mut fit_first = true;
    loop {
        let fit = match slf.recv_trunc(true, buf) {
            Ok(Some(fit)) => fit,
            Ok(None) => return Ok(RecvResult::EndOfStream),
            Err(e) => {
                buf.set_fill(0);
                buf.is_one_msg = false;
                return Err(e);
            }
        };
        if fit {
            break;
        } else {
            fit_first = false;
            buf.set_fill(0);
            if let Err(qe) = buf.ensure_capacity(buf.len() * 2) {
                return Ok(RecvResult::QuotaExceeded(qe));
            }
        }
    }
    slf.discard_msg()?;
    Ok(if fit_first {
        RecvResult::Fit(buf.len_filled())
    } else {
        RecvResult::Spilled(buf.len_filled())
    })
}

/// Implements [`RecvMsg::recv_msg()`] via [`TruncatingRecvMsgWithFullSize::try_recv_msg()`].
pub fn recv_via_try_recv<TRMWFS: TruncatingRecvMsgWithFullSize + ?Sized>(
    slf: &mut TRMWFS,
    buf: &mut MsgBuf<'_>,
) -> Result<RecvResult, TRMWFS::Error> {
    let ok = match slf.try_recv_msg(buf)?.into() {
        RecvResult::Spilled(sz) => {
            if let Err(qe) = buf.ensure_capacity(sz) {
                return Ok(RecvResult::QuotaExceeded(qe));
            }
            let fitsz = match slf.try_recv_msg(buf)? {
                TryRecvResult::Fit(sz) => sz,
                TryRecvResult::Spilled(..) => panic_try_recv_retcon(),
                TryRecvResult::EndOfStream => return Ok(RecvResult::EndOfStream),
            };
            debug_assert_eq!(sz, fitsz);
            RecvResult::Spilled(sz)
        }
        fit_or_end => fit_or_end,
    };
    Ok(ok)
}
